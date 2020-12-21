use crate::util::Timer;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Player {
    pub shoot_cooldown: Timer,
    pub inventory: Inventory,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerPlan {
    pub shoot: Option<Vec2>,
}

// TODO: `Inventory` is not a Legion component, so maybe it should go somewhere else?
#[derive(Clone, Debug)]
pub struct Inventory {}
