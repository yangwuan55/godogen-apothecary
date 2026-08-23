//! Purchase logic for the Market and Upgrades panels. Input arrives as
//! `UiAction` messages from `actions.rs` (mouse clicks or number keys).

use super::actions::{InputSet, UiAction};
use super::audio::SfxRequest;
use super::data::{MATERIALS, SHELF_CAPACITY, UPGRADES, UpgradeId};
use super::resources::{Economy, FxEvent, FxKind, GameScreen, Inventory, Paused, UpgradesState};
use bevy::prelude::*;

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            market_purchase
                .after(InputSet)
                .run_if(in_state(GameScreen::Market))
                .run_if(not_paused),
        );
        app.add_systems(
            Update,
            upgrade_purchase
                .after(InputSet)
                .run_if(in_state(GameScreen::Upgrades))
                .run_if(not_paused),
        );
    }
}

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

/// Buy materials / adjust restock quantity.
fn market_purchase(
    mut actions: MessageReader<UiAction>,
    mut econ: ResMut<Economy>,
    mut inv: ResMut<Inventory>,
    up: Res<UpgradesState>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxRequest>,
) {
    let capacity = SHELF_CAPACITY[up.level(UpgradeId::Shelf) as usize];
    for a in actions.read() {
        match a {
            UiAction::QtyInc => {
                inv.restock_qty = (inv.restock_qty + 1).min(10);
            }
            UiAction::QtyDec => {
                inv.restock_qty = inv.restock_qty.saturating_sub(1).max(1);
            }
            UiAction::BuyMaterial(i) => {
                let i = *i;
                let mat = &MATERIALS[i];
                if inv.counts[i] >= capacity {
                    fx.write(FxEvent {
                        kind: FxKind::Fizzle,
                        pos: Vec2::new(300.0, 200.0),
                        text: Some("货架已满！".into()),
                    });
                    sfx.write(SfxRequest::Error);
                    continue;
                }
                let qty = inv.restock_qty;
                let cost = mat.cost * qty;
                if econ.gold >= cost {
                    econ.gold -= cost;
                    inv.counts[i] = (inv.counts[i] + qty).min(capacity);
                    econ.purchases += 1;
                    fx.write(FxEvent {
                        kind: FxKind::GoldText,
                        pos: Vec2::new(300.0, 200.0),
                        text: Some(format!("购入 {} x{}", mat.name, qty)),
                    });
                    sfx.write(SfxRequest::Coin);
                } else {
                    fx.write(FxEvent {
                        kind: FxKind::Fizzle,
                        pos: Vec2::new(300.0, 200.0),
                        text: Some("金币不足！".into()),
                    });
                    sfx.write(SfxRequest::Error);
                }
            }
            _ => {}
        }
    }
}

/// Buy upgrades.
fn upgrade_purchase(
    mut actions: MessageReader<UiAction>,
    mut econ: ResMut<Economy>,
    mut up: ResMut<UpgradesState>,
    mut fx: MessageWriter<FxEvent>,
    mut sfx: MessageWriter<SfxRequest>,
) {
    for a in actions.read() {
        if let UiAction::BuyUpgrade(i) = a {
            let i = *i;
            let def = &UPGRADES[i];
            let lvl = up.levels[i] as usize;
            if lvl >= def.max_level as usize {
                fx.write(FxEvent {
                    kind: FxKind::Fizzle,
                    pos: Vec2::new(300.0, 300.0),
                    text: Some(format!("{} 已满级", def.name)),
                });
                sfx.write(SfxRequest::Error);
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
                sfx.write(SfxRequest::Coin);
            } else {
                fx.write(FxEvent {
                    kind: FxKind::Fizzle,
                    pos: Vec2::new(300.0, 300.0),
                    text: Some("金币不足！".into()),
                });
                sfx.write(SfxRequest::Error);
            }
        }
    }
}
