//! Procedural shop scene: background, counter, shelves, cauldron, deco potions.

use super::resources::GameScreen;
use bevy::prelude::*;

/// Marker for the shop scene root.
#[derive(Component)]
pub struct ShopRoot;

pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_shop_scene);
        app.add_systems(
            Update,
            cauldron_effect
                .run_if(in_state(GameScreen::Playing))
                .run_if(not_paused),
        );
        app.add_systems(
            Update,
            cauldron_liquid
                .run_if(in_state(GameScreen::Playing))
                .run_if(not_paused),
        );
    }
}

fn not_paused(paused: Res<super::resources::Paused>) -> bool {
    !paused.0
}

/// The cauldron body entity + bubble spawner marker.
#[derive(Component)]
pub struct Cauldron;

/// The colored liquid surface inside the cauldron.
#[derive(Component)]
pub struct CauldronLiquid;

fn build_shop_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            ShopRoot,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
        ))
        .with_children(|p| {
            // Back wall
            p.spawn((
                Sprite::from_color(Color::srgb(0.32, 0.22, 0.14), Vec2::new(1280.0, 520.0)),
                Transform::from_xyz(0.0, 60.0, -20.0),
            ));
            // Wall plank lines
            for i in 0..6 {
                let y = -80.0 + i as f32 * 52.0;
                p.spawn((
                    Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.08), Vec2::new(1280.0, 3.0)),
                    Transform::from_xyz(0.0, y, -19.0),
                ));
            }
            // Floor
            p.spawn((
                Sprite::from_color(Color::srgb(0.38, 0.30, 0.22), Vec2::new(1280.0, 200.0)),
                Transform::from_xyz(0.0, -260.0, -18.0),
            ));
            // Rug under the counter area
            p.spawn((
                Sprite::from_color(Color::srgb(0.50, 0.20, 0.20), Vec2::new(520.0, 150.0)),
                Transform::from_xyz(-215.0, -230.0, -17.0),
            ));
            // Shelves (decorative) behind counter
            for (_i, (sx, c)) in [
                (-150.0, Color::srgb(0.75, 0.30, 0.35)),
                (-50.0, Color::srgb(0.35, 0.55, 0.85)),
                (50.0, Color::srgb(0.55, 0.75, 0.45)),
            ]
            .iter()
            .enumerate()
            {
                let x = -275.0 + *sx;
                p.spawn((
                    Sprite::from_color(Color::srgb(0.25, 0.18, 0.11), Vec2::new(90.0, 100.0)),
                    Transform::from_xyz(x, 60.0, -10.0),
                ));
                // Shelf bottles
                for j in 0..3 {
                    p.spawn((
                        Sprite::from_color(*c, Vec2::new(18.0, 30.0)),
                        Transform::from_xyz(x - 24.0 + j as f32 * 24.0, 70.0, -9.0),
                    ));
                }
            }
            // Counter
            p.spawn((
                Sprite::from_color(Color::srgb(0.42, 0.30, 0.18), Vec2::new(560.0, 34.0)),
                Transform::from_xyz(-215.0, -20.0, -5.0),
            ));
            p.spawn((
                Sprite::from_color(Color::srgb(0.30, 0.21, 0.13), Vec2::new(560.0, 60.0)),
                Transform::from_xyz(-215.0, 6.0, -6.0),
            ));
            // Sign
            p.spawn((
                Text2d::new("✦ 炼金药铺 ✦"),
                TextFont {
                    font: asset_server.load("fonts/NotoSansSC.ttf").into(),
                    font_size: FontSize::Px(36.0),
                    weight: FontWeight::BOLD,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.80, 0.35)),
                Transform::from_xyz(-215.0, 230.0, -8.0),
            ));
            // Cauldron (left of counter)
            p.spawn((
                Cauldron,
                Sprite::from_color(Color::srgb(0.20, 0.20, 0.24), Vec2::new(110.0, 60.0)),
                Transform::from_xyz(-395.0, -20.0, -4.0),
            ));
            p.spawn((
                Sprite::from_color(Color::srgb(0.30, 0.30, 0.36), Vec2::new(130.0, 16.0)),
                Transform::from_xyz(-395.0, 10.0, -3.0),
            ));
            // Cauldron liquid surface (color reflects current brewing state).
            p.spawn((
                CauldronLiquid,
                Sprite::from_color(Color::srgba(0.3, 0.7, 0.9, 0.85), Vec2::new(92.0, 22.0)),
                Transform::from_xyz(-395.0, 2.0, -2.0),
            ));
            // Deco potions on the counter
            for (dx, c) in [
                (420.0, Color::srgb(0.4, 0.8, 0.4)),
                (470.0, Color::srgb(0.8, 0.5, 0.8)),
            ] {
                p.spawn((
                    Sprite::from_color(c, Vec2::new(20.0, 34.0)),
                    Transform::from_xyz(-215.0 + dx, 26.0, -4.0),
                ));
                p.spawn((
                    Sprite::from_color(Color::srgb(0.45, 0.32, 0.22), Vec2::new(24.0, 10.0)),
                    Transform::from_xyz(-215.0 + dx, 46.0, -3.0),
                ));
            }
        });
}

