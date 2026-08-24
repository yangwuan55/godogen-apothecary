//! Core plugin: resources, state, plugins, scene setup, title flow.

use super::actions::{InputPlugin, InputSet, UiAction};
use super::audio::GameAudioPlugin;
use super::customers::CustomersPlugin;
use super::economy::EconomyPlugin;
use super::panels::PanelsPlugin;
use super::particles::ParticlesPlugin;
use super::resources::*;
use super::tutorial::TutorialPlugin;
use super::ui::UiPlugin;
use super::visual::VisualPlugin;
use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScreen>()
            .insert_resource(ClearColor(Color::srgb(0.10, 0.09, 0.12)))
            .insert_resource(Economy::new())
            .insert_resource(Inventory::new())
            .insert_resource(UpgradesState::new())
            .insert_resource(Brewing::new())
            .insert_resource(CustomerQueue::new())
            .insert_resource(WindowCamera(true))
            .insert_resource(Paused::default())
            .insert_resource(TempControl::default())
            .insert_resource(TutorialSettings::default())
            .init_resource::<ForceDayEnd>()
            .add_plugins((
                InputPlugin,
                CustomersPlugin,
                EconomyPlugin,
                PanelsPlugin,
                UiPlugin,
                ParticlesPlugin,
                VisualPlugin,
                TutorialPlugin,
                GameAudioPlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                title_start
                    .after(InputSet)
                    .run_if(in_state(GameScreen::Title)),
            );
    }
}

/// Whether the normal window camera should be spawned. The offscreen capture
/// binary sets this to `false` and provides its own image-target camera, so the
/// window camera (which would target a non-existent primary window) never
/// exists and can't interfere with the capture render target.
#[derive(Resource)]
pub struct WindowCamera(pub bool);

fn setup(mut commands: Commands, window_camera: Res<WindowCamera>) {
    if !window_camera.0 {
        return;
    }
    commands.spawn(Camera2d);
}

/// Starting the game from the title screen (button click or Enter key).
fn title_start(mut actions: MessageReader<UiAction>, mut next: ResMut<NextState<GameScreen>>) {
    if actions.read().any(|a| *a == UiAction::StartGame) {
        next.set(GameScreen::Playing);
    }
}
