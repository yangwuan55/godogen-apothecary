//! Customer spawning, movement, patience, order acceptance, speech bubbles.

use super::actions::{InputSet, UiAction};
use super::audio::SfxRequest;
use super::brewing::update_brewing;
use super::data::{CUSTOMER_KINDS, RECIPES, UpgradeId};
use super::resources::{
    Brewing, Customer, CustomerQueue, CustomerState, Economy, FxEvent, FxKind, GameScreen,
    Inventory, Paused, UpgradesState,
};
use bevy::prelude::*;

/// Front-of-line position (counter).
pub const COUNTER_POS: Vec2 = Vec2::new(-215.0, -60.0);
pub const QUEUE_OFFSET: f32 = 118.0;

/// Marker on a customer's speech bubble so it can be updated each frame.
#[derive(Component)]
pub struct CustomerBubble;

pub struct CustomersPlugin;

impl Plugin for CustomersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_customers,
                move_customers,
                tick_patience,
                accept_order,
                advance_queue,
                leave_customers,
                update_bubbles,
            )
                .chain()
                .after(InputSet)
                .run_if(playing_and_not_paused),
        );
        app.add_systems(
            Update,
            update_brewing.after(InputSet).run_if(playing_and_not_paused),
        );
    }
}

fn playing_and_not_paused(state: Res<State<GameScreen>>, paused: Res<Paused>) -> bool {
    *state.get() == GameScreen::Playing && !paused.0
}

fn sign_capacity(sign_lvl: u8) -> u32 {
    super::data::SIGN_CUSTOMERS[sign_lvl as usize]
}

fn rand01() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (n % 1000) as f32 / 1000.0
}

/// Spawn customers over the day up to the sign capacity.
fn spawn_customers(
    mut commands: Commands,
    mut queue: ResMut<CustomerQueue>,
    econ: Res<Economy>,
    up: Res<UpgradesState>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    customers: Query<(), With<Customer>>,
) {
    let max_today = sign_capacity(up.level(UpgradeId::Sign));
    if queue.spawned_today >= max_today {
        return;
    }
    queue.spawn_timer -= time.delta_secs();
    let spawn_interval = (econ.day_length / max_today.max(1) as f32).clamp(2.2, 9.0);
    if queue.spawn_timer <= 0.0 {
        queue.spawn_timer = spawn_interval * (0.7 + 0.6 * rand01());
        let slot = customers.iter().count() as u32;
        spawn_one_customer(&mut commands, &mut queue, &econ, &up, &asset_server, slot);
    }
}

fn spawn_one_customer(
    commands: &mut Commands,
    queue: &mut CustomerQueue,
    econ: &Economy,
    _up: &UpgradesState,
    asset_server: &AssetServer,
    slot: u32,
) {
    let unlocked: Vec<usize> = CUSTOMER_KINDS
        .iter()
        .enumerate()
        .filter(|(_, k)| k.unlock_reputation <= econ.rep_level)
        .map(|(i, _)| i)
        .collect();
    if unlocked.is_empty() {
        return;
    }
    let kind_idx = pick_weighted(&unlocked, econ.rep_level);

    let kind = &CUSTOMER_KINDS[kind_idx];
    let available: Vec<usize> = RECIPES
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.tier >= kind.min_tier && r.tier <= kind.max_tier && r.tier <= econ.rep_level
        })
        .map(|(i, _)| i)
        .collect();
    let recipe_idx = if available.is_empty() {
        let any: Vec<usize> = RECIPES
            .iter()
            .enumerate()
            .filter(|(_, r)| r.tier <= econ.rep_level)
            .map(|(i, _)| i)
            .collect();
        if any.is_empty() {
            return;
        }
        any[rand_idx(any.len())]
    } else {
        available[rand_idx(available.len())]
    };

    let budget = (kind.budget_min as f32 + rand01() * (kind.budget_max - kind.budget_min) as f32)
        .round() as u32;
    let patience = kind.patience * (0.8 + 0.4 * rand01());
    let target_pos = Vec2::new(COUNTER_POS.x + slot as f32 * QUEUE_OFFSET, COUNTER_POS.y);

    spawn_character(
        commands,
        asset_server,
        kind_idx,
        recipe_idx,
        budget,
        patience,
        target_pos,
        slot,
    );
    queue.spawned_today += 1;
}

