//! UI: HUD (with icons), brewing panel, market / upgrades / recipe book panels,
//! day report, end screens and the pause overlay. All UI is built from the
//! shared kit in `ui_kit.rs`; input (mouse + keyboard) arrives as `UiAction`
//! messages from `actions.rs`.

use super::actions::{InputSet, UiAction};
use super::data::{MATERIALS, NUM_MATERIALS, RECIPES, REP_THRESHOLDS, SHELF_CAPACITY, UPGRADES, UpgradeId};
use super::resources::*;
use super::ui_kit::*;
use bevy::app::AppExit;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, refresh_buttons);
        app.add_systems(
            Update,
            (
                update_hud,
                update_brew_panel,
                update_market_panel,
                update_upgrades_panel,
                update_recipe_book,
                update_day_report,
                update_end_screen,
                pause_control,
                panel_navigation,
            )
                .after(InputSet),
        );
        app.add_systems(Update, (panel_visibility, update_pause_overlay));
    }
}

// Marker components ---------------------------------------------------------

#[derive(Component)]
struct HudGold;
#[derive(Component)]
struct HudRep;
#[derive(Component)]
struct HudDay;
#[derive(Component)]
struct HudIncome;
#[derive(Component)]
struct HudClock;

#[derive(Component)]
struct BrewCustomer;
#[derive(Component)]
struct BrewOrder;
#[derive(Component)]
struct BrewPatience;
#[derive(Component)]
struct BrewTempVal;
#[derive(Component)]
struct BrewStatus;
#[derive(Component)]
struct BrewBurn;
#[derive(Component)]
struct BrewProg;
#[derive(Component)]
struct BrewStir;
#[derive(Component)]
struct BrewCue;
#[derive(Component)]
struct BrewHint;
#[derive(Component)]
struct TempFill;
#[derive(Component)]
struct ProgFill;

#[derive(Component)]
struct MarketGold;
#[derive(Component)]
struct MarketQty;
#[derive(Component)]
struct MarketStock(usize);

#[derive(Component)]
struct UpgradeGold;
#[derive(Component)]
struct UpgradeLevel(usize);

#[derive(Component)]
struct RecipeLine(usize);

#[derive(Component)]
struct ReportText;
#[derive(Component)]
struct ReportGoal;
#[derive(Component)]
struct ReportBar(usize); // quality index fill

#[derive(Component)]
struct EndText;

/// Tutorial banner root (spawned once, driven by `tutorial.rs`).
#[derive(Component)]
pub struct TutorialRoot;
#[derive(Component)]
pub struct TutorialText;

#[derive(Component)]
struct Panel(GameScreen);

// Setup ---------------------------------------------------------------------

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|root| {
            panel(root, GameScreen::Title, |p| title_screen(p, &asset_server));
            panel(root, GameScreen::Playing, |p| {
                hud(p, &asset_server);
                brew_panel(p, &asset_server);
            });
            panel(root, GameScreen::Market, |p| market_panel(p, &asset_server));
            panel(root, GameScreen::Upgrades, |p| upgrades_panel(p, &asset_server));
            panel(root, GameScreen::RecipeBook, |p| {
                recipe_book_panel(p, &asset_server)
            });
            panel(root, GameScreen::DayReport, |p| {
                day_report_panel(p, &asset_server)
            });
            panel(root, GameScreen::GameOver, |p| game_over_panel(p, &asset_server));
            panel(root, GameScreen::Victory, |p| victory_panel(p, &asset_server));
            // Always-present overlays (visibility driven separately).
            pause_overlay(root, &asset_server);
            tutorial_banner(root, &asset_server);
        });
}

fn panel<F: FnOnce(&mut ChildSpawnerCommands)>(
    parent: &mut ChildSpawnerCommands,
    screen: GameScreen,
    build: F,
) {
    parent
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Panel(screen),
            Visibility::Hidden,
        ))
        .with_children(build);
}

// Title ---------------------------------------------------------------------

