use twin_stick_shooter_core::Game;

use crate::action;
use crate::gui::in_game::RunningInGameMenu;
use crate::gui::{GuiResult, Menu};

use super::{Heading, HeadingStyle};

#[derive(Debug)]
pub struct TitleMenu;

impl Menu for TitleMenu {
    fn heading(&self) -> Option<Heading<'_>> {
        Some(Heading {
            style: HeadingStyle::Title,
            text: "this is a title screen",
        })
    }

    fn items(&self) -> &[&str] {
        &["New Game"]
    }

    fn on_start_pressed(&mut self, _game: &mut Game) -> GuiResult {
        GuiResult::Ok
    }

    fn invoke_item(&mut self, _index: usize, game: &mut Game) -> GuiResult {
        action::create_game(game);
        GuiResult::ReplaceMenu(Box::new(RunningInGameMenu))
    }
}
