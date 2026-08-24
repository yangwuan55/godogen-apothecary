//! Day cycle, settlement, and game-over/victory checks.

use super::actions::{InputSet, UiAction};
use super::resources::{Customer, CustomerQueue, Economy, ForceDayEnd, GameScreen, Paused};
use bevy::prelude::*;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            day_clock
                .run_if(in_state(GameScreen::Playing))
                .run_if(not_paused),
        );
        app.add_systems(
            Update,
            day_report_continue
                .after(InputSet)
                .run_if(in_state(GameScreen::DayReport)),
        );
    }
}

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

/// Advance the in-day clock; at the end settle rent and show the day report.
fn day_clock(
    mut next: ResMut<NextState<GameScreen>>,
    mut econ: ResMut<Economy>,
    time: Res<Time>,
    force: Res<ForceDayEnd>,
) {
    if force.0 {
        econ.day_elapsed = econ.day_length;
    }
    econ.day_elapsed += time.delta_secs();
    if econ.day_elapsed >= econ.day_length {
        if econ.gold >= econ.rent {
            econ.gold -= econ.rent;
        } else {
            econ.gold = 0;
            next.set(GameScreen::GameOver);
            return;
        }
        econ.check_level_up();
        next.set(GameScreen::DayReport);
    }
}

/// On the report screen, 继续营业 starts the next day (or victory).
fn day_report_continue(
    mut commands: Commands,
    mut actions: MessageReader<UiAction>,
    mut next: ResMut<NextState<GameScreen>>,
    mut econ: ResMut<Economy>,
    customers: Query<Entity, With<Customer>>,
    mut queue: ResMut<CustomerQueue>,
) {
    let mut continue_day = false;
    for a in actions.read() {
        if *a == UiAction::Continue {
            continue_day = true;
        }
    }
    if !continue_day {
        return;
    }
    if econ.rep_level >= 10 {
        next.set(GameScreen::Victory);
        return;
    }
    for e in &customers {
        commands.entity(e).despawn();
    }
    queue.spawned_today = 0;
    queue.spawn_timer = 1.5;
    econ.day += 1;
    econ.day_elapsed = 0.0;
    econ.day_income = 0;
    econ.day_quality = [0; 4];
    econ.rent = rent_for_level(econ.rep_level);
    next.set(GameScreen::Playing);
}

/// Daily rent scales with reputation so the day-end bill stays a real decision.
/// `Lv1 -> 15` keeps the day-1 balance identical to the legacy curve.
pub fn rent_for_level(rep_level: u8) -> u32 {
    7 + 8 * rep_level as u32
}