fn pick_weighted(indices: &[usize], rep_level: u8) -> usize {
    let mut total = 0u32;
    let mut weights = Vec::with_capacity(indices.len());
    for &i in indices {
        let k = &CUSTOMER_KINDS[i];
        let w = 1 + k
            .unlock_reputation
            .saturating_sub(rep_level.saturating_sub(2)) as u32;
        total += w;
        weights.push(w);
    }
    let mut r = (rand01() * total as f32) as u32;
    for (idx, &i) in indices.iter().enumerate() {
        if r < weights[idx] {
            return i;
        }
        r -= weights[idx];
    }
    indices[0]
}

fn spawn_character(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind_idx: usize,
    recipe_idx: usize,
    budget: u32,
    patience: f32,
    target_pos: Vec2,
    slot: u32,
) {
    let kind = &CUSTOMER_KINDS[kind_idx];
    let start = Vec3::new(1050.0 + slot as f32 * 30.0, 60.0, 10.0);
    let mut e = commands.spawn((
        Customer {
            kind_idx,
            recipe_idx,
            patience,
            patience_max: patience,
            budget,
            state: CustomerState::Walking,
            target_pos,
            home_pos: Vec2::new(1050.0, 60.0),
            wobble: rand01() * 6.28,
            queue_slot: slot,
        },
        Transform::from_translation(start),
        Visibility::default(),
    ));
    e.with_children(|p| {
        p.spawn((
            Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.25), Vec2::new(44.0, 10.0)),
            Transform::from_xyz(0.0, -34.0, -1.0),
        ));
        p.spawn((
            Sprite::from_color(kind.body_color, Vec2::new(34.0, 30.0)),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        p.spawn((
            Sprite::from_color(Color::srgb(0.92, 0.80, 0.70), Vec2::new(26.0, 26.0)),
            Transform::from_xyz(0.0, 24.0, 1.0),
        ));
        p.spawn((
            Sprite::from_color(Color::srgb(0.1, 0.1, 0.15), Vec2::new(4.0, 6.0)),
            Transform::from_xyz(-6.0, 26.0, 2.0),
        ));
        p.spawn((
            Sprite::from_color(Color::srgb(0.1, 0.1, 0.15), Vec2::new(4.0, 6.0)),
            Transform::from_xyz(6.0, 26.0, 2.0),
        ));
        match kind.hat_style {
            1 => {
                p.spawn((
                    Sprite::from_color(kind.hat_color, Vec2::new(32.0, 12.0)),
                    Transform::from_xyz(0.0, 40.0, 2.0),
                ));
                p.spawn((
                    Sprite::from_color(kind.hat_color, Vec2::new(20.0, 8.0)),
                    Transform::from_xyz(0.0, 48.0, 2.0),
                ));
            }
            2 => {
                p.spawn((
                    Sprite::from_color(kind.hat_color, Vec2::new(34.0, 10.0)),
                    Transform::from_xyz(0.0, 39.0, 2.0),
                ));
                p.spawn((
                    Sprite::from_color(kind.hat_color, Vec2::new(12.0, 22.0)),
                    Transform::from_xyz(0.0, 54.0, 3.0),
                ));
            }
            3 => {
                p.spawn((
                    Sprite::from_color(kind.hat_color, Vec2::new(34.0, 12.0)),
                    Transform::from_xyz(0.0, 40.0, 2.0),
                ));
                for dx in [-12.0_f32, 12.0] {
                    p.spawn((
                        Sprite::from_color(Color::srgb(1.0, 0.95, 0.3), Vec2::new(6.0, 6.0)),
                        Transform::from_xyz(dx, 44.0, 3.0),
                    ));
                }
            }
            _ => {}
        }
        // Speech bubble (content updated by `update_bubbles`).
        p.spawn((
            Text2d::new(""),
            TextFont {
                font: asset_server.load("fonts/NotoSansSC.ttf").into(),
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 1.0)),
            TextBackgroundColor(Color::srgba(0.10, 0.12, 0.25, 0.88)),
            CustomerBubble,
            Transform::from_xyz(0.0, 68.0, 4.0),
        ));
    });
}

