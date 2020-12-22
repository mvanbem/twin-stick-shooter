use std::sync::{Arc, Mutex};
use twin_stick_shooter_core::resource::{Input, Time};
use twin_stick_shooter_core::util::Timer;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub struct MenuState {
    inner: Arc<Mutex<InnerMenuState>>,
}

struct InnerMenuState {
    menu: Box<dyn Menu>,
    items: Vec<HtmlElement>,
    selection: Option<usize>,

    move_up_repeat: RepeatingButton,
    move_down_repeat: RepeatingButton,
    dpad_up_repeat: RepeatingButton,
    dpad_down_repeat: RepeatingButton,

    mousemove_callbacks: Vec<Closure<dyn FnMut()>>,
    mouseout_callback: Option<Closure<dyn FnMut()>>,
}

impl MenuState {
    pub fn new_main_menu() -> MenuState {
        MenuState::new(Box::new(MainMenu))
    }

    pub fn new(menu: Box<dyn Menu>) -> MenuState {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let gui = document.get_element_by_id("gui").unwrap();

        while let Some(child) = gui.last_child() {
            gui.remove_child(&child).unwrap();
        }

        gui.append_child(
            {
                let title: HtmlElement =
                    document.create_element("div").unwrap().dyn_into().unwrap();
                title.set_class_name("title");
                title.set_text_content(Some(menu.title()));
                title
            }
            .as_ref(),
        )
        .unwrap();
        gui.append_child(
            {
                let expand: HtmlElement =
                    document.create_element("div").unwrap().dyn_into().unwrap();
                expand.set_class_name("expand");
                expand
            }
            .as_ref(),
        )
        .unwrap();

        let mut items = vec![];
        for (index, item) in menu.items().iter().copied().enumerate() {
            gui.append_child(
                {
                    let button: HtmlElement =
                        document.create_element("div").unwrap().dyn_into().unwrap();
                    button.set_class_name(if index == 0 {
                        "button selected"
                    } else {
                        "button"
                    });
                    button.set_text_content(Some(item));
                    items.push(button.clone());
                    button
                }
                .as_ref(),
            )
            .unwrap();
        }
        assert!(items.len() > 0);

        gui.append_child(
            {
                let expand: HtmlElement =
                    document.create_element("div").unwrap().dyn_into().unwrap();
                expand.set_class_name("expand");
                expand
            }
            .as_ref(),
        )
        .unwrap();

        let inner = Arc::new(Mutex::new(InnerMenuState {
            menu,
            items,
            selection: Some(0),

            move_up_repeat: RepeatingButton::default(),
            move_down_repeat: RepeatingButton::default(),
            dpad_up_repeat: RepeatingButton::default(),
            dpad_down_repeat: RepeatingButton::default(),

            mousemove_callbacks: vec![],
            mouseout_callback: None,
        }));
        let mut inner_mut = inner.lock().unwrap();

        let mut mousemove_callbacks = vec![];
        let mouseout_callback = Closure::wrap(Box::new({
            let inner = Arc::clone(&inner);
            move || {
                inner.lock().unwrap().set_selection(None);
            }
        }) as Box<dyn FnMut()>);
        for (index, item) in inner_mut.items.iter().enumerate() {
            let callback = Closure::wrap(Box::new({
                let inner = Arc::clone(&inner);
                move || {
                    inner.lock().unwrap().set_selection(Some(index));
                }
            }) as Box<dyn FnMut()>);
            item.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref())
                .unwrap();
            item.add_event_listener_with_callback(
                "mouseout",
                mouseout_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
            mousemove_callbacks.push(callback);
        }
        inner_mut.mousemove_callbacks = mousemove_callbacks;
        inner_mut.mouseout_callback = Some(mouseout_callback);

        drop(inner_mut);
        MenuState { inner }
    }

    pub fn step(&mut self, time: &Time, input: &Input) {
        let mut inner = self.inner.lock().unwrap();

        let mut increments = vec![];

        if inner.move_up_repeat.step_and_is_firing(
            time,
            if input.move_.y < -0.7 {
                Some(true)
            } else if input.move_.y < -0.5 {
                None
            } else {
                Some(false)
            },
        ) {
            increments.push(Increment::Backward);
        }
        if inner.move_down_repeat.step_and_is_firing(
            time,
            if input.move_.y > 0.7 {
                Some(true)
            } else if input.move_.y > 0.5 {
                None
            } else {
                Some(false)
            },
        ) {
            increments.push(Increment::Forward);
        }
        if inner
            .dpad_up_repeat
            .step_and_is_firing(time, Some(input.dpad_up))
        {
            increments.push(Increment::Backward);
        }
        if inner
            .dpad_down_repeat
            .step_and_is_firing(time, Some(input.dpad_down))
        {
            increments.push(Increment::Forward);
        }

        let mut selection = inner.selection;
        for increment in increments.drain(..) {
            selection = Some(match increment {
                Increment::Backward => selection
                    .and_then(|index| index.checked_sub(1))
                    .unwrap_or_else(|| inner.items.len() - 1),
                Increment::Forward => selection
                    .and_then(|index| match index + 1 {
                        x if x < inner.items.len() => Some(x),
                        _ => None,
                    })
                    .unwrap_or(0),
            });
        }
        inner.set_selection(selection);
    }
}

impl InnerMenuState {
    fn set_selection(&mut self, selection: Option<usize>) {
        if selection != self.selection {
            if let Some(index) = self.selection {
                self.items[index].set_class_name("button");
            }
            self.selection = selection;
            if let Some(index) = self.selection {
                self.items[index].set_class_name("button selected");
            }
        }
    }
}

#[derive(Clone)]
struct RepeatingButton {
    timer: Timer,
}

impl RepeatingButton {
    fn new() -> RepeatingButton {
        RepeatingButton {
            timer: Timer::elapsed(),
        }
    }
    fn step_and_is_firing(&mut self, time: &Time, input: Option<bool>) -> bool {
        self.timer.step(time);
        match input {
            // Fire only if enough time has passed since the last firing.
            Some(true) => {
                if self.timer.is_elapsed() {
                    self.timer.reset(0.25);
                    true
                } else {
                    false
                }
            }
            // End the timer early if the input is released. This ensures manual repeats are not
            // delayed.
            Some(false) => {
                self.timer.elapse_now();
                false
            }
            // Never fire on indeterminate input.
            None => false,
        }
    }
}

impl Default for RepeatingButton {
    fn default() -> Self {
        RepeatingButton::new()
    }
}

#[derive(Clone, Copy, Debug)]
enum Increment {
    Backward,
    Forward,
}

pub trait Menu {
    fn title(&self) -> &str;
    fn items(&self) -> &[&str];
}

pub struct MainMenu;

impl Menu for MainMenu {
    fn title(&self) -> &str {
        "this is a title screen"
    }
    fn items(&self) -> &[&str] {
        &["Online Multiplayer", "Single Player", "Third Option"]
    }
}