fn title_screen(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(10.0),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "✦ 炼金药铺 ✦", 56.0, C_GOLD, assets, true);
        ui_text(p, "2D 药铺模拟经营", 20.0, C_SOFT, assets, false);
        ui_text(p, "", 6.0, C_SOFT, assets, false);
        ui_text(
            p,
            "经营一家小药铺：进货原料、熬制药水、服务顾客，",
            18.0,
            C_TXT,
            assets,
            false,
        );
        ui_text(
            p,
            "用声望解锁新配方和更尊贵的客人。",
            18.0,
            C_TXT,
            assets,
            false,
        );
        ui_text(p, "", 8.0, C_SOFT, assets, false);
        ui_text(p, "操作", 18.0, C_GOLD, assets, true);
        ui_text(
            p,
            "点击按钮 或 Enter 接单 · Tab 切换面板 · ↑↓ 调温 · Space 搅拌",
            16.0,
            C_SOFT,
            assets,
            false,
        );
        ui_text(
            p,
            "Esc / P 暂停 · 鼠标点击全程可用",
            16.0,
            C_SOFT,
            assets,
            false,
        );
        ui_text(p, "", 14.0, C_SOFT, assets, false);
        spawn_button(p, ButtonKind::Start, "开门营业", Val::Px(240.0), 52.0, C_GOLD, assets);
        ui_text(p, "按 Enter 也可开始", 14.0, C_HINT, assets, false);
    });
}

// HUD -----------------------------------------------------------------------

fn hud(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: px(56.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(px(16.0), px(8.0)),
            border_radius: BorderRadius::all(px(0.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.92)),
        BorderColor::all(C_BORDER),
    ))
    .with_children(|p| {
        // Left group: gold / reputation / day.
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(18.0),
            ..default()
        })
        .with_children(|p| {
            hud_item(p, assets, "金", C_GOLD, "金币 0g", HudGold);
            hud_item(p, assets, "声", C_PURPLE, "声望 Lv1  0/20", HudRep);
            hud_item(p, assets, "日", C_GREEN, "第 1 天", HudDay);
        });
        // Right group: income / clock + nav buttons.
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14.0),
            ..default()
        })
        .with_children(|p| {
            hud_item(p, assets, "收", C_GREEN, "今日 0g", HudIncome);
            hud_item(p, assets, "时", C_BLUE, "剩余 75s", HudClock);
            p.spawn(Node {
                width: px(2.0),
                height: px(28.0),
                ..default()
            })
            .insert(BackgroundColor(C_BORDER));
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::Market), "市场", Val::Px(76.0), 34.0, C_BORDER_HI, assets);
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::Upgrades), "升级", Val::Px(76.0), 34.0, C_BORDER_HI, assets);
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::RecipeBook), "配方书", Val::Px(86.0), 34.0, C_BORDER_HI, assets);
            spawn_button(p, ButtonKind::Pause, "暂停", Val::Px(76.0), 34.0, C_BORDER_HI, assets);
        });
    });
}

fn hud_item<M: Component>(
    p: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    glyph: &str,
    color: Color,
    label: &str,
    marker: M,
) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(8.0),
        ..default()
    })
    .with_children(|p| {
        icon_badge(p, glyph, color, 26.0, assets);
        p.spawn((
            Text::new(label),
            TextFont {
                font: font_handle(assets).into(),
                font_size: FontSize::Px(16.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(C_TXT),
            marker,
        ));
    });
}

// Brew panel ----------------------------------------------------------------

fn brew_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: px(430.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            right: px(0.0),
            top: px(56.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(18.0), px(16.0)),
            row_gap: px(14.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.07, 0.12, 0.95)),
        BorderColor::all(C_BORDER),
    ))
    .with_children(|p| {
        // ---- 柜台 card ----
        card(p, 14.0, |p| {
            ui_text(p, "柜台", 20.0, C_GOLD, assets, true);
            marker_text(p, "排队：无", 16.0, C_TXT, assets, false, BrewCustomer);
            marker_text(p, "订单：-", 16.0, C_SOFT, assets, false, BrewOrder);
            // Patience bar
            p.spawn((
                Node {
                    width: percent(100.0),
                    height: px(12.0),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::MAX,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.06, 0.055, 0.09)),
                BorderColor::all(C_BORDER),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Node {
                        width: percent(0.0),
                        height: percent(100.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(C_GREEN),
                    BrewPatience,
                ));
            });
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|p| {
                spawn_button(p, ButtonKind::AcceptOrder, "接单", Val::Px(180.0), 40.0, C_GREEN, assets);
            });
        });
        // ---- 坩埚 card ----
        card(p, 14.0, |p| {
            ui_text(p, "坩埚", 20.0, C_GOLD, assets, true);
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|p| {
                ui_text(p, "温度", 14.0, C_SOFT, assets, false);
                marker_text(p, "--°", 20.0, C_TXT, assets, true, BrewTempVal);
                marker_text(p, "状态：待机", 14.0, C_SOFT, assets, false, BrewStatus);
            });
            progress_bar(p, 386.0, 20.0, Some((0.0, 100.0)), C_BORDER, TempFill);
            // Ticks
            p.spawn(Node {
                width: px(386.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|p| {
                for t in ["0°", "25°", "50°", "75°", "100°"] {
                    ui_text(p, t, 12.0, C_HINT, assets, false);
                }
            });
            marker_text(p, "", 15.0, C_RED, assets, true, BrewBurn);
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|p| {
                ui_text(p, "熬制进度", 14.0, C_SOFT, assets, false);
                marker_text(p, "0%", 16.0, C_TXT, assets, true, BrewProg);
            });
            progress_bar(p, 386.0, 18.0, None, C_BORDER, ProgFill);
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10.0),
                ..default()
            })
            .with_children(|p| {
                ui_text(p, "搅拌", 14.0, C_SOFT, assets, false);
                marker_text(p, "0/0", 15.0, C_SOFT, assets, false, BrewStir);
                p.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                marker_text(p, "", 16.0, C_BLUE, assets, true, BrewCue);
            });
            marker_text(p, "", 15.0, C_HINT, assets, false, BrewHint);
            // Control buttons row
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(10.0),
                ..default()
            })
            .with_children(|p| {
                spawn_button(p, ButtonKind::TempDown, "降温 ↓", Val::Px(120.0), 42.0, C_BLUE, assets);
                spawn_button(p, ButtonKind::Stir, "搅拌", Val::Px(120.0), 42.0, C_BLUE, assets);
                spawn_button(p, ButtonKind::TempUp, "升温 ↑", Val::Px(120.0), 42.0, C_BLUE, assets);
            });
        });
    });
}

