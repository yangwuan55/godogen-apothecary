//! UI: HUD, brewing panel, market, upgrades, recipe book, day report, end screens.

use super::data::{MATERIALS, NUM_MATERIALS, RECIPES, SHELF_CAPACITY, UPGRADES, UpgradeId};
use super::resources::*;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(
            Update,
            update_hud_text.run_if(in_state(GameScreen::Playing)),
        );
        app.add_systems(
            Update,
            update_brew_panel.run_if(in_state(GameScreen::Playing)),
        );
        app.add_systems(Update, panel_visibility);
        app.add_systems(
            Update,
            update_day_report.run_if(in_state(GameScreen::DayReport)),
        );
        app.add_systems(
            Update,
            update_market_panel.run_if(in_state(GameScreen::Market)),
        );
        app.add_systems(
            Update,
            update_upgrades_panel.run_if(in_state(GameScreen::Upgrades)),
        );
        app.add_systems(
            Update,
            update_recipe_book.run_if(in_state(GameScreen::RecipeBook)),
        );
        app.add_systems(
            Update,
            update_end_screen
                .run_if(in_state(GameScreen::GameOver).or_else(in_state(GameScreen::Victory))),
        );
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
struct TempFill;
#[derive(Component)]
struct ProgFill;
#[derive(Component)]
struct BrewStir;
#[derive(Component)]
struct BrewStatus;
#[derive(Component)]
struct BrewHint;

#[derive(Component)]
struct MarketList;
#[derive(Component)]
struct UpgradeList;
#[derive(Component)]
struct RecipeList;
#[derive(Component)]
struct ReportText;
#[derive(Component)]
struct EndText;

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
            panel(root, GameScreen::Upgrades, |p| {
                upgrades_panel(p, &asset_server)
            });
            panel(root, GameScreen::RecipeBook, |p| {
                recipe_book_panel(p, &asset_server)
            });
            panel(root, GameScreen::DayReport, |p| {
                day_report_panel(p, &asset_server)
            });
            panel(root, GameScreen::GameOver, |p| {
                game_over_panel(p, &asset_server)
            });
            panel(root, GameScreen::Victory, |p| {
                victory_panel(p, &asset_server)
            });
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

// Panels --------------------------------------------------------------------

fn title_screen(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn(Node {
        width: percent(100.0),
        height: percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: px(12.0),
        ..default()
    })
    .with_children(|p| {
        text(
            p,
            "The Alchemist's Apothecary",
            FontSize::Px(52.0),
            TEXT_GOLD,
            assets,
            true,
        );
        text(
            p,
            "A 2D potion-shop management sim",
            FontSize::Px(22.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(p, "", FontSize::Px(8.0), TEXT_SOFT, assets, false);
        text(
            p,
            "Restock herbs, brew potions, serve customers.",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(
            p,
            "Rise in reputation to unlock recipes and richer clients.",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(
            p,
            "Perfect brews earn more gold and reputation.",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(p, "", FontSize::Px(8.0), TEXT_SOFT, assets, false);
        text(
            p,
            "Tab: panels   Enter: accept order   ↑/↓: temperature   Space: stir",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(
            p,
            "Press Enter to open the shop",
            FontSize::Px(24.0),
            TEXT_GOLD,
            assets,
            true,
        );
    });
}

fn hud(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn(Node {
        width: percent(100.0),
        height: px(44.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::axes(px(16.0), px(6.0)),
        ..default()
    })
    .with_children(|p| {
        text_marker(
            p,
            "[ GOLD ] 0g",
            FontSize::Px(20.0),
            TEXT_GOLD,
            assets,
            true,
            HudGold,
        );
        text_marker(
            p,
            "Rep Lv 1",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
            HudRep,
        );
        text_marker(
            p,
            "Day 1",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
            HudDay,
        );
        text_marker(
            p,
            "Income 0g",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
            HudIncome,
        );
        text_marker(
            p,
            "75s",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
            HudClock,
        );
    });
}

fn brew_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: px(400.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            right: px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(20.0), px(14.0)),
            row_gap: px(10.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.07, 0.12, 0.94)),
    ))
    .with_children(|p| {
        text(p, "Counter", FontSize::Px(24.0), TEXT_GOLD, assets, true);
        text_marker(
            p,
            "Waiting: nobody",
            FontSize::Px(17.0),
            TEXT_SOFT,
            assets,
            false,
            BrewCustomer,
        );
        text_marker(
            p,
            "Order: -",
            FontSize::Px(17.0),
            TEXT_SOFT,
            assets,
            false,
            BrewOrder,
        );
        text(p, "", FontSize::Px(6.0), TEXT_SOFT, assets, false);
        text(p, "Cauldron", FontSize::Px(20.0), TEXT_GOLD, assets, true);
        text(
            p,
            "Temperature",
            FontSize::Px(14.0),
            TEXT_SOFT,
            assets,
            false,
        );
        bar(p, TempFill);
        text(
            p,
            "Brew progress",
            FontSize::Px(14.0),
            TEXT_SOFT,
            assets,
            false,
        );
        bar(p, ProgFill);
        text_marker(
            p,
            "Stir: -",
            FontSize::Px(15.0),
            TEXT_SOFT,
            assets,
            false,
            BrewStir,
        );
        text_marker(
            p,
            "Status: idle",
            FontSize::Px(15.0),
            TEXT_SOFT,
            assets,
            false,
            BrewStatus,
        );
        text_marker(
            p,
            "",
            FontSize::Px(14.0),
            TEXT_HINT,
            assets,
            false,
            BrewHint,
        );
    });
}

fn bar<M: Component>(p: &mut ChildSpawnerCommands, _m: M) {
    p.spawn(Node {
        width: px(350.0),
        height: px(16.0),
        border: UiRect::all(px(2.0)),
        border_radius: BorderRadius::MAX,
        ..default()
    })
    .with_children(|bar| {
        bar.spawn((
            Node {
                width: percent(0.0),
                height: percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgb(0.35, 0.75, 0.4)),
            _m,
        ));
    });
}

fn market_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(30.0)),
            row_gap: px(10.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.98)),
    ))
    .with_children(|p| {
        text(
            p,
            "Market — buy ingredients",
            FontSize::Px(30.0),
            TEXT_GOLD,
            assets,
            true,
        );
        text(
            p,
            "Press 1-9 / 0 / - / = to buy. Tab to leave.",
            FontSize::Px(15.0),
            TEXT_HINT,
            assets,
            false,
        );
        text_marker(
            p,
            "",
            FontSize::Px(16.0),
            TEXT_SOFT,
            assets,
            false,
            MarketList,
        );
    });
}

