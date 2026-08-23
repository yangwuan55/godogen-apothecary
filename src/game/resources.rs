//! Global game resources.

use super::data::{NUM_MATERIALS, UpgradeId};
use bevy::prelude::*;

/// Current panel / game screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameScreen {
    #[default]
    Title,
    Playing,
    Market,
    Upgrades,
    RecipeBook,
    DayReport,
    GameOver,
    Victory,
}

#[derive(Resource)]
pub struct Economy {
    pub gold: u32,
    pub reputation: u32,
    pub rep_level: u8, // 1..=10
    pub day: u32,
    pub day_elapsed: f32, // seconds into the day
    pub day_length: f32,  // seconds per day
    pub day_income: u32,
    pub rent: u32,
    pub served: u32,
    pub lost: u32,
    pub perfect_count: u32,
    /// Quality distribution of today's deliveries: [Poor, Normal, Good, Perfect].
    pub day_quality: [u32; 4],
    /// Total restock purchases this run (used by the tutorial).
    pub purchases: u32,
}

impl Economy {
    pub fn new() -> Self {
        Self {
            gold: 60,
            reputation: 0,
            rep_level: 1,
            day: 1,
            day_elapsed: 0.0,
            day_length: 75.0,
            day_income: 0,
            rent: 15,
            served: 0,
            lost: 0,
            perfect_count: 0,
            day_quality: [0; 4],
            purchases: 0,
        }
    }

    /// Reputation needed for the next level, if any.
    pub fn next_threshold(&self) -> Option<u32> {
        super::data::REP_THRESHOLDS
            .get(self.rep_level as usize)
            .copied()
    }

    /// Advance reputation level if thresholds crossed. Returns true if leveled up.
    pub fn check_level_up(&mut self) -> bool {
        while let Some(next) = self.next_threshold() {
            if self.reputation >= next && self.rep_level < 10 {
                self.rep_level += 1;
                continue;
            }
            break;
        }
        self.rep_level >= 10
    }
}

#[derive(Resource)]
pub struct Inventory {
    pub counts: [u32; NUM_MATERIALS],
    pub restock_qty: u32, // units bought per restock press
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            counts: [6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0],
            restock_qty: 1,
        }
    }
}

#[derive(Resource)]
pub struct UpgradesState {
    pub levels: [u8; 4], // indexed by UpgradeId
}

impl UpgradesState {
    pub fn new() -> Self {
        Self {
            levels: [0, 0, 0, 0],
        }
    }
    pub fn level(&self, id: UpgradeId) -> u8 {
        self.levels[id as usize]
    }
}

/// A customer waiting or interacting in the shop.
#[derive(Component)]
pub struct Customer {
    pub kind_idx: usize,
    pub recipe_idx: usize,
    pub patience: f32, // remaining seconds
    pub patience_max: f32,
    pub budget: u32,
    pub state: CustomerState,
    pub target_pos: Vec2,
    pub home_pos: Vec2,
    pub wobble: f32,
    pub queue_slot: u32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CustomerState {
    Walking,
    Waiting,
    Served,
    Leaving,
}

#[derive(Resource, Default)]
pub struct CustomerQueue {
    pub next_id: u32,
    pub spawn_timer: f32,
    pub spawned_today: u32,
    pub slots: Vec<Option<Customer>>, // fixed-size roster, front = index 0
}

impl CustomerQueue {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            spawn_timer: 2.0,
            spawned_today: 0,
            slots: Vec::new(),
        }
    }
}

/// Active brewing session.
#[derive(Resource)]
pub struct Brewing {
    pub active: bool,
    pub recipe_idx: usize,
    pub progress: f32,        // 0..100
    pub temp: f32,            // 0..100
    pub stir_hits: Vec<bool>, // per stir point, true if done well
    pub stir_window: f32,
    pub in_window_time: f32, // seconds spent in ideal window
    pub raw_time: f32,       // total brewing seconds
    pub burnt: bool,
    pub cold_time: f32,
    pub burn_time: f32,
    pub quality_score: f32,
    pub auto_serve: bool, // autopilot marker for capture
}

impl Brewing {
    pub fn new() -> Self {
        Self {
            active: false,
            recipe_idx: 0,
            progress: 0.0,
            temp: 50.0,
            stir_hits: Vec::new(),
            stir_window: 0.0,
            in_window_time: 0.0,
            raw_time: 0.0,
            burnt: false,
            cold_time: 0.0,
            burn_time: 0.0,
            quality_score: 0.0,
            auto_serve: false,
        }
    }
}

/// Flying potion / coin particle events.
#[derive(Message, Clone)]
pub struct FxEvent {
    pub kind: FxKind,
    pub pos: Vec2,
    pub text: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum FxKind {
    #[default]
    Bubble,
    Steam,
    Sparkle,
    GoldText,
    QualityText,
    LevelUp,
    Fizzle,
}

/// Global pause flag. When true, gameplay systems freeze; the pause overlay UI
/// stays interactive so the player can resume / return to title / quit.
#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// Capture-only cheat: force the current day to end immediately.
#[derive(Resource, Default)]
pub struct ForceDayEnd(pub bool);

/// Current temperature-control intent, set by both keyboard (hold ↑/↓) and
/// mouse (hold the +/− buttons). `brewing::update_brewing` reads this instead
/// of polling the keyboard directly, so both input channels behave identically.
#[derive(Resource, Default)]
pub struct TempControl {
    pub up: bool,
    pub down: bool,
}

/// Master switch for the in-game tutorial hints. The offscreen capture binary
/// sets this to `false` so the proof video is deterministic.
#[derive(Resource)]
pub struct TutorialSettings {
    pub enabled: bool,
}
impl Default for TutorialSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}