/// Move customers along their path to the queue target; walking-off customers
/// head for the exit. A small bob is applied *on top of* the path position so
/// the vertical movement is never swallowed by the bob.
fn move_customers(time: Res<Time>, mut q: Query<(&mut Customer, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut c, mut t) in &mut q {
        let target = if c.state == CustomerState::Leaving {
            Vec3::new(1150.0, 80.0, 10.0)
        } else {
            Vec3::new(c.target_pos.x, c.target_pos.y, 10.0)
        };
        let cur = t.translation;
        let d = target - cur;
        let step = 130.0 * dt;
        if d.length() > step && step > 0.0 {
            t.translation = cur + d.normalize() * step;
        } else {
            t.translation = target;
            if c.state == CustomerState::Walking {
                c.state = CustomerState::Waiting;
            }
        }
        // Gentle bob on top of the path (never resets the path movement).
        c.wobble += dt * 9.0;
        let bob = match c.state {
            CustomerState::Walking => (c.wobble).sin() * 2.0,
            CustomerState::Waiting => (c.wobble * 0.5).sin() * 1.5,
            CustomerState::Served => (c.wobble * 1.4).sin() * 3.0,
            CustomerState::Leaving => (c.wobble).sin() * 2.0,
        };
        t.translation.y += bob;
    }
}

/// Patience drains while waiting/served; hitting zero makes the customer leave.
fn tick_patience(
    time: Res<Time>,
    mut q: Query<(Entity, &mut Customer)>,
    mut econ: ResMut<Economy>,
    mut brewing: ResMut<Brewing>,
    mut fx: MessageWriter<FxEvent>,
) {
    for (_e, mut c) in &mut q {
        if c.state == CustomerState::Waiting || c.state == CustomerState::Served {
            c.patience -= time.delta_secs();
            if c.patience <= 0.0 && c.state != CustomerState::Leaving {
                if brewing.active {
                    brewing.active = false; // aborted order
                }
                c.state = CustomerState::Leaving;
                econ.lost += 1;
                fx.write(FxEvent {
                    kind: FxKind::Fizzle,
                    pos: Vec2::new(COUNTER_POS.x + 60.0, COUNTER_POS.y + 70.0),
                    text: Some("顾客离开了".into()),
                });
            }
        }
    }
}

/// Accept the front customer's order (mouse 接单 button or Enter/E).
fn accept_order(
    mut actions: MessageReader<UiAction>,
    mut brewing: ResMut<Brewing>,
    mut inv: ResMut<Inventory>,
    _up: Res<UpgradesState>,
    mut q: Query<(Entity, &mut Customer)>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxRequest>,
) {
    if brewing.active || !actions.read().any(|a| *a == UiAction::AcceptOrder) {
        return;
    }
    // Front = smallest queue_slot among Waiting.
    let mut front_e: Option<Entity> = None;
    let mut best_slot = u32::MAX;
    for (e, c) in q.iter_mut() {
        if c.state == CustomerState::Waiting && c.queue_slot < best_slot {
            best_slot = c.queue_slot;
            front_e = Some(e);
        }
    }
    let Some(front_e) = front_e else { return };
    let Ok((_e, mut cust)) = q.get_mut(front_e) else {
        return;
    };
    let recipe = &RECIPES[cust.recipe_idx];

    if !recipe.mats.iter().all(|&m| inv.counts[m as usize] >= 1) {
        fx.write(FxEvent {
            kind: FxKind::Fizzle,
            pos: Vec2::new(COUNTER_POS.x + 60.0, COUNTER_POS.y + 80.0),
            text: Some("缺少材料！".into()),
        });
        sfx.write(SfxRequest::Error);
        return;
    }
    for &m in recipe.mats {
        inv.counts[m as usize] -= 1;
    }
    brewing.active = true;
    brewing.recipe_idx = cust.recipe_idx;
    brewing.progress = 0.0;
    brewing.temp = 50.0;
    brewing.raw_time = 0.0;
    brewing.in_window_time = 0.0;
    brewing.cold_time = 0.0;
    brewing.burn_time = 0.0;
    brewing.burnt = false;
    brewing.stir_hits = vec![false; recipe.stir_points as usize];

    cust.state = CustomerState::Served;
}