fn marker_text<M: Component>(
    p: &mut ChildSpawnerCommands,
    s: &str,
    size: f32,
    color: Color,
    assets: &AssetServer,
    bold: bool,
    marker: M,
) {
    p.spawn((
        Text::new(s),
        TextFont {
            font: font_handle(assets).into(),
            font_size: FontSize::Px(size),
            weight: if bold {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            },
            ..default()
        },
        TextColor(color),
        marker,
    ));
}

// Market --------------------------------------------------------------------

fn market_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(16.0)),
            row_gap: px(10.0),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "市场 — 进货原料", 30.0, C_GOLD, assets, true);
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(12.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            ui_text(p, "每次购入", 15.0, C_SOFT, assets, false);
            spawn_button(p, ButtonKind::QtyDec, "−", Val::Px(34.0), 32.0, C_GOLD, assets);
            marker_text(p, "x1", 16.0, C_TXT, assets, true, MarketQty);
            spawn_button(p, ButtonKind::QtyInc, "+", Val::Px(34.0), 32.0, C_GOLD, assets);
            ui_text(p, "  [ ] / [ = ] 键也可调整", 13.0, C_HINT, assets, false);
        });
        // Material rows
        p.spawn((
            Node {
                width: px(880.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                overflow: Overflow::clip_y(),
                border_radius: BorderRadius::all(px(12.0)),
                ..default()
            },
            BackgroundColor(C_CARD),
            BorderColor::all(C_BORDER),
        ))
        .with_children(|p| {
            for i in 0..NUM_MATERIALS {
                let m = &MATERIALS[i];
                p.spawn((
                    Node {
                        width: percent(100.0),
                        height: px(32.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(12.0),
                        padding: UiRect::axes(px(12.0), px(0.0)),
                        ..default()
                    },
                    BackgroundColor(if i % 2 == 0 { C_CARD_ALT } else { C_CARD }),
                ))
                .with_children(|p| {
                    // color swatch
                    p.spawn((
                        Node {
                            width: px(18.0),
                            height: px(18.0),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(m.color),
                    ));
                    ui_text(p, m.name, 16.0, C_TXT, assets, false);
                    ui_text(p, format!("T{}", m.tier).as_str(), 13.0, C_HINT, assets, false);
                    ui_text(p, format!("{}g/份", m.cost).as_str(), 15.0, C_GOLD, assets, true);
                    p.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                    marker_text(p, "库存 0/6", 14.0, C_SOFT, assets, false, MarketStock(i));
                    spawn_button(p, ButtonKind::BuyMaterial(i), "购入", Val::Px(88.0), 30.0, C_GREEN, assets);
                });
            }
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(20.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            marker_text(p, "金币：0g", 18.0, C_GOLD, assets, true, MarketGold);
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::Playing), "返回柜台", Val::Px(160.0), 42.0, C_BORDER_HI, assets);
        });
    });
}