fn upgrades_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(30.0)),
            row_gap: px(10.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.98)),
    ))
    .with_children(|p| {
        text(
            p,
            "Workshop Upgrades",
            FontSize::Px(30.0),
            TEXT_GOLD,
            assets,
            true,
        );
        text(
            p,
            "Press 1-4 to buy. Tab to leave.",
            FontSize::Px(15.0),
            TEXT_HINT,
            assets,
            false,
        );
        text_marker(
            p,
            "",
            FontSize::Px(16.0),
            TEXT_SOFT,
            assets,
            false,
            UpgradeList,
        );
    });
}

fn recipe_book_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
    p.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(40.0), px(30.0)),
            row_gap: px(10.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.98)),
    ))
    .with_children(|p| {
        text(
            p,
            "Recipe Book",
            FontSize::Px(30.0),
            TEXT_GOLD,
            assets,
            true,
        );
        text(
            p,
            "Unlocked recipes appear as reputation rises.",
            FontSize::Px(15.0),
            TEXT_HINT,
            assets,
            false,
        );
        text_marker(
            p,
            "",
            FontSize::Px(15.0),
            TEXT_SOFT,
            assets,
            false,
            RecipeList,
        );
    });
}

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
        BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.98)),
    ))
    .with_children(|p| {
        text(p, "Day Report", FontSize::Px(36.0), TEXT_GOLD, assets, true);
        text_marker(
            p,
            "",
            FontSize::Px(20.0),
            TEXT_SOFT,
            assets,
            false,
            ReportText,
        );
        text(
            p,
            "Press Enter to continue",
            FontSize::Px(18.0),
            TEXT_HINT,
            assets,
            false,
        );
    });
}

fn game_over_panel(p: &mut ChildSpawnerCommands, assets: &AssetServer) {
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
        BackgroundColor(Color::srgba(0.08, 0.02, 0.02, 0.98)),
    ))
    .with_children(|p| {
        text(
            p,
            "Bankrupt",
            FontSize::Px(46.0),
            Color::srgb(0.95, 0.3, 0.3),
            assets,
            true,
        );
        text_marker(p, "", FontSize::Px(20.0), TEXT_SOFT, assets, false, EndText);
        text(
            p,
            "The shop closes its doors.",
            FontSize::Px(18.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(
            p,
            "Press Enter to return to title",
            FontSize::Px(18.0),
            TEXT_HINT,
            assets,
            false,
        );
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
            row_gap: px(12.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.06, 0.10, 0.98)),
    ))
    .with_children(|p| {
        text(
            p,
            "Master Alchemist",
            FontSize::Px(46.0),
            TEXT_GOLD,
            assets,
            true,
        );
        text_marker(p, "", FontSize::Px(20.0), TEXT_SOFT, assets, false, EndText);
        text(
            p,
            "Your reputation is the stuff of legend.",
            FontSize::Px(20.0),
            TEXT_SOFT,
            assets,
            false,
        );
        text(
            p,
            "Press Enter to return to title",
            FontSize::Px(18.0),
            TEXT_HINT,
            assets,
            false,
        );
    });
}

