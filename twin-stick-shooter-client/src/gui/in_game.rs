use twin_stick_shooter_core::Game;

use crate::gui::title::TitleMenu;
use crate::gui::{GuiResult, Heading, HeadingStyle, Menu};

#[derive(Debug)]
pub struct RunningInGameMenu;

impl Menu for RunningInGameMenu {
    fn heading(&self) -> Option<Heading<'_>> {
        None
    }

    fn items(&self) -> &[&str] {
        &[]
    }

    fn on_start_pressed(&mut self, game: &mut Game) -> GuiResult {
        game.set_is_paused(true);
        GuiResult::ReplaceMenu(Box::new(PausedInGameMenu))
    }

    fn invoke_item(&mut self, _index: usize, _game: &mut Game) -> GuiResult {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct PausedInGameMenu;

impl Menu for PausedInGameMenu {
    fn heading(&self) -> Option<Heading<'_>> {
        Some(Heading {
            style: HeadingStyle::Regular,
            text: "Paused",
        })
    }

    fn items(&self) -> &[&str] {
        &["Continue", "Quit"]
    }

    fn on_start_pressed(&mut self, game: &mut Game) -> GuiResult {
        unpause_game(game)
    }

    fn invoke_item(&mut self, index: usize, game: &mut Game) -> GuiResult {
        match index {
            0 => unpause_game(game),
            1 => GuiResult::ReplaceMenu(Box::new(QuitGameMenu)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct QuitGameMenu;

impl Menu for QuitGameMenu {
    fn heading(&self) -> Option<Heading<'_>> {
        Some(Heading {
            style: HeadingStyle::Regular,
            text: "Quit Game?",
        })
    }

    fn items(&self) -> &[&str] {
        &["No", "Yes"]
    }

    fn on_start_pressed(&mut self, game: &mut Game) -> GuiResult {
        unpause_game(game)
    }

    fn invoke_item(&mut self, index: usize, game: &mut Game) -> GuiResult {
        match index {
            0 => GuiResult::ReplaceMenu(Box::new(PausedInGameMenu)),
            1 => {
                game.reset();
                GuiResult::ReplaceMenu(Box::new(TitleMenu))
            }
            _ => unreachable!(),
        }
    }
}

fn unpause_game(game: &mut Game) -> GuiResult {
    game.set_is_paused(false);
    GuiResult::ReplaceMenu(Box::new(RunningInGameMenu))
}
