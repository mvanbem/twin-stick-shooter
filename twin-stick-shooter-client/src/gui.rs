use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use twin_stick_shooter_core::resource::{Input, Time};
use twin_stick_shooter_core::util::Timer;
use twin_stick_shooter_core::Game;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub mod in_game;
pub mod title;

const BUTTON_COOLDOWN: f32 = 0.25;

pub struct GuiState {
    inner: Arc<Mutex<InnerGuiState>>,
}

struct InnerGuiState {
    menu: Box<dyn Menu>,
    items: Vec<HtmlElement>,
    selection: Option<usize>,

    move_up_repeat: RepeatingButton,
    move_down_repeat: RepeatingButton,
    dpad_up_repeat: RepeatingButton,
    dpad_down_repeat: RepeatingButton,
    start_one_shot: OneShotButton,
    confirm_one_shot: OneShotButton,

    event_queue: VecDeque<Event>,
    mousemove_callbacks: Vec<Closure<dyn FnMut()>>,
    mouseout_callback: Option<Closure<dyn FnMut()>>,
    click_callbacks: Vec<Closure<dyn FnMut()>>,
}

impl GuiState {
    pub fn new(menu: Box<dyn Menu>) -> GuiState {
        let gui = GuiState {
            inner: Arc::new(Mutex::new(InnerGuiState {
                menu,
                items: vec![],
                selection: None,

                move_up_repeat: RepeatingButton::new(),
                move_down_repeat: RepeatingButton::new(),
                dpad_up_repeat: RepeatingButton::new(),
                dpad_down_repeat: RepeatingButton::new(),
                start_one_shot: OneShotButton::new(),
                confirm_one_shot: OneShotButton::new(),

                event_queue: VecDeque::new(),
                mousemove_callbacks: vec![],
                mouseout_callback: None,
                click_callbacks: vec![],
            })),
        };
        gui.inner.lock().unwrap().actuate(&gui.inner);
        gui
    }

    pub fn step(&self, time: &Time, input: &Input, game: &mut Game) {
        let mut inner = self.inner.lock().unwrap();

        // Pull out a mutable copy for zero or more updates, then apply the change (if any) later.
        let mut tmp_selection = inner.selection;

        // First, process any queued events that arrived via JS callbacks.
        let mut event_queue = std::mem::replace(&mut inner.event_queue, VecDeque::new());
        for event in event_queue.drain(..) {
            match event {
                Event::Select(selection) => tmp_selection = selection,
                Event::Confirm(index) => {
                    let gui_result = inner.menu.invoke_item(index, game);
                    if inner.handle_gui_result(&self.inner, gui_result) {
                        return;
                    }
                }
            }
        }

        for increment in GuiState::step_and_interpret_increments(&mut *inner, time, input).drain(..)
        {
            tmp_selection = match increment {
                Increment::Backward => tmp_selection
                    .and_then(|index| index.checked_sub(1))
                    .or_else(|| inner.items.len().checked_sub(1)),
                Increment::Forward => tmp_selection
                    .and_then(|index| match index + 1 {
                        x if x < inner.items.len() => Some(x),
                        _ => None,
                    })
                    .or_else(|| if inner.items.len() > 0 { Some(0) } else { None }),
            };
        }

        // Apply any changes in the selection.
        inner.set_selection(tmp_selection);

        if inner.start_one_shot.step_and_is_firing(input.start) {
            let gui_result = inner.menu.on_start_pressed(game);
            if inner.handle_gui_result(&self.inner, gui_result) {
                return;
            }
        }

        if inner.confirm_one_shot.step_and_is_firing(input.confirm) {
            if let Some(index) = inner.selection {
                let gui_result = inner.menu.invoke_item(index, game);
                if inner.handle_gui_result(&self.inner, gui_result) {
                    return;
                }
            }
        }
    }

    fn step_and_interpret_increments(
        inner: &mut InnerGuiState,
        time: &Time,
        input: &Input,
    ) -> Vec<Increment> {
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

        increments
    }
}

impl InnerGuiState {
    fn actuate(&mut self, inner: &Arc<Mutex<InnerGuiState>>) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let gui = document.get_element_by_id("gui").unwrap();

