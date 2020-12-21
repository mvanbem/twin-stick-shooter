use cgmath::vec2;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use twin_stick_shooter_core::resource::{Input, Subframe};
use twin_stick_shooter_core::Game;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, Gamepad, GamepadButton, GamepadMappingType, HtmlCanvasElement,
    KeyboardEvent, Window,
};

mod draw;
mod time_accumulator;

use time_accumulator::{Seconds, TimeAccumulator};

use crate::time_accumulator::Milliseconds;

pub struct App {
    game: Game,
    last_dimensions: Option<(u32, u32)>,
    time_accumulator: TimeAccumulator,

    keys: HashMap<char, bool>,
    key_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,

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

    // Construct the App instance inside an Arc and a Mutex. This makes it easy to ensure it lives
    // long enough for all of the various asynchronous callbacks that it will register.
    let app = Arc::new(Mutex::new(App {
        game,
        last_dimensions: None,
        time_accumulator: TimeAccumulator::default(),

        keys: "wasdijkl ".chars().map(|c| (c, false)).collect(),
        key_callback: None,

        canvas,
        ctx,
        draw_callback: None,
    }));
    let mut app_mut = app.lock().unwrap();

    // Register for keyboard events.
    app_mut.key_callback = Some(Closure::wrap(Box::new({
        let app = Arc::clone(&app);
        move |e: KeyboardEvent| {
            let key = e.key();
            if key.len() == 1 {
                key.chars().next().map(|key| {
                    app.lock().unwrap().keys.entry(key).and_modify(|entry| {
                        *entry = match e.type_().as_str() {
                            "keydown" => true,
                            "keyup" => false,
                            _ => unreachable!(),
                        }
                    });
                });
            }
        }
    }) as Box<dyn FnMut(KeyboardEvent)>));
    window
        .add_event_listener_with_callback(
            "keydown",
            app_mut
                .key_callback
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();
    window
        .add_event_listener_with_callback(
            "keyup",
            app_mut
                .key_callback
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();

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

    fn get_key(&self, key: char) -> bool {
        self.keys.get(&key).copied().unwrap_or_default()
    }

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
        draw::draw(&self.canvas, &self.ctx, &self.game, &input);

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
        window
            .navigator()
            .get_gamepads()
            .unwrap()
            .iter()
            .flat_map(|gamepad| {
                let gamepad = gamepad.dyn_into::<Gamepad>().ok()?;
                if gamepad.mapping() != GamepadMappingType::Standard {
                    return None;
                }

                let axes = gamepad.axes();
                let buttons = gamepad.buttons();
                Some(Input {
                    move_: vec2(
                        axes.get(0).as_f64().unwrap() as f32,
                        axes.get(1).as_f64().unwrap() as f32,
                    ),
                    aim: vec2(
                        axes.get(2).as_f64().unwrap() as f32,
                        axes.get(3).as_f64().unwrap() as f32,
                    ),
                    fire: buttons.get(7).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
                })
            })
            .next()
            .unwrap_or_else(|| Input {
                move_: vec2(
                    if self.get_key('d') { 1.0 } else { 0.0 }
                        + if self.get_key('a') { -1.0 } else { 0.0 },
                    if self.get_key('s') { 1.0 } else { 0.0 }
                        + if self.get_key('w') { -1.0 } else { 0.0 },
                ),
                aim: vec2(
                    if self.get_key('l') { 1.0 } else { 0.0 }
                        + if self.get_key('j') { -1.0 } else { 0.0 },
                    if self.get_key('k') { 1.0 } else { 0.0 }
                        + if self.get_key('i') { -1.0 } else { 0.0 },
                ),
                fire: self.get_key(' '),
            })
    }

    fn interpolate(&mut self) {
        self.game.interpolate(Subframe(
            self.time_accumulator.accumulator() / App::FIXED_TIMESTEP,
        ));
    }
}