// Helpers -------------------------------------------------------------------

const TEXT_GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT_SOFT: Color = Color::srgb(0.82, 0.84, 0.90);
const TEXT_HINT: Color = Color::srgb(0.55, 0.62, 0.72);

fn font_handle(assets: &AssetServer, bold: bool) -> Handle<bevy::text::Font> {
    if bold {
        assets.load("fonts/FiraSans-Bold.ttf")
    } else {
        assets.load("fonts/FiraSans-Regular.ttf")
    }
}

fn text(
    p: &mut ChildSpawnerCommands,
    s: &str,
    size: FontSize,
    color: Color,
    assets: &AssetServer,
    bold: bool,
) {
    p.spawn((
        Text::new(s),
        TextFont {
            font: font_handle(assets, bold).into(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}

fn text_marker<M: Component>(
    p: &mut ChildSpawnerCommands,
    s: &str,
    size: FontSize,
    color: Color,
    assets: &AssetServer,
    bold: bool,
    _marker: M,
) {
    p.spawn((
        Text::new(s),
        TextFont {
            font: font_handle(assets, bold).into(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        _marker,
    ));
}

// Visibility ----------------------------------------------------------------

fn panel_visibility(state: Res<State<GameScreen>>, mut q: Query<(&Panel, &mut Visibility)>) {
    let cur = *state.get();
    for (panel, mut vis) in &mut q {
        // Only the active screen's panel is visible; everything else must be
        // explicitly Hidden, or the opaque panels would all render on top of
        // each other (and of the world).
        *vis = if panel.0 == cur {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// Updates -------------------------------------------------------------------

fn update_hud_text(
    econ: Res<Economy>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudGold>>,
        Query<&mut Text, With<HudRep>>,
        Query<&mut Text, With<HudDay>>,
        Query<&mut Text, With<HudIncome>>,
        Query<&mut Text, With<HudClock>>,
    )>,
) {
    texts.p0().single_mut().unwrap().0 = format!("[ GOLD ] {}g", econ.gold);
    texts.p1().single_mut().unwrap().0 = format!("Rep Lv {}  {}", econ.rep_level, econ.reputation);
    texts.p2().single_mut().unwrap().0 = format!("Day {}", econ.day);
    texts.p3().single_mut().unwrap().0 = format!("Income {}g", econ.day_income);
    texts.p4().single_mut().unwrap().0 = format!("{:.0}s", econ.day_length - econ.day_elapsed);
}

fn update_brew_panel(
    brewing: Res<Brewing>,
    customers: Query<&Customer>,
    mut texts: ParamSet<(
        Query<&mut Text, With<BrewOrder>>,
        Query<&mut Text, With<BrewCustomer>>,
        Query<&mut Text, With<BrewStir>>,
        Query<&mut Text, With<BrewStatus>>,
        Query<&mut Text, With<BrewHint>>,
    )>,
    mut fills: ParamSet<(
        Query<&mut Node, With<TempFill>>,
        Query<&mut Node, With<ProgFill>>,
    )>,
) {
    // Front customer.
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
        texts.p1().single_mut().unwrap().0 = format!("{} ({}g)", kind_name, c.budget);
        match c.state {
            CustomerState::Waiting => {
                texts.p0().single_mut().unwrap().0 = format!("{} wants {}", kind_name, r.name);
                texts.p4().single_mut().unwrap().0 = "Press Enter to accept the order".to_string();
            }
            CustomerState::Served => {
                texts.p0().single_mut().unwrap().0 = format!("{} waits for {}", kind_name, r.name);
                texts.p4().single_mut().unwrap().0 = "Brew: ↑/↓ temp, Space to stir".to_string();
            }
            _ => {}
        }
    } else {
        texts.p1().single_mut().unwrap().0 = "Waiting: nobody".to_string();
        texts.p0().single_mut().unwrap().0 = "Order: -".to_string();
        texts.p4().single_mut().unwrap().0 = "".to_string();
    }

    // Brewing state.
    if brewing.active {
        let r = &RECIPES[brewing.recipe_idx];
        texts.p2().single_mut().unwrap().0 = format!(
            "Stir: {}/{}",
            brewing.stir_hits.iter().filter(|&&h| h).count(),
            r.stir_points
        );
        let (status_txt, color) = brew_status(&brewing, r);
        texts.p3().single_mut().unwrap().0 = status_txt;
        let _ = color;
        if let Ok(mut tf) = fills.p0().single_mut() {
            tf.width = percent(brewing.temp);
        }
        if let Ok(mut pf) = fills.p1().single_mut() {
            pf.width = percent(brewing.progress);
        }
    } else {
        texts.p2().single_mut().unwrap().0 = "Stir: -".to_string();
        texts.p3().single_mut().unwrap().0 = "Status: idle".to_string();
        if let Ok(mut tf) = fills.p0().single_mut() {
            tf.width = percent(0.0);
        }
        if let Ok(mut pf) = fills.p1().single_mut() {
            pf.width = percent(0.0);
        }
    }
}

fn brew_status(brewing: &Brewing, r: &super::data::RecipeDef) -> (String, Color) {
    if brewing.burnt {
        return ("Status: BURNT".to_string(), Color::srgb(0.95, 0.3, 0.3));
    }
    if brewing.temp > r.temp_max + 18.0 {
        return ("Status: too hot!".to_string(), Color::srgb(0.95, 0.4, 0.3));
    }
    if brewing.temp < r.temp_min - 15.0 {
        return ("Status: too cold".to_string(), Color::srgb(0.5, 0.7, 0.95));
    }
    ("Status: in window".to_string(), Color::srgb(0.4, 0.85, 0.5))
}

fn update_day_report(econ: Res<Economy>, mut t: Query<&mut Text, With<ReportText>>) {
    t.single_mut().unwrap().0 = format!(
        "Income today: {}g\nRent: -{}g\nNet: {}g\n\nServed: {}   Lost: {}\nPerfect brews: {}\nReputation: {}  (Lv {})\n\nGold: {}g",
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
}

fn update_market_panel(
    econ: Res<Economy>,
    inv: Res<Inventory>,
    up: Res<UpgradesState>,
    mut t: Query<&mut Text, With<MarketList>>,
) {
    let cap = SHELF_CAPACITY[up.level(UpgradeId::Shelf) as usize];
    let mut s = String::new();
    for i in 0..NUM_MATERIALS {
        let m = &MATERIALS[i];
        s.push_str(&format!(
            "  {}  {} {}g  [stock {}/{}]  {}\n",
            key_label(i),
            m.emoji,
            m.cost,
            inv.counts[i],
            cap,
            m.name
        ));
    }
    s.push_str(&format!("\nGold: {}g", econ.gold));
    t.single_mut().unwrap().0 = s;
}

fn update_upgrades_panel(
    econ: Res<Economy>,
    up: Res<UpgradesState>,
    mut t: Query<&mut Text, With<UpgradeList>>,
) {
    let mut s = String::new();
    for (i, def) in UPGRADES.iter().enumerate() {
        let lvl = up.levels[i];
        let cost = if lvl >= def.max_level {
            "MAX".to_string()
        } else {
            format!("{}g", def.costs[lvl as usize])
        };
        s.push_str(&format!(
            "  [{}] {}  Lv {}/{}  cost {}\n       {}\n",
            i + 1,
            def.name,
            lvl,
            def.max_level,
            cost,
            def.desc
        ));
    }
    s.push_str(&format!("\nGold: {}g", econ.gold));
    t.single_mut().unwrap().0 = s;
}

fn update_recipe_book(
    econ: Res<Economy>,
    inv: Res<Inventory>,
    mut t: Query<&mut Text, With<RecipeList>>,
) {
    let mut s = String::new();
    for r in RECIPES.iter() {
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
        let prefix = if unlocked { "[ok]" } else { "[--]" };
        s.push_str(&format!(
            "{} {}  {}g  temp {}-{}  {}s  {}\n",
            prefix,
            r.name,
            r.base_price,
            r.temp_min as i32,
            r.temp_max as i32,
            r.brew_time as i32,
            mats.join(" + ")
        ));
    }
    s.push_str(&format!(
        "\nReputation Lv {} unlocks tier-{} recipes.",
        econ.rep_level, econ.rep_level
    ));
    t.single_mut().unwrap().0 = s;
}

fn update_end_screen(econ: Res<Economy>, mut t: Query<&mut Text, With<EndText>>) {
    t.single_mut().unwrap().0 = format!(
        "Day {} reached · {} served · {} lost\nReputation: {}",
        econ.day, econ.served, econ.lost, econ.reputation
    );
}

fn key_label(i: usize) -> &'static str {
    match i {
        0 => "1",
        1 => "2",
        2 => "3",
        3 => "4",
        4 => "5",
        5 => "6",
        6 => "7",
        7 => "8",
        8 => "9",
        9 => "0",
        10 => "-",
        11 => "=",
        _ => "?",
    }
}
