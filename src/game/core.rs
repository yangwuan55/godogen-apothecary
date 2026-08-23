//! Core plugin: resources, state, plugins, scene setup, title flow.

use super::customers::CustomersPlugin;
use super::economy::EconomyPlugin;
use super::panels::PanelsPlugin;
use super::particles::ParticlesPlugin;
use super::resources::*;
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
            .add_plugins((
                CustomersPlugin,
                EconomyPlugin,
                PanelsPlugin,
                UiPlugin,
                ParticlesPlugin,
                VisualPlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(Update, title_start.run_if(in_state(GameScreen::Title)));
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

/// Enter begins the game from the title screen.
fn title_start(input: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameScreen>>) {
    if input.just_pressed(KeyCode::Enter) {
        next.set(GameScreen::Playing);
    }
}
