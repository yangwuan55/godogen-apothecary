//! Tab navigation between panels, and Market / Upgrades purchase logic.

use super::data::{MATERIALS, NUM_MATERIALS, SHELF_CAPACITY, UPGRADES, UpgradeId};
use super::resources::{Economy, FxEvent, FxKind, GameScreen, Inventory, UpgradesState};
use bevy::prelude::*;

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tab_navigation);
        app.add_systems(Update, market_purchase.run_if(in_state(GameScreen::Market)));
        app.add_systems(
            Update,
            upgrade_purchase.run_if(in_state(GameScreen::Upgrades)),
        );
    }
}

const PANEL_ORDER: [GameScreen; 4] = [
    GameScreen::Playing,
    GameScreen::Market,
    GameScreen::Upgrades,
    GameScreen::RecipeBook,
];

/// Tab cycles through panels.
fn tab_navigation(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameScreen>>,
    mut next: ResMut<NextState<GameScreen>>,
) {
    if input.just_pressed(KeyCode::Tab) {
        let cur = *state.get();
        if let Some(pos) = PANEL_ORDER.iter().position(|&p| p == cur) {
            next.set(PANEL_ORDER[(pos + 1) % PANEL_ORDER.len()]);
        } else {
            next.set(GameScreen::Playing);
        }
    }
}

/// Buy materials with the matching number key (0..9, -, =).
fn market_purchase(
    input: Res<ButtonInput<KeyCode>>,
    mut econ: ResMut<Economy>,
    mut inv: ResMut<Inventory>,
    up: Res<UpgradesState>,
    mut fx: MessageWriter<FxEvent>,
) {
    let capacity = SHELF_CAPACITY[up.level(UpgradeId::Shelf) as usize];
    for i in 0..NUM_MATERIALS {
        if !input.just_pressed(key_for_index(i)) {
            continue;
        }
        let mat = &MATERIALS[i];
        if inv.counts[i] >= capacity {
            fx.write(FxEvent {
                kind: FxKind::Fizzle,
                pos: Vec2::new(300.0, 200.0),
                text: Some("货架已满！".into()),
            });
            continue;
        }
        let qty = inv.restock_qty;
        let cost = mat.cost * qty;
        if econ.gold >= cost {
            econ.gold -= cost;
            inv.counts[i] = (inv.counts[i] + qty).min(capacity);
            fx.write(FxEvent {
                kind: FxKind::GoldText,
                pos: Vec2::new(300.0, 200.0),
                text: Some(format!("购入 {}x{}", mat.name, qty)),
            });
        } else {
            fx.write(FxEvent {
                kind: FxKind::Fizzle,
                pos: Vec2::new(300.0, 200.0),
                text: Some("金币不足！".into()),
            });
        }
    }
}

/// Buy upgrades with keys 1..4.
fn upgrade_purchase(
    input: Res<ButtonInput<KeyCode>>,
    mut econ: ResMut<Economy>,
    mut up: ResMut<UpgradesState>,
    mut fx: MessageWriter<FxEvent>,
) {
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ]
    .iter()
    .enumerate()
    {
        if input.just_pressed(*key) {
            let def = &UPGRADES[i];
            let lvl = up.levels[i] as usize;
            if lvl >= def.max_level as usize {
                continue;
            }
            let cost = def.costs[lvl];
            if econ.gold >= cost {
                econ.gold -= cost;
                up.levels[i] += 1;
                fx.write(FxEvent {
                    kind: FxKind::GoldText,
                    pos: Vec2::new(300.0, 300.0),
                    text: Some(format!("{} 升至 {} 级", def.name, up.levels[i])),
                });
            } else {
                fx.write(FxEvent {
                    kind: FxKind::Fizzle,
                    pos: Vec2::new(300.0, 300.0),
                    text: Some("金币不足！".into()),
                });
            }
        }
    }
}

fn key_for_index(i: usize) -> KeyCode {
    match i {
        0 => KeyCode::Digit1,
        1 => KeyCode::Digit2,
        2 => KeyCode::Digit3,
        3 => KeyCode::Digit4,
        4 => KeyCode::Digit5,
        5 => KeyCode::Digit6,
        6 => KeyCode::Digit7,
        7 => KeyCode::Digit8,
        8 => KeyCode::Digit9,
        9 => KeyCode::Digit0,
        10 => KeyCode::Minus,
        11 => KeyCode::Equal,
        _ => KeyCode::Digit1,
    }
}