// Upgrades ------------------------------------------------------------------

fn upgrades_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(26.0)),
            row_gap: px(10.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "工坊升级", 30.0, C_GOLD, assets, true);
        p.spawn(Node {
            width: px(860.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(12.0),
            ..default()
        })
        .with_children(|p| {
            for i in 0..UPGRADES.len() {
                let def = &UPGRADES[i];
                card(p, 14.0, |p| {
                    p.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(12.0),
                        ..default()
                    })
                    .with_children(|p| {
                        ui_text(p, def.name, 18.0, C_TXT, assets, true);
                        marker_text(p, format!("等级 {}/{}", 0, def.max_level).as_str(), 14.0, C_SOFT, assets, false, UpgradeLevel(i));
                        p.spawn(Node {
                            flex_grow: 1.0,
                            ..default()
                        });
                        spawn_button(p, ButtonKind::BuyUpgrade(i), format!("{}g", def.costs[0]).as_str(), Val::Px(110.0), 36.0, C_GOLD, assets);
                    });
                    ui_text(p, def.desc, 14.0, C_HINT, assets, false);
                });
            }
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(20.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            marker_text(p, "金币：0g", 18.0, C_GOLD, assets, true, UpgradeGold);
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::Playing), "返回柜台", Val::Px(160.0), 42.0, C_BORDER_HI, assets);
        });
    });
}

// Recipe book ---------------------------------------------------------------

fn recipe_book_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(26.0)),
            row_gap: px(10.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "配方书", 30.0, C_GOLD, assets, true);
        ui_text(p, "声望提升后解锁更高阶配方。", 15.0, C_HINT, assets, false);
        p.spawn((
            Node {
                width: px(880.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                overflow: Overflow::clip_y(),
                border_radius: BorderRadius::all(px(12.0)),
                ..default()
            },
            BackgroundColor(C_CARD),
            BorderColor::all(C_BORDER),
        ))
        .with_children(|p| {
            for (idx, r) in RECIPES.iter().enumerate() {
                p.spawn((
                    Node {
                        width: percent(100.0),
                        height: px(40.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(10.0),
                        padding: UiRect::axes(px(12.0), px(0.0)),
                        ..default()
                    },
                    BackgroundColor(if idx % 2 == 0 { C_CARD_ALT } else { C_CARD }),
                ))
                .with_children(|p| {
                    p.spawn((
                        Node {
                            width: px(14.0),
                            height: px(14.0),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(r.color),
                    ));
                    marker_text(p, "", 15.0, C_TXT, assets, false, RecipeLine(idx));
                    p.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                });
            }
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(20.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, ButtonKind::OpenPanel(GameScreen::Playing), "返回柜台", Val::Px(160.0), 42.0, C_BORDER_HI, assets);
        });
    });
}

// Day report ----------------------------------------------------------------

fn day_report_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12.0),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "今日结算", 36.0, C_GOLD, assets, true);
        card(p, 20.0, |p| {
            marker_text(p, "", 19.0, C_TXT, assets, false, ReportText);
            // quality bar chart
            ui_text(p, "当日出品品质", 15.0, C_SOFT, assets, false);
            for qi in 0..4 {
                p.spawn(Node {
                    width: px(520.0),
                    height: px(18.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10.0),
                    ..default()
                })
                .with_children(|p| {
                    ui_text(p, quality_label(qi), 14.0, C_TXT, assets, false);
                    p.spawn((
                        Node {
                            width: px(360.0),
                            height: px(14.0),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::MAX,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.06, 0.055, 0.09)),
                        BorderColor::all(C_BORDER),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: percent(0.0),
                                height: percent(100.0),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(quality_color(qi)),
                            ReportBar(qi),
                        ));
                    });
                    marker_text(p, "0", 14.0, C_SOFT, assets, false, ReportBarCount(qi));
                });
            }
            marker_text(p, "", 16.0, C_BLUE, assets, true, ReportGoal);
        });
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(16.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, ButtonKind::Continue, "继续营业", Val::Px(180.0), 46.0, C_GREEN, assets);
            spawn_button(p, ButtonKind::Restart, "回标题", Val::Px(140.0), 46.0, C_BORDER_HI, assets);
        });
    });
}

