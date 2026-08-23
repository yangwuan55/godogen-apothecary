//! Day cycle, settlement, and game-over/victory checks.

use super::resources::{Customer, CustomerQueue, Economy, GameScreen};
use bevy::prelude::*;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, day_clock.run_if(in_state(GameScreen::Playing)));
        app.add_systems(
            Update,
            day_report_continue.run_if(in_state(GameScreen::DayReport)),
        );
        app.add_systems(
            Update,
            end_screen_continue
                .run_if(in_state(GameScreen::GameOver).or_else(in_state(GameScreen::Victory))),
        );
    }
}

/// Advance the in-day clock; at the end settle rent and show the day report.
fn day_clock(mut next: ResMut<NextState<GameScreen>>, mut econ: ResMut<Economy>, time: Res<Time>) {
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

/// On the report screen Enter/Space starts the next day, or victory / game over.
fn day_report_continue(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameScreen>>,
    mut econ: ResMut<Economy>,
    customers: Query<Entity, With<Customer>>,
    mut queue: ResMut<CustomerQueue>,
) {
    if !(input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space)) {
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
    econ.rent += 1;
    next.set(GameScreen::Playing);
}

fn end_screen_continue(input: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameScreen>>) {
    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
        next.set(GameScreen::Title);
    }
}