        // Remove all item event listeners. In particular, the mousemove/mouseout events may fire
        // while the items are being removed, which could cause inappropriate state changes.
        for ((item, mousemove_callback), click_callback) in self
            .items
            .iter()
            .zip(&self.mousemove_callbacks)
            .zip(&self.click_callbacks)
        {
            item.remove_event_listener_with_callback(
                "mousemove",
                mousemove_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
            item.remove_event_listener_with_callback(
                "mouseout",
                self.mouseout_callback
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
            item.remove_event_listener_with_callback(
                "click",
                click_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
        }

        self.items.clear();
        while let Some(child) = gui.last_child() {
            gui.remove_child(&child).unwrap();
        }
        self.selection = None;
        self.event_queue.clear();
        self.mousemove_callbacks.clear();
        self.mouseout_callback = None;
        self.click_callbacks.clear();

        if let Some(heading) = self.menu.heading() {
            gui.append_child(
                {
                    let element: HtmlElement =
                        document.create_element("div").unwrap().dyn_into().unwrap();
                    element.set_class_name(match heading.style {
                        HeadingStyle::Regular => "heading",
                        HeadingStyle::Title => "heading title",
                    });
                    element.set_text_content(Some(heading.text));
                    element
                }
                .as_ref(),
            )
            .unwrap();
        }

        gui.append_child(
            {
                let element: HtmlElement =
                    document.create_element("div").unwrap().dyn_into().unwrap();
                element.set_class_name("expand");
                element
            }
            .as_ref(),
        )
        .unwrap();

        for (index, item) in self.menu.items().iter().copied().enumerate() {
            gui.append_child(
                {
                    let element: HtmlElement =
                        document.create_element("div").unwrap().dyn_into().unwrap();
                    element.set_class_name(if index == 0 {
                        "button selected"
                    } else {
                        "button"
                    });
                    element.set_text_content(Some(item));
                    self.items.push(element.clone());
                    element
                }
                .as_ref(),
            )
            .unwrap();
        }
        if self.items.len() > 0 {
            self.selection = Some(0)
        }

        gui.append_child(
            {
                let element: HtmlElement =
                    document.create_element("div").unwrap().dyn_into().unwrap();
                element.set_class_name("expand");
                element
            }
            .as_ref(),
        )
        .unwrap();

        let mouseout_callback = Closure::wrap(Box::new({
            let inner = Arc::clone(&inner);
            move || {
                inner
                    .lock()
                    .unwrap()
                    .event_queue
                    .push_back(Event::Select(None));
            }
        }) as Box<dyn FnMut()>);
        for (index, item) in self.items.iter().enumerate() {
            let mousemove_callback = Closure::wrap(Box::new({
                let inner = Arc::clone(&inner);
                move || {
                    inner
                        .lock()
                        .unwrap()
                        .event_queue
                        .push_back(Event::Select(Some(index)));
                }
            }) as Box<dyn FnMut()>);
            item.add_event_listener_with_callback(
                "mousemove",
                mousemove_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
            self.mousemove_callbacks.push(mousemove_callback);

            item.add_event_listener_with_callback(
                "mouseout",
                mouseout_callback.as_ref().unchecked_ref(),
            )
            .unwrap();

            let click_callback = Closure::wrap(Box::new({
                let inner = Arc::clone(&inner);
                move || {
                    inner
                        .lock()
                        .unwrap()
                        .event_queue
                        .push_back(Event::Confirm(index));
                }
            }) as Box<dyn FnMut()>);
            item.add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())
                .unwrap();
            self.click_callbacks.push(click_callback);
        }
        self.mouseout_callback = Some(mouseout_callback);
    }

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

    #[must_use]
    fn handle_gui_result(
        &mut self,
        inner: &Arc<Mutex<InnerGuiState>>,
        gui_result: GuiResult,
    ) -> bool {
        match gui_result {
            GuiResult::Ok => false,
            GuiResult::ReplaceMenu(menu) => {
                self.menu = menu;
                self.actuate(inner);
                true
            }
        }
    }
}

enum Event {
    Select(Option<usize>),
    Confirm(usize),
}

#[derive(Clone, Debug)]
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
                    self.timer.reset(BUTTON_COOLDOWN);
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

#[derive(Clone, Debug)]
struct OneShotButton {
    prev: bool,
}

impl OneShotButton {
    fn new() -> OneShotButton {
        // Assume the button was held in order to avoid a held button activating a newly appearing
        // option on the first step.
        OneShotButton { prev: true }
    }

    fn step_and_is_firing(&mut self, input: bool) -> bool {
        let prev = self.prev;
        self.prev = input;
        input && !prev
    }
}

#[derive(Clone, Copy, Debug)]
enum Increment {
    Backward,
    Forward,
}

pub struct Heading<'a> {
    style: HeadingStyle,
    text: &'a str,
}

pub enum HeadingStyle {
    Regular,
    Title,
}

pub trait Menu: Debug {
    fn heading(&self) -> Option<Heading<'_>>;
    fn items(&self) -> &[&str];
    fn on_start_pressed(&mut self, game: &mut Game) -> GuiResult;
    fn invoke_item(&mut self, index: usize, game: &mut Game) -> GuiResult;
}

#[derive(Debug)]
#[must_use]
pub enum GuiResult {
    Ok,
    ReplaceMenu(Box<dyn Menu>),
    // TODO: deny option with animation and sound
}