fn quality_label(qi: usize) -> &'static str {
    match qi {
        0 => "劣质",
        1 => "普通",
        2 => "良好",
        _ => "完美",
    }
}
fn quality_color(qi: usize) -> Color {
    match qi {
        0 => Color::srgb(0.5, 0.5, 0.55),
        1 => C_BLUE,
        2 => C_GREEN,
        _ => C_GOLD,
    }
}

#[derive(Component)]
struct ReportBarCount(usize);

// End screens ---------------------------------------------------------------

fn game_over_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.09, 0.02, 0.02)),
    ))
    .with_children(|p| {
        ui_text(p, "破产了！", 48.0, C_RED, assets, true);
        ui_text(p, "租金交不起了，药铺关门了。", 18.0, C_SOFT, assets, false);
        marker_text(p, "", 20.0, C_TXT, assets, false, EndText);
        spawn_button(p, ButtonKind::Restart, "重新开始", Val::Px(200.0), 48.0, C_GOLD, assets);
        ui_text(p, "按 Enter 也可返回标题", 14.0, C_HINT, assets, false);
    });
}

fn victory_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(14.0),
            ..default()
        },
        BackgroundColor(C_BG),
    ))
    .with_children(|p| {
        ui_text(p, "炼金大师", 48.0, C_GOLD, assets, true);
        ui_text(p, "你的声望已成为传奇。", 20.0, C_SOFT, assets, false);
        marker_text(p, "", 20.0, C_TXT, assets, false, EndText);
        spawn_button(p, ButtonKind::Restart, "返回标题", Val::Px(200.0), 48.0, C_GOLD, assets);
        ui_text(p, "按 Enter 也可返回标题", 14.0, C_HINT, assets, false);
    });
}

// Pause overlay -------------------------------------------------------------

fn pause_overlay(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
        PausedOverlayRoot,
        Visibility::Hidden,
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                width: px(360.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(28.0)),
                row_gap: px(14.0),
                border: UiRect::all(px(2.0)),
                border_radius: BorderRadius::all(px(16.0)),
                ..default()
            },
            BackgroundColor(C_CARD),
            BorderColor::all(C_GOLD),
        ))
        .with_children(|p| {
            ui_text(p, "暂停", 30.0, C_GOLD, assets, true);
            spawn_button(p, ButtonKind::Resume, "继续营业", Val::Px(240.0), 46.0, C_GREEN, assets);
            spawn_button(p, ButtonKind::Restart, "返回标题", Val::Px(240.0), 46.0, C_BORDER_HI, assets);
            spawn_button(p, ButtonKind::Quit, "退出游戏", Val::Px(240.0), 46.0, C_RED, assets);
            ui_text(p, "Esc / P 继续", 14.0, C_HINT, assets, false);
        });
    });
}

// Tutorial banner -----------------------------------------------------------

fn tutorial_banner(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            position_type: PositionType::Absolute,
            bottom: px(18.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,
        TutorialRoot,
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                padding: UiRect::axes(px(22.0), px(12.0)),
                border: UiRect::all(px(2.0)),
                border_radius: BorderRadius::all(px(14.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.09, 0.16, 0.96)),
            BorderColor::all(C_GOLD),
        ))
        .with_children(|p| {
            icon_badge(p, "!", C_GOLD, 26.0, assets);
            marker_text(p, "", 16.0, C_TXT, assets, false, TutorialText);
        });
    });
}

// Visibility ----------------------------------------------------------------

/// Navigate between gameplay screens (market / upgrades / recipe book / back).
/// `OpenPanel` is written by the HUD buttons and the Tab key; this system is
/// the single place that turns it into a real state transition.
fn panel_navigation(
    mut actions: MessageReader<UiAction>,
    mut next: ResMut<NextState<GameScreen>>,
) {
    for a in actions.read() {
        if let UiAction::OpenPanel(p) = a {
            next.set(*p);
        }
    }
}

