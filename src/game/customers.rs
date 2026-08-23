//! Customer spawning, movement, patience, and order acceptance.

use super::brewing::update_brewing;
use super::data::{CUSTOMER_KINDS, RECIPES, UpgradeId};
use super::resources::{
    Brewing, Customer, CustomerQueue, CustomerState, Economy, FxEvent, FxKind, GameScreen,
    Inventory, UpgradesState,
};
use bevy::prelude::*;

/// Front-of-line position (counter).
pub const COUNTER_POS: Vec2 = Vec2::new(300.0, -60.0);
pub const QUEUE_OFFSET: f32 = 118.0;

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
            )
                .chain()
                .run_if(in_state(GameScreen::Playing)),
        );
        app.add_systems(Update, update_brewing.run_if(in_state(GameScreen::Playing)));
    }
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
    // Pick a kind unlocked at this reputation.
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
        // Order bubble (text updated by bubble_system).
        p.spawn((
            Text2d::new(""),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf").into(),
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 1.0)),
            TextBackgroundColor(Color::srgba(0.1, 0.12, 0.25, 0.85)),
            Transform::from_xyz(0.0, 66.0, 4.0),
        ));
    });
}

/// Move customers toward their queue target; walk off screen when leaving/served.
fn move_customers(time: Res<Time>, mut q: Query<(&mut Customer, &mut Transform)>) {
    for (mut c, mut t) in &mut q {
        let target = if c.state == CustomerState::Leaving || c.state == CustomerState::Served {
            Vec3::new(1150.0, 80.0, 10.0)
        } else {
            Vec3::new(c.target_pos.x, c.target_pos.y, 10.0)
        };
        let cur = t.translation;
        let d = target - cur;
        if d.length() > 2.0 {
            let speed = 130.0;
            t.translation = cur + d.normalize() * (speed * time.delta_secs()).min(d.length());
        } else if c.state == CustomerState::Walking {
            c.state = CustomerState::Waiting;
        }
        // Gentle walk bob while moving.
        if c.state == CustomerState::Walking {
            c.wobble += time.delta_secs() * 9.0;
            t.translation.y = cur.y + (c.wobble).sin() * 2.0;
        }
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
                    text: Some("customer left".into()),
                });
            }
        }
    }
}

/// Accept the front customer's order with Enter/E.
fn accept_order(
    input: Res<ButtonInput<KeyCode>>,
    mut brewing: ResMut<Brewing>,
    mut inv: ResMut<Inventory>,
    _up: Res<UpgradesState>,
    mut q: Query<(Entity, &mut Customer)>,
    mut fx: MessageWriter<FxEvent>,
) {
    if brewing.active || !(input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::KeyE))
    {
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
            text: Some("Need ingredients!".into()),
        });
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

/// After a served/left customer clears the counter, everyone behind moves up one slot.
fn advance_queue(mut q: Query<&mut Customer>) {
    // Reassign slots so the front is 0 whenever no one at slot 0 is served/waiting at the counter.
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
        // Everyone shifts left.
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

fn rand_idx(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    (rand01() * n as f32) as usize % n
}
