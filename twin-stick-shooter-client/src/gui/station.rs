use twin_stick_shooter_core::game::Game;

use crate::action;
use crate::gui::{GuiResult, Heading, HeadingStyle, Menu};

use super::in_game::RunningInGameMenu;

#[derive(Debug)]
pub struct StationDockedMenu;

impl Menu for StationDockedMenu {
    fn heading(&self) -> Option<Heading<'_>> {
        Some(Heading {
            style: HeadingStyle::Regular,
            text: "Docked at Station",
        })
    }

    fn items(&self) -> &[&str] {
        &["Launch"]
    }

    fn on_start_pressed(&mut self, _game: &mut Game) -> GuiResult {
        GuiResult::Ok
    }

    fn invoke_item(&mut self, _index: usize, game: &mut Game) -> GuiResult {
        action::launch_from_station(game);
        GuiResult::ReplaceMenu(Box::new(RunningInGameMenu))
    }
}
