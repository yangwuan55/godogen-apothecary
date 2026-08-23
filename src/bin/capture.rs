//! Offscreen proof-video capture with an autopilot that plays the game.
//!
//! Renders the real game to an offscreen `RenderTarget::Image` at a fixed 30 fps
//! (deterministic via `TimeUpdateStrategy::ManualDuration`), records frames to
//! `screenshots/result/frameNNNNN.png`, and encodes with ffmpeg afterwards.
//! The autopilot drives real input in `PreUpdate` *after* Bevy's input system
//! clears the per-frame flags, so `just_pressed` semantics match a human
//! player (pressing earlier in `First` gets wiped by `ButtonInput::clear()`).

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Capturing, Screenshot, save_to_disk};
use bevy::time::TimeUpdateStrategy;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use godogen_apothecary::build_app;
use godogen_apothecary::game::data::RECIPES;
use godogen_apothecary::game::resources::{Brewing, Customer, CustomerState, GameScreen};
use std::time::Duration;

const W: u32 = 1280;
const H: u32 = 720;
const FRAMES: u32 = 480; // 16 s @ 30 fps

#[derive(Resource)]
struct CaptureTarget {
    handle: Handle<Image>,
    frame: u32,
}

fn main() {
    let mut app = build_app(|plugins| {
        plugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
    });
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        1.0 / 30.0,
    )))
    .insert_resource(godogen_apothecary::game::core::WindowCamera(false))
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 30.0,
    )))
    .add_systems(Startup, capture_setup)
    .add_systems(PreUpdate, autopilot.after(bevy::input::InputSystems))
    .add_systems(Update, capture_frame)
    .run();
}

fn capture_setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut img = Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None);
    img.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let handle = images.add(img);
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        RenderTarget::from(handle.clone()),
        Camera::default(),
    ));
    commands.insert_resource(CaptureTarget { handle, frame: 0 });
    std::fs::create_dir_all("screenshots/result").ok();
}

/// Record one frame at a time; wait for the previous screenshot to finish.
fn capture_frame(
    mut commands: Commands,
    mut target: ResMut<CaptureTarget>,
    capt: Query<(), With<Capturing>>,
) {
    if capt.iter().next().is_some() {
        return;
    }
    let f = target.frame;
    if f >= FRAMES {
        std::process::exit(0);
    }
    let path = format!("screenshots/result/frame{:05}.png", f);
    commands
        .spawn(Screenshot::image(target.handle.clone()))
        .observe(save_to_disk(path));
    target.frame += 1;
}

/// Play the game automatically: start from the title, accept orders, keep
/// temperature in the ideal window, let auto-stir do its job, and cycle
/// through the three panels to showcase them.
fn autopilot(
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut brewing: ResMut<Brewing>,
    state: Res<State<GameScreen>>,
    customers: Query<&Customer>,
    target: Res<CaptureTarget>,
) {
    // Held keys: release every frame, then re-press only when needed.
    input.release(KeyCode::ArrowUp);
    input.release(KeyCode::ArrowDown);
    // Momentary keys: release so each press fires a fresh `just_pressed`.
    input.release(KeyCode::Tab);
    input.release(KeyCode::Enter);
    input.release(KeyCode::Space);

    let f = target.frame;

    match *state.get() {
        GameScreen::Title => {
            // Let the title be visible for the first second, then start.
            if f == 30 {
                input.press(KeyCode::Enter);
            }
        }
        GameScreen::Playing => {
            // Open the Market panel at 4s; the panel branch below advances it.
            if f == 120 {
                input.press(KeyCode::Tab);
            }
            if brewing.active {
                brewing.auto_serve = true;
                let r = &RECIPES[brewing.recipe_idx];
                let center = (r.temp_min + r.temp_max) * 0.5;
                if brewing.temp < center - 1.5 {
                    input.press(KeyCode::ArrowUp);
                } else if brewing.temp > center + 1.5 {
                    input.press(KeyCode::ArrowDown);
                }
            } else {
                // Accept the front waiting customer, if any.
                let mut want = false;
                let mut best = u32::MAX;
                for c in customers.iter() {
                    if c.state == CustomerState::Waiting && c.queue_slot < best {
                        best = c.queue_slot;
                        want = true;
                    }
                }
                if want {
                    input.press(KeyCode::Enter);
                }
            }
        }
        GameScreen::Market | GameScreen::Upgrades | GameScreen::RecipeBook => {
            // Dwell ~1s in each panel, then Tab to the next one.
            if f >= 150 && f % 30 == 0 {
                input.press(KeyCode::Tab);
            }
        }
        GameScreen::DayReport => {
            input.press(KeyCode::Enter);
        }
        _ => {}
    }
}