/// After a served/left customer clears the counter, everyone behind moves up.
fn advance_queue(mut q: Query<&mut Customer>) {
    let mut serving: Vec<u32> = Vec::new();
    for c in q.iter() {
        if c.state == CustomerState::Waiting || c.state == CustomerState::Served {
            serving.push(c.queue_slot);
        }
    }
    if serving.is_empty() {
        return;
    }
    serving.sort_unstable();
    let min_slot = serving[0];
    if min_slot > 0 {
        for mut c in q.iter_mut() {
            if c.state == CustomerState::Waiting || c.state == CustomerState::Served {
                if c.queue_slot > 0 {
                    c.queue_slot -= 1;
                }
                c.target_pos = Vec2::new(
                    COUNTER_POS.x + c.queue_slot as f32 * QUEUE_OFFSET,
                    COUNTER_POS.y,
                );
            }
        }
    }
}

/// Despawn customers once they've walked off screen.
fn leave_customers(mut commands: Commands, q: Query<(Entity, &Customer, &Transform)>) {
    for (e, c, t) in &q {
        if (c.state == CustomerState::Leaving || c.state == CustomerState::Served)
            && t.translation.x > 1120.0
        {
            commands.entity(e).despawn();
        }
    }
}

/// Keep each customer's speech bubble in sync with their state.
fn update_bubbles(
    customers: Query<(&Customer, &Children)>,
    mut bubbles: Query<(
        &mut Text2d,
        &mut TextColor,
        &mut TextBackgroundColor,
        &mut Transform,
    )>,
) {
    for (c, children) in &customers {
        for child in children {
            if let Ok((mut text, mut color, mut bg, mut t)) = bubbles.get_mut(*child) {
                let ratio = (c.patience / c.patience_max).clamp(0.0, 1.0);
                let urgent = ratio < 0.25 && (c.state == CustomerState::Waiting || c.state == CustomerState::Served);
                let (txt, txt_col, bg_col) = match c.state {
                    CustomerState::Walking => {
                        ("".to_string(), Color::srgb(0.95, 0.95, 1.0), Color::srgba(0.10, 0.12, 0.25, 0.88))
                    }
                    CustomerState::Waiting => {
                        let recipe = &RECIPES[c.recipe_idx];
                        if urgent {
                            (
                                "快一点！我要走了！".to_string(),
                                Color::srgb(1.0, 0.45, 0.45),
                                Color::srgba(0.35, 0.06, 0.06, 0.92),
                            )
                        } else {
                            (
                                format!("想要「{}」", recipe.name),
                                Color::srgb(0.95, 0.95, 1.0),
                                Color::srgba(0.10, 0.12, 0.25, 0.88),
                            )
                        }
                    }
                    CustomerState::Served => {
                        if urgent {
                            ("还没好吗？！".to_string(), Color::srgb(1.0, 0.45, 0.45), Color::srgba(0.35, 0.06, 0.06, 0.92))
                        } else {
                            ("在熬了…".to_string(), Color::srgb(0.9, 0.9, 0.95), Color::srgba(0.10, 0.12, 0.25, 0.88))
                        }
                    }
                    CustomerState::Leaving => (
                        "等不及了…".to_string(),
                        Color::srgb(0.6, 0.6, 0.7),
                        Color::srgba(0.08, 0.08, 0.12, 0.85),
                    ),
                };
                text.0 = txt;
                color.0 = txt_col;
                bg.0 = bg_col;
                // Fade & shrink the bubble slightly while leaving.
                if c.state == CustomerState::Leaving {
                    let a = ((t.translation.x - 1120.0) / 150.0).clamp(0.15, 1.0);
                    let s = 0.7 + 0.3 * a;
                    t.scale = Vec3::splat(s);
                } else {
                    t.scale = Vec3::ONE;
                }
            }
        }
    }
}

fn rand_idx(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    (rand01() * n as f32) as usize % n
}
