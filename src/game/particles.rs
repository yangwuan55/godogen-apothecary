//! Particle effects and floating text driven by FxEvent.

use super::resources::{FxEvent, FxKind};
use super::visual::{Bubble, animate_bubbles};
use bevy::prelude::*;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FxEvent>();
        app.add_systems(Update, (spawn_from_fx, animate_bubbles, animate_floaters));
    }
}

#[derive(Component)]
struct Floater {
    vel: Vec2,
    life: f32,
    max_life: f32,
}

fn spawn_from_fx(
    mut commands: Commands,
    mut events: MessageReader<FxEvent>,
    asset_server: Res<AssetServer>,
) {
    for ev in events.read() {
        match ev.kind {
            FxKind::Bubble | FxKind::Steam => {}
            FxKind::Sparkle => {
                for _ in 0..6 {
                    let angle = rand01() * std::f32::consts::TAU;
                    let speed = 60.0 + rand01() * 80.0;
                    commands.spawn((
                        Sprite::from_color(Color::srgb(1.0, 0.9, 0.4), Vec2::splat(6.0)),
                        Transform::from_xyz(ev.pos.x, ev.pos.y, 5.0),
                        Bubble {
                            vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                            life: 0.7,
                        },
                    ));
                }
            }
            FxKind::Fizzle => {
                for _ in 0..4 {
                    let angle = rand01() * std::f32::consts::TAU;
                    let speed = 50.0 + rand01() * 60.0;
                    commands.spawn((
                        Sprite::from_color(Color::srgb(0.7, 0.7, 0.8), Vec2::splat(7.0)),
                        Transform::from_xyz(ev.pos.x, ev.pos.y, 5.0),
                        Bubble {
                            vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                            life: 0.5,
                        },
                    ));
                }
            }
            FxKind::GoldText | FxKind::QualityText | FxKind::LevelUp => {
                if let Some(txt) = &ev.text {
                    let color = match ev.kind {
                        FxKind::GoldText => Color::srgb(0.98, 0.85, 0.35),
                        FxKind::QualityText => Color::srgb(0.6, 0.9, 0.6),
                        _ => Color::srgb(0.9, 0.5, 0.9),
                    };
                    commands.spawn((
                        Text2d::new(txt.clone()),
                        TextFont {
                            font: asset_server.load("fonts/NotoSansSC.ttf").into(),
                            font_size: FontSize::Px(22.0),
                            weight: FontWeight::BOLD,
                            ..default()
                        },
                        TextColor(color),
                        Transform::from_xyz(ev.pos.x, ev.pos.y, 8.0),
                        Floater {
                            vel: Vec2::new(0.0, 55.0),
                            life: 1.4,
                            max_life: 1.4,
                        },
                    ));
                }
            }
        }
    }
}

fn animate_floaters(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Floater, &mut Transform, &mut TextColor)>,
) {
    for (e, mut f, mut t, mut tc) in &mut q {
        f.life -= time.delta_secs();
        if f.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        t.translation += (f.vel * time.delta_secs()).extend(0.0);
        tc.0.set_alpha((f.life / f.max_life).clamp(0.0, 1.0));
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
