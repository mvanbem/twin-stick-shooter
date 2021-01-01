use cgmath::vec2;
use gui::in_game::RunningInGameMenu;
use gui::station::StationDockedMenu;
use model::ModelManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use twin_stick_shooter_core::game::Game;
use twin_stick_shooter_core::resource::{GuiOverride, Input, Subframe, Time};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, Document, Gamepad, GamepadButton, GamepadMappingType,
    HtmlCanvasElement, HtmlElement, HtmlInputElement, KeyboardEvent, TouchEvent, Window,
};

mod action;
mod draw;
mod gui;
mod model;
mod time_accumulator;

use gui::title::TitleMenu;
use gui::GuiState;
use time_accumulator::{Seconds, TimeAccumulator};

use crate::time_accumulator::Milliseconds;

pub struct App {
    game: Game,
    last_dimensions: Option<(u32, u32)>,
    time_accumulator: TimeAccumulator,

    keys: HashMap<String, bool>,
    key_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    touch_start_input: bool,
    active_touches: usize,
    touch_callback: Option<Closure<dyn FnMut(TouchEvent)>>,

    gui: GuiState,

    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    draw_callback: Option<Closure<dyn FnMut(f64)>>,
    model_manager: ModelManager,
}

#[wasm_bindgen]
pub fn launch() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Construct the underlying game model.
    let game = Game::new();

    // Look up some objects in the JS environment.
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let quickstart = &window.location().hash().unwrap() == "#quickstart";

    // Set up the main canvas element.
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_id("canvas");
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

        keys: "wasdijkl "
            .chars()
            .map(|c| (c.to_string(), false))
            .chain(
                (&["Enter", "Escape"])
                    .iter()
                    .copied()
                    .map(|s| (s.to_string(), false)),
            )
            .collect(),
        key_callback: None,
        touch_start_input: false,
        active_touches: 0,
        touch_callback: None,

        gui: GuiState::new(Box::new(TitleMenu)),

        canvas,
        ctx,
        draw_callback: None,
        model_manager: ModelManager::default(),
    }));
    let mut app_mut = app.lock().unwrap();

    if quickstart {
        app_mut.gui = GuiState::new(Box::new(RunningInGameMenu));
        action::create_game(&mut app_mut.game);
    }

    // Register for keyboard events.
    app_mut.key_callback = Some(Closure::wrap(Box::new({
        let app = Arc::clone(&app);
        move |e: KeyboardEvent| {
            let mut app_mut = app.lock().unwrap();
            let key = e.key();
            if app_mut.keys.contains_key(&key) {
                *app_mut.keys.get_mut(&key).unwrap() = match e.type_().as_str() {
                    "keydown" => true,
                    "keyup" => false,
                    _ => unreachable!(),
                };
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

    // Hook touch events on the canvas in order to provide a start/pause button on mobile touch. If
    // the touch opens a GUI menu over the touch point, it seems to respond immediately to the touch
    // on release. This amusing logic temporarily applies an `unstable` CSS class to the root GUI
    // element, which activates an override to disable pointer events while it's present. The
    // `touchend` handler calls `preventDefault()`, which also appears to be necessary to prevent
    // unwanted interaction with the newly shown elements.
    app_mut.touch_callback = Some(Closure::wrap(Box::new({
        let app = Arc::clone(&app);
        move |e: TouchEvent| {
            let mut app_mut = app.lock().unwrap();
            match &*e.type_() {
                "touchstart" => {
                    app_mut.touch_start_input = true;
                    app_mut.active_touches += 1;

                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let gui = document.get_element_by_id("gui").unwrap();
                    gui.set_class_name("gui unstable");
                }
                "touchend" => {
                    app_mut.active_touches = app_mut.active_touches.checked_sub(1).unwrap_or(0);
                    if app_mut.active_touches == 0 {
                        app_mut.touch_start_input = false;
                    }

                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let gui = document.get_element_by_id("gui").unwrap();
                    gui.set_class_name("gui");

                    e.prevent_default();
                }
                _ => unreachable!(),
            }
        }
    }) as Box<dyn FnMut(TouchEvent)>));
    app_mut
        .canvas
        .add_event_listener_with_callback(
            "touchstart",
            app_mut
                .touch_callback
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();
    app_mut
        .canvas
        .add_event_listener_with_callback(
            "touchend",
            app_mut
                .touch_callback
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

    fn get_key(&self, key: &str) -> bool {
        self.keys.get(key).copied().unwrap_or_default()
    }

    fn update_and_draw(&mut self, timestamp: f64) {
        let window = web_sys::window().unwrap();
        self.update_dimensions(&window);

        let document = window.document().unwrap();
        let debug = App::sample_debug_state(&document);

        let elapsed_seconds = self
            .time_accumulator
            .update_for_timestamp(Milliseconds(timestamp));
        let input = self.sample_input(&window);
        self.step(elapsed_seconds, &input);

        self.interpolate();
        draw::draw(
            &self.canvas,
            &self.ctx,
            &self.model_manager,
            &self.game,
            &input,
            &debug,
        );
        self.update_debug_ui(&document);

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
                    dpad_up: buttons.get(12).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
                    dpad_down: buttons.get(13).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
                    confirm: buttons.get(0).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
                    start: self.touch_start_input
                        || buttons.get(9).dyn_into::<GamepadButton>().unwrap().value() > 0.5,
                })
            })
            .next()
            .unwrap_or_else(|| Input {
                move_: vec2(
                    if self.get_key("d") { 1.0 } else { 0.0 }
                        + if self.get_key("a") { -1.0 } else { 0.0 },
                    if self.get_key("s") { 1.0 } else { 0.0 }
                        + if self.get_key("w") { -1.0 } else { 0.0 },
                ),
                aim: vec2(
                    if self.get_key("l") { 1.0 } else { 0.0 }
                        + if self.get_key("j") { -1.0 } else { 0.0 },
                    if self.get_key("k") { 1.0 } else { 0.0 }
                        + if self.get_key("i") { -1.0 } else { 0.0 },
                ),
                fire: self.get_key(" "),
                dpad_up: false,
                dpad_down: false,
                confirm: self.get_key("Enter"),
                start: self.touch_start_input || self.get_key("Escape"),
            })
    }

    fn step(&mut self, elapsed_seconds: Option<Seconds>, input: &Input) {
        // Step the GUI.
        if let Some(Seconds(elapsed_seconds)) = elapsed_seconds {
            self.gui
                .step(&Time { elapsed_seconds }, &input, &mut self.game);
        }

        // Step the game.
        if self.game.is_paused() {
            // Prevent the time accumulator from filling while paused. Otherwise, after unpause the
            // game will make several steps as if it needed to catch up.
            self.time_accumulator
                .try_consume(self.time_accumulator.accumulator());
        } else {
            while self.time_accumulator.try_consume(App::FIXED_TIMESTEP) {
                self.game.step(App::FIXED_TIMESTEP.seconds(), input.clone());
            }
        }

        // Apply queued GUI overrides from the step.
        for gui_override in self.game.gui_override_queue().drain() {
            // TODO: Is this silly? Why isn't this just a coalescing Option<GuiOverride>?
            self.gui.replace_with(match gui_override {
                GuiOverride::StationDocked => Box::new(StationDockedMenu),
            });
        }
    }

    fn interpolate(&mut self) {
        let subframe = Subframe(if self.game.is_paused() {
            1.0
        } else {
            self.time_accumulator.accumulator() / App::FIXED_TIMESTEP
        });
        self.game.interpolate(subframe);
    }

    fn sample_debug_state(document: &Document) -> DebugState {
        DebugState {
            draw_hitboxes: document
                .get_element_by_id("debug-draw-hitboxes")
                .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
                .map(|input| input.checked())
                .unwrap_or(false),
        }
    }

    fn update_debug_ui(&self, document: &Document) {
        let counters = self.game.collide_counters();

        if let Some(element) = document
            .get_element_by_id("debug-hitbox-counters")
            .and_then(|element| element.dyn_into::<HtmlElement>().ok())
        {
            element.set_inner_text(&format!("{:#?}", &*counters));
        }
    }
}

#[derive(Clone, Debug)]
pub struct DebugState {
    draw_hitboxes: bool,
}
