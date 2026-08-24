//! Headless service-flow tests (issue #1).
//!
//! Seam: the real game app built through `build_app`, with winit/window/
//! render/audio disabled, stepped at a fixed 30 Hz exactly like the offscreen
//! capture. Player input enters through the same `ButtonInput` path the
//! autopilot uses (PreUpdate, after Bevy's input systems), and assertions read
//! live game resources only — no internal implementation details.

use bevy::prelude::*;
use godogen_apothecary::build_app;
use godogen_apothecary::game::customers::{COUNTER_POS, pick_recipe};
use godogen_apothecary::game::data::RECIPES;
use godogen_apothecary::game::resources::{
    Brewing, Customer, CustomerState, Economy, GameScreen, Inventory,
};
use std::time::Duration;

fn headless_app() -> App {
    let mut app = build_app(|p| {
        p.set(bevy::window::WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        })
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::audio::AudioPlugin>()
    });
    // Bevy 0.19 initializes the renderer asynchronously. `App::run` waits for
    // plugin readiness before stepping; a manual update loop must do the same,
    // or frame 1 races RenderDevice insertion and render-adjacent systems
    // (e.g. morph batching) fail validation.
    loop {
        match app.plugins_state() {
            bevy::app::PluginsState::Adding => {
                bevy::tasks::tick_global_task_pools_on_main_thread();
                std::thread::sleep(Duration::from_millis(2));
            }
            bevy::app::PluginsState::Ready => app.finish(),
            bevy::app::PluginsState::Finished => app.cleanup(),
            bevy::app::PluginsState::Cleaned => break,
        }
    }
    // AudioPlugin is off (silent tests); audio.rs still needs the asset store.
    app.init_resource::<Assets<bevy::audio::AudioSource>>();
    // 与 capture 一致：无窗口环境下不生成窗口相机。
    app.insert_resource(godogen_apothecary::game::core::WindowCamera(false));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(1.0 / 30.0),
    ));
    app.insert_resource(bevy::time::Time::<Fixed>::from_hz(30.0));
    app.insert_resource(Driver {
        frame: 0,
        presses: vec![],
    });
    app.add_systems(PreUpdate, drive.after(bevy::input::InputSystems));
    app
}

#[derive(Resource)]
struct Driver {
    frame: u64,
    presses: Vec<(u64, KeyCode)>,
}

fn drive(mut input: ResMut<ButtonInput<KeyCode>>, mut d: ResMut<Driver>) {
    for k in [KeyCode::Enter, KeyCode::Tab, KeyCode::Space] {
        input.release(k);
    }
    d.frame += 1;
    for (f, k) in &d.presses {
        if *f == d.frame {
            input.press(*k);
        }
    }
}

fn start_game(app: &mut App) {
    app.world_mut()
        .resource_mut::<Driver>()
        .presses
        .push((5, KeyCode::Enter));
}

/// Spawn a bare logic-level customer (visual children are irrelevant here).
fn spawn_customer(app: &mut App, slot: u32, recipe_idx: usize, patience: f32) -> Entity {
    app.world_mut()
        .spawn((
            Customer {
                kind_idx: 0,
                recipe_idx,
                patience,
                patience_max: patience,
                budget: 300,
                state: CustomerState::Waiting,
                target_pos: COUNTER_POS + Vec2::new(slot as f32 * 118.0, 0.0),
                home_pos: Vec2::new(1050.0, 60.0),
                wobble: 0.0,
                queue_slot: slot,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .id()
}

fn fill_inventory(app: &mut App, units: u32) {
    for c in app
        .world_mut()
        .resource_mut::<Inventory>()
        .counts
        .iter_mut()
    {
        *c = units;
    }
}

fn run(app: &mut App, frames: u64) {
    for _ in 0..frames {
        app.update();
    }
}

// --- Case D regression: entering Playing must not panic (ForceDayEnd init) --
#[test]
fn d_entering_playing_does_not_panic() {
    let mut app = headless_app();
    start_game(&mut app);
    run(&mut app, 150);
    let state = *app.world().resource::<State<GameScreen>>().get();
    assert_eq!(state, GameScreen::Playing);
}

// --- Case A: a queuer timing out must NOT abort the active brew -------------
#[test]
fn a_queuer_timeout_keeps_brew_alive() {
    let mut app = headless_app();
    start_game(&mut app);
    run(&mut app, 10); // reach Playing
    fill_inventory(&mut app, 9);
    spawn_customer(&mut app, 0, 0, 100.0); // front, long patience
    spawn_customer(&mut app, 1, 1, 0.25); // rear, expires in ~8 frames
    app.world_mut()
        .resource_mut::<Driver>()
        .presses
        .push((12, KeyCode::Enter));

    run(&mut app, 40); // ~1.3s: rear customer expired long ago
    {
        let brewing = app.world().resource::<Brewing>();
        assert!(brewing.active, "brew must survive an unrelated walkout");
    }
    let econ = app.world().resource::<Economy>();
    assert_eq!(econ.lost, 1, "rear customer should be counted as lost");

    // Let the brew finish (治愈药水 6s) and confirm the sale lands.
    run(&mut app, 260);
    let econ = app.world().resource::<Economy>();
    assert!(econ.gold > 60, "order should complete and pay out");
    assert_eq!(econ.served, 1);
}

// --- Case B: patience freezes once accepted ---------------------------------
#[test]
fn b_served_patience_is_frozen() {
    let mut app = headless_app();
    start_game(&mut app);
    run(&mut app, 10);
    fill_inventory(&mut app, 9);
    spawn_customer(&mut app, 0, 0, 0.3); // would expire ~9 frames if ticking
    app.world_mut()
        .resource_mut::<Driver>()
        .presses
        .push((12, KeyCode::Enter));

    run(&mut app, 60); // ~2s: far beyond the original 0.3s patience
    {
        let brewing = app.world().resource::<Brewing>();
        assert!(brewing.active, "accepted order keeps brewing past patience");
    }
    let still_there = app
        .world_mut()
        .query::<(&Customer,)>()
        .iter(app.world())
        .any(|(c,)| c.state == CustomerState::Served);
    assert!(still_there, "buyer must still be at the counter");

    run(&mut app, 240); // finish the potion
    let econ = app.world().resource::<Economy>();
    assert!(econ.gold > 60);
    assert_eq!(econ.served, 1);
    assert_eq!(econ.lost, 0, "nobody walks out once accepted");
}

// --- Case C: pick_recipe honours budget, tiers and reputation ---------------
#[test]
fn c_pick_recipe_respects_budget_and_tiers() {
    // Affordable pool: 贵族 budget caps below 凤凰之泪 (220g).
    for _ in 0..100 {
        let idx = pick_recipe(3, 3, 8, 200);
        assert_eq!(RECIPES[idx].tier, 3);
        assert!(
            RECIPES[idx].base_price <= 200,
            "picked unaffordable {}",
            idx
        );
    }
    // Nothing affordable at budget 5 → documented fallback to any T1.
    for _ in 0..50 {
        let idx = pick_recipe(1, 1, 1, 5);
        assert_eq!(RECIPES[idx].tier, 1);
    }
    // Tier window above reputation → fall back to unlocked tiers.
    for _ in 0..50 {
        let idx = pick_recipe(2, 2, 1, 999);
        assert_eq!(RECIPES[idx].tier, 1);
    }
}
