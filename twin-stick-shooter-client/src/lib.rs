use cgmath::num_traits::zero;
use cgmath::vec2;
use legion::IntoQuery;
use std::sync::{Arc, Mutex};
use twin_stick_shooter_core::collision::Shape;
use twin_stick_shooter_core::component::{
    Hitbox, HitboxState, Hurtbox, HurtboxState, Player, Position,
};
use twin_stick_shooter_core::resource::Input;
use twin_stick_shooter_core::Game;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Gamepad, GamepadButton, HtmlCanvasElement, Window};

mod time_accumulator;

use time_accumulator::{Seconds, TimeAccumulator};

use crate::time_accumulator::Milliseconds;

pub struct App {
    game: Game,
    last_dimensions: Option<(u32, u32)>,
    time_accumulator: TimeAccumulator,

    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    draw_callback: Option<Closure<dyn FnMut(f64)>>,
}

#[wasm_bindgen]
pub fn launch() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Construct the underlying game model.
    let game = Game::new();

    // Look up some objects in the JS environment.
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Set up the main canvas element.
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    body.append_child(&canvas).unwrap();
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    // Construct the singleton App instance inside an Arc and a Mutex. This makes it easy to
    // ensure the App instance lives long enough for all of the various asynchronous callbacks
    // that it will register.
    let app = Arc::new(Mutex::new(App {
        game,
        last_dimensions: None,
        time_accumulator: TimeAccumulator::default(),

        canvas,
        ctx,
        draw_callback: None,
    }));
    let mut app_mut = app.lock().unwrap();

    // Request an animation frame callback for the first update_and_draw. Each update_and_draw
    // will schedule its successor.
    app_mut.draw_callback = Some(Closure::wrap(Box::new({
        let app = Arc::clone(&app);
        move |timestamp: f64| {
            app.lock().unwrap().update_and_draw(timestamp);
        }
    }) as Box<dyn FnMut(f64)>));
    window
        .request_animation_frame(
            app_mut
                .draw_callback
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();
}

impl App {
    const FIXED_TIMESTEP: Seconds = Seconds(1.0 / 100.0);

    fn update_and_draw(&mut self, timestamp: f64) {
        self.time_accumulator
            .update_for_timestamp(Milliseconds(timestamp));

        let window = web_sys::window().unwrap();
        self.update_dimensions(&window);

        let input = self.sample_input(&window);
        while self.time_accumulator.try_consume(App::FIXED_TIMESTEP) {
            self.game.step(App::FIXED_TIMESTEP.seconds(), input.clone());
        }

        self.interpolate();
        self.draw(&window);
    }

    fn update_dimensions(&mut self, window: &Window) {
        let device_pixel_ratio = window.device_pixel_ratio();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let rect = body.get_bounding_client_rect();

        let width = (rect.width() * device_pixel_ratio).ceil();
        let height = (rect.height() * device_pixel_ratio).ceil();
        let int_width = width as u32;
        let int_height = height as u32;
        if self.last_dimensions != Some((int_width, int_height)) {
            self.canvas.set_width(int_width);
            self.canvas.set_height(int_height);
            self.last_dimensions = Some((int_width, int_height));

            let style = self.canvas.style();
            style.set_css_text(&format!(
                "width: {}px; height: {}px",
                width / device_pixel_ratio,
                height / device_pixel_ratio,
            ));
        }
    }

    fn sample_input(&self, window: &Window) -> Input {
        let gamepads = window.navigator().get_gamepads().unwrap();
        if let Ok(gamepad) = gamepads.get(0).dyn_into::<Gamepad>() {
            let axes = gamepad.axes();
            let buttons = gamepad.buttons();
            Input {
                move_: vec2(
                    axes.get(0).as_f64().unwrap() as f32,
                    axes.get(1).as_f64().unwrap() as f32,
                ),
                aim: vec2(
                    axes.get(2).as_f64().unwrap() as f32,
                    axes.get(3).as_f64().unwrap() as f32,
                ),
                fire: buttons.get(7).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
            }
        } else {
            Input {
                move_: zero(),
                aim: zero(),
                fire: false,
            }
        }
    }

    fn interpolate(&mut self) {
        // TODO: Run a Legion system to compute interpolated positions based on the current time
        // accumulator.
    }

    fn draw(&mut self, window: &Window) {
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;
        let ctx = self.ctx.clone();
        ctx.reset_transform().unwrap();
        ctx.set_fill_style(&JsValue::from_str("#000"));
        ctx.fill_rect(0.0, 0.0, w, h);
        ctx.translate(0.5 * w, 0.5 * h).unwrap();

        // Draw players.
        for (&Position(pos), _) in <(&Position, &Player)>::query().iter(self.game.world()) {
            ctx.begin_path();
            ctx.arc(
                pos.x as f64,
                pos.y as f64,
                20.0 as f64,
                0.0,
                std::f64::consts::TAU,
            )
            .unwrap();
            ctx.close_path();

            ctx.set_fill_style(&JsValue::from_str("#4f4"));
            ctx.fill();

            ctx.set_stroke_style(&JsValue::from_str("#282"));
            ctx.set_line_width(2.0);
            ctx.stroke();
        }

        // Draw hurtboxes.
        for (&Position(pos), hitbox, hitbox_state) in
            <(&Position, &Hurtbox, &HurtboxState)>::query().iter(self.game.world())
        {
            match &hitbox.shape {
                Shape::Circle(circle) => {
                    let color = if hitbox_state.hit_by_entities.is_empty() {
                        "rgba(64, 64, 255, 0.5)"
                    } else {
                        "rgba(255, 255, 255, 0.5)"
                    };

                    ctx.begin_path();
                    ctx.arc(
                        pos.x as f64,
                        pos.y as f64,
                        circle.radius as f64,
                        0.0,
                        std::f64::consts::TAU,
                    )
                    .unwrap();
                    ctx.close_path();

                    ctx.set_fill_style(&JsValue::from_str(color));
                    ctx.fill();
                }
            }
        }

        // Draw hitboxes.
        for (&Position(pos), hitbox, hitbox_state) in
            <(&Position, &Hitbox, &HitboxState)>::query().iter(self.game.world())
        {
            match &hitbox.shape {
                Shape::Circle(circle) => {
                    let color = if hitbox_state.hit_entities.is_empty() {
                        "rgba(255, 64, 64, 0.5)"
                    } else {
                        "rgba(255, 255, 255, 0.5)"
                    };

                    ctx.begin_path();
                    ctx.arc(
                        pos.x as f64,
                        pos.y as f64,
                        circle.radius as f64,
                        0.0,
                        std::f64::consts::TAU,
                    )
                    .unwrap();
                    ctx.close_path();

                    ctx.set_fill_style(&JsValue::from_str(color));
                    ctx.fill();
                }
            }
        }

        window
            .request_animation_frame(
                self.draw_callback
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }
}