/// Bubbles rising from the cauldron while brewing.
fn cauldron_effect(
    brewing: Res<super::resources::Brewing>,
    time: Res<Time>,
    mut commands: Commands,
    mut timer: Local<f32>,
    cauldron: Query<&Transform, With<Cauldron>>,
) {
    if !brewing.active {
        return;
    }
    *timer += time.delta_secs();
    if *timer < 0.18 {
        return;
    }
    *timer = 0.0;
    let Ok(ct) = cauldron.single() else {
        return;
    };
    let bx = ct.translation.x + (rand01() * 60.0 - 30.0);
    commands.spawn((
        Sprite::from_color(Color::srgba(0.5, 0.8, 0.9, 0.8), Vec2::splat(10.0)),
        Transform::from_xyz(bx, ct.translation.y + 10.0, 0.0),
        Bubble {
            vel: Vec2::new(0.0, 60.0),
            life: 1.2,
        },
    ));
}

#[derive(Component)]
pub struct Bubble {
    pub vel: Vec2,
    pub life: f32,
}

pub fn animate_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Bubble, &mut Transform, &mut Sprite)>,
) {
    for (e, mut b, mut t, mut s) in &mut q {
        b.life -= time.delta_secs();
        if b.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        t.translation.x += b.vel.x * time.delta_secs();
        t.translation.y += b.vel.y * time.delta_secs();
        let a = (b.life / 1.2).clamp(0.0, 1.0);
        s.color.set_alpha(a);
    }
}

fn rand01() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (n % 1000) as f32 / 1000.0
}

/// Tint the cauldron liquid: cold = blue, ideal window = recipe color,
/// overheating = red/orange. Pulses once burnt.
fn cauldron_liquid(
    brewing: Res<super::resources::Brewing>,
    time: Res<Time>,
    mut q: Query<&mut Sprite, With<CauldronLiquid>>,
) {
    let Ok(mut sprite) = q.single_mut() else { return };
    if !brewing.active {
        sprite.color = Color::srgba(0.30, 0.60, 0.85, 0.45);
        return;
    }
    let r = &RECIPES[brewing.recipe_idx];
    let base = r.color;
    let pulse = (time.elapsed_secs() * 8.0).sin() * 0.5 + 0.5;
    sprite.color = if brewing.burnt {
        Color::srgba(0.2, 0.1, 0.05, 0.9)
    } else if brewing.temp < r.temp_min - 15.0 {
        Color::srgba(0.35, 0.55, 0.95, 0.8)
    } else if brewing.temp > r.temp_max + 8.0 {
        let c = base.to_srgba();
        Color::srgba(
            (c.red * 0.4 + 0.85 * pulse).clamp(0.0, 1.0),
            c.green * 0.35,
            0.15,
            0.9,
        )
    } else {
        base.with_alpha(0.85)
    };
}

use super::data::RECIPES;
