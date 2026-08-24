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
use godogen_apothecary::game::resources::{
    Brewing, Customer, CustomerState, ForceDayEnd, GameScreen,
};
use std::time::Duration;

const W: u32 = 1280;
const H: u32 = 720;
const FRAMES: u32 = 480; // 16 s @ 30 fps

#[derive(Resource)]
struct CaptureTarget {
    handle: Handle<Image>,
    frame: u32,
}

/// Tracks which captured frame the autopilot has already driven. The schedule
/// runner catch-up loop can run several `Update`s for the same captured frame
/// (the async screenshot gates `capture_frame`), so without this guard a single
/// Tab/Enter press would repeat once per extra update and flip multiple panels.
#[derive(Resource, Default)]
struct AutopilotState {
    acted_on: u32,
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
    // Sync the fixed-timestep loop with the 30 fps capture cadence so autopilot
    // and state transitions run exactly once per captured frame (otherwise the
    // default 1/60 fixed timestep runs the update twice per frame and a single
    // Tab press can flip two panels at once).
    .insert_resource(Time::<Fixed>::from_hz(30.0))
    .insert_resource(godogen_apothecary::game::core::WindowCamera(false))
    .insert_resource(godogen_apothecary::game::resources::TutorialSettings { enabled: false })
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1.0 / 30.0,
    )))
    .init_resource::<AutopilotState>()
    .init_resource::<ForceDayEnd>()
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
/// through the panels (with a purchase + quantity demo) before resuming the
/// brewing loop so the clip shows the full loop including delivery.
fn autopilot(
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut ap: ResMut<AutopilotState>,
    mut brewing: ResMut<Brewing>,
    mut force_day: ResMut<ForceDayEnd>,
    state: Res<State<GameScreen>>,
    customers: Query<(&Customer, &Transform)>,
    target: Res<CaptureTarget>,
) {
    // Held keys: release every frame, then re-press only when needed.
    input.release(KeyCode::ArrowUp);
    input.release(KeyCode::ArrowDown);
    // Momentary keys: release so each press fires a fresh `just_pressed`.
    input.release(KeyCode::Tab);
    input.release(KeyCode::Enter);
    input.release(KeyCode::Space);
    input.release(KeyCode::Escape);
    input.release(KeyCode::Digit1);
    input.release(KeyCode::Digit2);
    input.release(KeyCode::Digit3);
    input.release(KeyCode::BracketRight);

    let f = target.frame;
    if ap.acted_on == f {
        return;
    }
    ap.acted_on = f;

    // ---- Scripted one-shot inputs (frame-driven, independent of current
    // state, so a Tab that should open the market fires even if a previous
    // press already moved us there). --------------------------------
    let script: &[(u32, Option<KeyCode>, Option<bool>)] = &[
        (20, Some(KeyCode::Enter), None),         // Title -> Playing
        (120, Some(KeyCode::Tab), None),          // -> Market  (visit #1)
        (128, Some(KeyCode::Digit1), None),       // buy material 0
        (134, Some(KeyCode::BracketRight), None), // qty +1
        (140, Some(KeyCode::Digit1), None),       // buy again (2 units)
        (148, Some(KeyCode::Tab), None),          // -> Upgrades
        (156, Some(KeyCode::Digit2), None),       // buy Furnace if affordable
        (164, Some(KeyCode::Tab), None),          // -> RecipeBook
        (172, Some(KeyCode::Tab), None),          // -> Playing
        (300, Some(KeyCode::Tab), None),          // -> Market  (visit #2)
        (308, Some(KeyCode::Digit3), None),       // buy material 2
        (316, Some(KeyCode::Tab), None),          // -> Upgrades
        (324, Some(KeyCode::Digit1), None),       // buy Cauldron if affordable
        (332, Some(KeyCode::Tab), None),          // -> RecipeBook
        (340, Some(KeyCode::Tab), None),          // -> Playing
        (420, Some(KeyCode::Escape), None),       // pause overlay
        (432, Some(KeyCode::Escape), None),       // resume
        (460, None, Some(true)),                  // force day end
        (470, Some(KeyCode::Enter), Some(false)), // continue to day 2
    ];
    for (sf, key, flag) in script {
        if f == *sf {
            if let Some(k) = key {
                input.press(*k);
            }
            if let Some(v) = flag {
                force_day.0 = *v;
            }
        }
    }

    // ---- Ongoing gameplay: keep a brew on target temperature, accept the
    // front waiting customer whenever the cauldron is free. --------------
    if *state.get() == GameScreen::Playing {
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
            let mut want = false;
            let mut best = u32::MAX;
            for (c, _tr) in customers.iter() {
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
}
