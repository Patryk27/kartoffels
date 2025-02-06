use glam::IVec2;

/// Action to apply on the world after [`AliveBot::tick()`] finishes.
///
/// This exists mostly to avoid borrowck issues - e.g. moving a robot requires
/// unique access to the entire `world.bots`, which conflicts with `.tick()`
/// that needs unique access to the bot itself.
// TODO ^ this is refactorable
#[derive(Debug)]
pub enum BotAction {
    ArmDrop { at: IVec2, idx: u8 },
    ArmPick { at: IVec2 },
    ArmStab { at: IVec2 },
    MotorMove { at: IVec2 },
}