fn panel_visibility(state: Res<State<GameScreen>>, mut q: Query<(&Panel, &mut Visibility)>) {
    let cur = *state.get();
    for (panel, mut vis) in &mut q {
        *vis = if panel.0 == cur {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_pause_overlay(paused: Res<Paused>, mut q: Query<&mut Visibility, With<PausedOverlayRoot>>) {
    if let Ok(mut v) = q.single_mut() {
        *v = if paused.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// Actions (pause / quit / navigation) --------------------------------------

fn pause_control(
    mut actions: MessageReader<UiAction>,
    mut paused: ResMut<Paused>,
    mut next: ResMut<NextState<GameScreen>>,
    mut exit: MessageWriter<AppExit>,
) {
    for a in actions.read() {
        match a {
            UiAction::Pause => paused.0 = true,
            UiAction::Resume => paused.0 = false,
            UiAction::Restart => {
                paused.0 = false;
                next.set(GameScreen::Title);
            }
            UiAction::Quit => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    }
}

// Updates -------------------------------------------------------------------

fn update_hud(
    econ: Res<Economy>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudGold>>,
        Query<&mut Text, With<HudRep>>,
        Query<&mut Text, With<HudDay>>,
        Query<&mut Text, With<HudIncome>>,
        Query<&mut Text, With<HudClock>>,
    )>,
) {
    texts.p0().single_mut().unwrap().0 = format!("金币 {}g", econ.gold);
    let rep_next = if econ.rep_level >= 10 {
        "MAX".to_string()
    } else {
        format!(
            "{}/{}",
            econ.reputation, REP_THRESHOLDS[econ.rep_level as usize]
        )
    };
    texts.p1().single_mut().unwrap().0 = format!("声望 Lv{}  {}", econ.rep_level, rep_next);
    texts.p2().single_mut().unwrap().0 = format!("第 {} 天", econ.day);
    texts.p3().single_mut().unwrap().0 = format!("今日 {}g", econ.day_income);
    texts.p4().single_mut().unwrap().0 = format!("剩余 {:.0}s", econ.day_length - econ.day_elapsed);
}

fn update_brew_panel(
    brewing: Res<Brewing>,
    up: Res<UpgradesState>,
    customers: Query<&Customer>,
    time: Res<Time>,
    mut texts: ParamSet<(
        Query<&mut Text, (With<BrewCustomer>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewOrder>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewProg>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewStir>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewCue>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewStatus>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewTempVal>, Without<BrewHint>)>,
        Query<&mut Text, (With<BrewBurn>, Without<BrewHint>)>,
    )>,
    mut hint: Query<&mut Text, With<BrewHint>>,
    mut nodes: ParamSet<(
        Query<&mut Node, With<BrewPatience>>,
        Query<&mut Node, (With<TempFill>, Without<ProgFill>)>,
        Query<&mut Node, (With<ProgFill>, Without<TempFill>)>,
        Query<(&mut Node, &mut BackgroundColor), With<BarWindow>>,
    )>,
) {
    let pulse = (time.elapsed_secs() * 6.0).sin() * 0.5 + 0.5;

    // Front customer info + patience.
    let mut front: Option<&Customer> = None;
    for c in customers.iter() {
        if c.state == CustomerState::Waiting || c.state == CustomerState::Served {
            match front {
                Some(f) if f.queue_slot > c.queue_slot => front = Some(c),
                Some(_) => {}
                None => front = Some(c),
            }
        }
    }

    if let Some(c) = front {
        let r = &RECIPES[c.recipe_idx];
        let kind_name = &super::data::CUSTOMER_KINDS[c.kind_idx].name;
        texts.p0().single_mut().unwrap().0 = format!("排队：{}（{}g）", kind_name, c.budget);
        match c.state {
            CustomerState::Waiting => {
                texts.p1().single_mut().unwrap().0 = format!("订单：{} 想要 {}", kind_name, r.name);
            }
            CustomerState::Served => {
                texts.p1().single_mut().unwrap().0 = format!("{} 在等 {}", kind_name, r.name);
            }
            _ => {}
        }
        let ratio = (c.patience / c.patience_max).clamp(0.0, 1.0);
        if let Ok(mut n) = nodes.p0().single_mut() {
            n.width = percent(ratio * 100.0);
        }
    } else {
        texts.p0().single_mut().unwrap().0 = "排队：无".to_string();
        texts.p1().single_mut().unwrap().0 = "订单：-".to_string();
        if let Ok(mut n) = nodes.p0().single_mut() {
            n.width = percent(0.0);
        }
    }

    if brewing.active {
        let r = &RECIPES[brewing.recipe_idx];
        let clvl = up.level(UpgradeId::Cauldron);
        let lo = (r.temp_min - 4.0 * clvl as f32).clamp(0.0, 100.0);
        let hi = (r.temp_max + 4.0 * clvl as f32).clamp(0.0, 100.0);

        texts.p2().single_mut().unwrap().0 = format!("{:.0}%", brewing.progress);
        let hits = brewing.stir_hits.iter().filter(|&&h| h).count();
        texts.p3().single_mut().unwrap().0 = format!("{}/{}", hits, r.stir_points);
        texts.p5().single_mut().unwrap().0 = brew_status(&brewing, r).0;
        texts.p6().single_mut().unwrap().0 = format!("{:.0}°", brewing.temp);

        // Burn warning pulse.
        let burn_warn = brewing.temp > r.temp_max + 8.0;
        texts.p7().single_mut().unwrap().0 = if brewing.burnt {
            "烧焦了！这锅毁了".to_string()
        } else if burn_warn {
            "！小心烧焦！马上降温".to_string()
        } else {
            "".to_string()
        };

        // Stir cue inside an active window.
        if let Some(at) = active_stir_window(&brewing) {
            let cue = format!("● 现在搅拌！（{}%）", at as i32);
            texts.p4().single_mut().unwrap().0 = cue;
        } else {
            texts.p4().single_mut().unwrap().0 = "".to_string();
        }

        if let Ok(mut n) = nodes.p1().single_mut() {
            n.width = percent(brewing.temp);
        }
        if let Ok(mut n) = nodes.p2().single_mut() {
            n.width = percent(brewing.progress);
        }
        if let Ok((mut n, mut bg)) = nodes.p3().single_mut() {
            n.left = percent(lo);
            n.width = percent((hi - lo).max(4.0));
            let inwin = brewing.temp >= lo && brewing.temp <= hi;
            bg.0 = if inwin {
                Color::srgba(0.976, 0.780, 0.310, 0.55 + 0.25 * pulse)
            } else {
                Color::srgba(0.976, 0.780, 0.310, 0.20)
            };
        }
    } else {
        texts.p2().single_mut().unwrap().0 = "0%".to_string();
        texts.p3().single_mut().unwrap().0 = "0/0".to_string();
        texts.p4().single_mut().unwrap().0 = "".to_string();
        texts.p5().single_mut().unwrap().0 = "状态：待机".to_string();
        texts.p6().single_mut().unwrap().0 = "--°".to_string();
        texts.p7().single_mut().unwrap().0 = "".to_string();
        if let Ok(mut n) = nodes.p1().single_mut() {
            n.width = percent(0.0);
        }
        if let Ok(mut n) = nodes.p2().single_mut() {
            n.width = percent(0.0);
        }
        if let Ok((mut n, _bg)) = nodes.p3().single_mut() {
            n.left = percent(0.0);
            n.width = percent(0.0);
        }
    }

    // Bottom hint.
    if !brewing.active {
        if front.is_some() {
            hint.single_mut().unwrap().0 = "点「接单」或按 Enter 接下订单".to_string();
        } else {
            hint.single_mut().unwrap().0 = "等待顾客上门…".to_string();
        }
    } else {
        hint.single_mut().unwrap().0 = "↑↓ 调温，Space 或点「搅拌」".to_string();
    }
}

/// Returns the progress % where the currently-active stir window begins.
fn active_stir_window(brewing: &Brewing) -> Option<f32> {
    let points = brewing.stir_hits.len();
    if points == 0 {
        return None;
    }
    for i in 0..points {
        if brewing.stir_hits[i] {
            continue;
        }
        let at = 100.0 * (i as f32 + 0.5) / points as f32;
        if brewing.progress >= at && brewing.progress <= at + 20.0 {
            return Some(at);
        }
        if brewing.progress > at + 20.0 {
            continue;
        }
    }
    None
}

fn brew_status(brewing: &Brewing, r: &super::data::RecipeDef) -> (String, Color) {
    if brewing.burnt {
        return ("已烧焦".to_string(), C_RED);
    }
    if brewing.temp > r.temp_max + 8.0 {
        return ("过热！".to_string(), C_RED);
    }
    if brewing.temp < r.temp_min - 15.0 {
        return ("过冷".to_string(), C_BLUE);
    }
    if brewing.temp < r.temp_min || brewing.temp > r.temp_max {
        return ("温度偏离".to_string(), C_GOLD);
    }
    ("温度合适 ✓".to_string(), C_GREEN)
}

fn update_market_panel(
    econ: Res<Economy>,
    inv: Res<Inventory>,
    up: Res<UpgradesState>,
    mut params: ParamSet<(
        Query<&mut Text, With<MarketGold>>,
        Query<&mut Text, With<MarketQty>>,
        Query<(&MarketStock, &mut Text)>,
    )>,
) {
    let cap = SHELF_CAPACITY[up.level(UpgradeId::Shelf) as usize];
    if let Ok(mut g) = params.p0().single_mut() {
        g.0 = format!("金币：{}g", econ.gold);
    }
    if let Ok(mut q) = params.p1().single_mut() {
        q.0 = format!("x{}", inv.restock_qty);
    }
    for (stock, mut text) in &mut params.p2() {
        let i = stock.0;
        let have = inv.counts[i];
        let full = have >= cap;
        text.0 = if full {
            format!("已满 {}/{}", have, cap)
        } else {
            format!("库存 {}/{}", have, cap)
        };
    }
}

fn update_upgrades_panel(
    econ: Res<Economy>,
    up: Res<UpgradesState>,
    mut params: ParamSet<(
        Query<&mut Text, With<UpgradeGold>>,
        Query<(&UpgradeLevel, &mut Text)>,
    )>,
) {
    if let Ok(mut gg) = params.p0().single_mut() {
        gg.0 = format!("金币：{}g", econ.gold);
    }
    for (lv, mut text) in &mut params.p1() {
        let i = lv.0;
        let def = &UPGRADES[i];
        let l = up.levels[i];
        text.0 = format!("等级 {}/{}", l, def.max_level);
    }
}

fn update_recipe_book(
    econ: Res<Economy>,
    inv: Res<Inventory>,
    mut lines: Query<(&RecipeLine, &mut Text)>,
) {
    for (line, mut text) in &mut lines {
        let r = &RECIPES[line.0];
        let unlocked = r.tier <= econ.rep_level;
        let mats: Vec<String> = r
            .mats
            .iter()
            .map(|&m| {
                let have = inv.counts[m as usize] >= 1;
                format!(
                    "{}{}",
                    if have { "" } else { "?" },
                    MATERIALS[m as usize].name
                )
            })
            .collect();
        let prefix = if unlocked { "[可炼]" } else { "[未解锁]" };
        text.0 = format!(
            "{} {}  售价{}g  温 {}-{}°  {}s  {}",
            prefix,
            r.name,
            r.base_price,
            r.temp_min as i32,
            r.temp_max as i32,
            r.brew_time as i32,
            mats.join(" + ")
        );
    }
}

fn update_day_report(
    econ: Res<Economy>,
    mut params: ParamSet<(
        Query<&mut Text, With<ReportText>>,
        Query<&mut Text, With<ReportGoal>>,
        Query<(&ReportBar, &mut Node)>,
        Query<(&ReportBarCount, &mut Text)>,
    )>,
) {
    params.p0().single_mut().unwrap().0 = format!(
        "今日收入：{}g    租金：-{}g    净收入：{}g\n\n服务顾客：{}    流失：{}    完美出品：{}\n声望：{}（Lv {}）    金币：{}g",
        econ.day_income,
        econ.rent,
        econ.day_income.saturating_sub(econ.rent),
        econ.served,
        econ.lost,
        econ.perfect_count,
        econ.reputation,
        econ.rep_level,
        econ.gold
    );
    params.p1().single_mut().unwrap().0 = next_day_goal(&econ);

    let max_q = econ.day_quality.iter().copied().max().unwrap_or(0).max(1);
    for (bar, mut n) in &mut params.p2() {
        let q = econ.day_quality[bar.0];
        n.width = percent(q as f32 / max_q as f32 * 100.0);
    }
    for (c, mut text) in &mut params.p3() {
        text.0 = econ.day_quality[c.0].to_string();
    }
}

/// A concrete, actionable goal for the next day.
fn next_day_goal(econ: &Economy) -> String {
    if econ.rep_level >= 10 {
        "你已经通关！声望达到巅峰。".to_string()
    } else if econ.rep_level < 3 {
        format!(
            "明日目标：声望升到 Lv{}（还差 {} 点）——卖品质更好的药水赚声望",
            econ.rep_level + 1,
            REP_THRESHOLDS[econ.rep_level as usize].saturating_sub(econ.reputation)
        )
    } else {
        format!(
            "明日目标：声望升到 Lv{}，并赚够 {}g 交租金",
            econ.rep_level + 1,
            econ.rent + 1
        )
    }
}

fn update_end_screen(econ: Res<Economy>, mut t: Query<&mut Text, With<EndText>>) {
    let txt = format!(
        "坚持到第 {} 天 · 服务顾客 {} · 流失 {}\n完美出品 {} · 声望 {}",
        econ.day, econ.served, econ.lost, econ.perfect_count, econ.reputation
    );
    for mut text in &mut t {
        text.0 = txt.clone();
    }
}
