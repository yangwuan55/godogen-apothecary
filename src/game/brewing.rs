//! Brewing: temperature/progress/stir quality mechanic.
//!
//! Temperature is driven by the `TempControl` resource (set by both keyboard and
//! mouse via `actions::collect_input`), stir is a discrete `UiAction::Stir`.

use super::actions::UiAction;
use super::customers::COUNTER_POS;
use super::data::{Quality, RECIPES, UpgradeId, quality_from_score};
use super::audio::SfxRequest;
use super::resources::{Brewing, Customer, CustomerState, Economy, FxEvent, FxKind, TempControl, UpgradesState};
use bevy::prelude::*;

/// Update an active brew. Finishes it (earning gold/rep) when progress completes.
pub fn update_brewing(
    mut brewing: ResMut<Brewing>,
    mut econ: ResMut<Economy>,
    up: Res<UpgradesState>,
    time: Res<Time>,
    temp: Res<TempControl>,
    mut actions: MessageReader<UiAction>,
    mut customers: Query<(Entity, &mut Customer)>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxRequest>,
) {
    if !brewing.active {
        return;
    }
    let dt = time.delta_secs();
    let recipe = &RECIPES[brewing.recipe_idx];

    // --- Temperature (shared keyboard/mouse hold state) ---
    let lvl = up.level(UpgradeId::Furnace);
    let heat = (26.0 - 3.0 * lvl as f32).max(10.0);
    let cool = (22.0 - 3.0 * lvl as f32).max(8.0);
    let drift = (0.07 * (1.0 - 0.2 * lvl as f32)).max(0.01);

    if temp.up {
        brewing.temp += heat * dt;
    } else if temp.down {
        brewing.temp -= cool * dt;
    } else {
        brewing.temp += (50.0 - brewing.temp) * drift * dt;
    }
    brewing.temp = brewing.temp.clamp(0.0, 100.0);

    // --- Progress ---
    let clvl = up.level(UpgradeId::Cauldron);
    let (wmin, wmax) = (
        recipe.temp_min - 4.0 * clvl as f32,
        recipe.temp_max + 4.0 * clvl as f32,
    );
    let speed = 1.0 / (recipe.brew_time * (0.96_f32).powi(clvl as i32));

    brewing.raw_time += dt;
    brewing.progress += speed * 100.0 * dt;

    if brewing.temp >= wmin && brewing.temp <= wmax {
        brewing.in_window_time += dt;
    }
    if brewing.temp < recipe.temp_min - 15.0 {
        brewing.cold_time += dt;
    }

    // --- Burn ---
    let burn_limit = 2.5 - 0.3 * clvl as f32;
    if brewing.temp > recipe.temp_max + 18.0 {
        brewing.burn_time += dt;
        if brewing.burn_time > burn_limit && !brewing.burnt {
            brewing.burnt = true;
            fx.write(FxEvent {
                kind: FxKind::Fizzle,
                pos: Vec2::new(COUNTER_POS.x + 180.0, 240.0),
                text: Some("烧焦了！".into()),
            });
            sfx.write(SfxRequest::Burn);
        }
    } else {
        brewing.burn_time = 0.0;
    }

    // --- Stir ---
    let points = recipe.stir_points as usize;
    if brewing.stir_hits.len() != points {
        brewing.stir_hits = vec![false; points];
    }
    let stir_pressed = actions.read().any(|a| *a == UiAction::Stir);
    for i in 0..points {
        let at = 100.0 * (i as f32 + 0.5) / points as f32;
        if brewing.stir_hits[i] {
            continue;
        }
        if brewing.progress >= at && brewing.progress <= at + 20.0 {
            if stir_pressed || (brewing.auto_serve && brewing.progress >= at + 3.0) {
                brewing.stir_hits[i] = true;
                fx.write(FxEvent {
                    kind: FxKind::Sparkle,
                    pos: Vec2::new(COUNTER_POS.x + 180.0, 250.0),
                    text: None,
                });
                sfx.write(SfxRequest::Stir);
            }
        } else if brewing.progress > at + 20.0 {
            brewing.stir_hits[i] = true; // missed
        }
    }

    // --- Finish ---
    if brewing.progress >= 100.0 {
        finish_brew(&mut brewing, &mut econ, &mut fx, &mut sfx);
        // The front served customer receives the potion and leaves happily.
        let mut front_e: Option<Entity> = None;
        let mut best = u32::MAX;
        for (e, c) in customers.iter() {
            if c.state == CustomerState::Served && c.queue_slot < best {
                best = c.queue_slot;
                front_e = Some(e);
            }
        }
        if let Some(e) = front_e {
            if let Ok((_, mut c)) = customers.get_mut(e) {
                c.state = CustomerState::Leaving;
            }
            fx.write(FxEvent {
                kind: FxKind::Sparkle,
                pos: Vec2::new(COUNTER_POS.x, COUNTER_POS.y + 40.0),
                text: None,
            });
        }
    }
}

fn finish_brew(
    brewing: &mut Brewing,
    econ: &mut Economy,
    fx: &mut MessageWriter<FxEvent>,
    sfx: &mut MessageWriter<SfxRequest>,
) {
    let recipe = &RECIPES[brewing.recipe_idx];

    let window_ratio = (brewing.in_window_time / brewing.raw_time.max(0.01)).min(1.0);
    let hit_count = brewing.stir_hits.iter().filter(|&&h| h).count();
    let hit_ratio = if brewing.stir_hits.is_empty() {
        0.0
    } else {
        hit_count as f32 / brewing.stir_hits.len() as f32
    };
    let cold_ratio = (brewing.cold_time / brewing.raw_time.max(0.01)).min(1.0);

    let mut score = 0.30 + 0.45 * window_ratio + 0.18 * hit_ratio - 0.18 * cold_ratio;
    if brewing.burnt {
        score -= 0.25;
    }
    let score = score.clamp(0.0, 1.0);
    let q = quality_from_score(score);

    let earned = ((recipe.base_price as f32) * q.price_mult()).round() as u32;
    econ.gold += earned;
    econ.day_income += earned;
    let rep = q.rep_gain();
    econ.reputation += rep;
    econ.day_quality[q_idx(q)] += 1;
    if q == Quality::Perfect {
        econ.perfect_count += 1;
    }
    econ.served += 1;

    fx.write(FxEvent {
        kind: FxKind::QualityText,
        pos: Vec2::new(COUNTER_POS.x + 60.0, 320.0),
        text: Some(format!("{}：+{}g", q.label(), earned)),
    });
    fx.write(FxEvent {
        kind: FxKind::GoldText,
        pos: Vec2::new(COUNTER_POS.x + 40.0, 300.0),
        text: Some(format!("+{} 声望", rep)),
    });
    sfx.write(SfxRequest::Success);
    sfx.write(SfxRequest::Coin);

    brewing.active = false;
    brewing.progress = 0.0;
    brewing.in_window_time = 0.0;
    brewing.raw_time = 0.0;
    brewing.burn_time = 0.0;
    brewing.cold_time = 0.0;
    brewing.burnt = false;
    brewing.stir_hits.clear();
    brewing.auto_serve = false;
}

fn q_idx(q: Quality) -> usize {
    match q {
        Quality::Poor => 0,
        Quality::Normal => 1,
        Quality::Good => 2,
        Quality::Perfect => 3,
    }
}
