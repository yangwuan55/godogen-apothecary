//! UI kit: shared palette, button spawn helpers and per-frame style refresh.
//! All game UI (HUD, panels, tutorial, pause) is built from these primitives so
//! colors, radii, fonts and interaction states stay consistent everywhere.

use super::data::{
    MATERIALS, SHELF_CAPACITY, UPGRADES, UpgradeId,
};
use super::resources::{
    Brewing, Customer, CustomerState, Economy, GameScreen, Inventory, Paused, UpgradesState,
};
use bevy::prelude::*;

// ---- Palette --------------------------------------------------------------

pub const C_BG: Color = Color::srgb(0.070, 0.063, 0.110);
pub const C_PANEL: Color = Color::srgb(0.098, 0.090, 0.149);
pub const C_CARD: Color = Color::srgb(0.133, 0.122, 0.200);
pub const C_CARD_ALT: Color = Color::srgb(0.165, 0.150, 0.235);
pub const C_BORDER: Color = Color::srgb(0.227, 0.208, 0.322);
pub const C_BORDER_HI: Color = Color::srgb(0.42, 0.38, 0.58);

pub const C_GOLD: Color = Color::srgb(0.976, 0.780, 0.310);
pub const C_GREEN: Color = Color::srgb(0.42, 0.80, 0.47);
pub const C_RED: Color = Color::srgb(0.878, 0.353, 0.353);
pub const C_BLUE: Color = Color::srgb(0.435, 0.659, 0.863);
pub const C_PURPLE: Color = Color::srgb(0.70, 0.55, 0.95);

pub const C_TXT: Color = Color::srgb(0.91, 0.90, 0.94);
pub const C_SOFT: Color = Color::srgb(0.66, 0.64, 0.72);
pub const C_HINT: Color = Color::srgb(0.48, 0.46, 0.56);

/// Disabled / unavailable button look.
const C_DISABLED: Color = Color::srgb(0.19, 0.18, 0.24);
const C_DISABLED_TXT: Color = Color::srgb(0.42, 0.40, 0.48);

pub fn font_handle(assets: &AssetServer) -> Handle<bevy::text::Font> {
    assets.load("fonts/NotoSansSC.ttf")
}

// ---- Buttons --------------------------------------------------------------

/// Semantic action a UI button triggers. `actions::collect_input` turns the
/// mouse interaction into the matching `UiAction` message.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Start,
    AcceptOrder,
    Stir,
    TempUp,
    TempDown,
    QtyInc,
    QtyDec,
    BuyMaterial(usize),
    BuyUpgrade(usize),
    OpenPanel(GameScreen),
    Pause,
    Resume,
    Continue,
    Restart,
    Quit,
}

/// Root marker of the always-present pause overlay (spawned once in `ui.rs`,
/// visibility driven by `Paused`). Buttons inside it are styled independently.
#[derive(Component)]
pub struct PausedOverlayRoot;

/// Marker for buttons that are currently impossible (out of stock / no money /
/// already maxed / no active brew). The style refresher grays them out.
#[derive(Component)]
pub struct ButtonDisabled;

/// Spawn a labeled rounded button with hover/press interaction.
pub fn spawn_button(
    p: &mut ChildSpawnerCommands,
    kind: ButtonKind,
    label: &str,
    width: Val,
    height: f32,
    accent: Color,
    assets: &AssetServer,
) {
    p.spawn((
        Node {
            width,
            height: px(height),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(accent),
        BorderColor::all(C_BORDER),
        Button,
        kind,
    ))
    .with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont {
                font: font_handle(assets).into(),
                font_size: FontSize::Px(15.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(color_for_accent(accent)),
        ));
    });
}

/// Text color that reads well on a given button accent.
pub fn color_for_accent(a: Color) -> Color {
    // Keep it simple: buttons are either bright accent (dark text) or dark (light text).
    if luminance(a) > 0.45 {
        Color::srgb(0.10, 0.09, 0.14)
    } else {
        C_TXT
    }
}

fn luminance(c: Color) -> f32 {
    let c = c.to_srgba();
    0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue
}

/// Per-frame: apply hover/pressed/disabled styling to every button.
pub fn refresh_buttons(
    econ: Res<Economy>,
    inv: Res<Inventory>,
    up: Res<UpgradesState>,
    brewing: Res<Brewing>,
    paused: Res<Paused>,
    customers: Query<&Customer>,
    mut q: Query<
        (
            &ButtonKind,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&ButtonDisabled>,
            &Children,
        ),
        (With<Button>, Without<PausedOverlayRoot>),
    >,
    mut label_colors: Query<&mut TextColor>,
) {
    let cap = SHELF_CAPACITY[up.level(UpgradeId::Shelf) as usize];
    let front_waiting = customers
        .iter()
        .any(|c| c.state == CustomerState::Waiting);

    for (kind, inter, mut bg, mut border, disabled, children) in &mut q {
        let (enabled, accent) = match kind {
            ButtonKind::Start => (true, C_GOLD),
            ButtonKind::AcceptOrder => (!brewing.active && front_waiting && !paused.0, C_GREEN),
            ButtonKind::Stir => (brewing.active && !paused.0, C_BLUE),
            ButtonKind::TempUp | ButtonKind::TempDown => (brewing.active && !paused.0, C_BLUE),
            ButtonKind::QtyInc => (inv.restock_qty < 10, C_GOLD),
            ButtonKind::QtyDec => (inv.restock_qty > 1, C_GOLD),
            ButtonKind::BuyMaterial(i) => {
                let m = &MATERIALS[*i];
                let qty = inv.restock_qty;
                (
                    inv.counts[*i] < cap && econ.gold >= m.cost * qty,
                    C_GREEN,
                )
            }
            ButtonKind::BuyUpgrade(i) => {
                let def = &UPGRADES[*i];
                let lvl = up.levels[*i] as usize;
                let aff = lvl < def.max_level as usize && econ.gold >= def.costs[lvl];
                (aff, C_GOLD)
            }
            ButtonKind::Continue => (true, C_GREEN),
            ButtonKind::OpenPanel(_) | ButtonKind::Pause | ButtonKind::Resume | ButtonKind::Restart => {
                (true, C_BORDER_HI)
            }
            ButtonKind::Quit => (true, C_RED),
        };

        let base = if enabled && disabled.is_none() { accent } else { C_DISABLED };
        let mut bg_col = base;
        let mut border_col = C_BORDER;
        match inter {
            Interaction::Hovered if enabled => {
                bg_col = lighten(base, 0.10);
                border_col = C_BORDER_HI;
            }
            Interaction::Pressed if enabled => {
                bg_col = darken(base, 0.12);
                border_col = C_GOLD;
            }
            _ => {}
        }
        bg.0 = bg_col;
        *border = BorderColor::all(border_col);

        // Tint the label according to enabled state.
        if let Some(child) = children.iter().next() {
            if let Ok(mut tc) = label_colors.get_mut(child) {
                tc.0 = if enabled && disabled.is_none() {
                    color_for_accent(accent)
                } else {
                    C_DISABLED_TXT
                };
            }
        }
    }
}

fn lighten(c: Color, f: f32) -> Color {
    let s = c.to_srgba();
    Color::srgb(
        (s.red + (1.0 - s.red) * f).clamp(0.0, 1.0),
        (s.green + (1.0 - s.green) * f).clamp(0.0, 1.0),
        (s.blue + (1.0 - s.blue) * f).clamp(0.0, 1.0),
    )
}
fn darken(c: Color, f: f32) -> Color {
    let s = c.to_srgba();
    Color::srgb(
        (s.red * (1.0 - f)).clamp(0.0, 1.0),
        (s.green * (1.0 - f)).clamp(0.0, 1.0),
        (s.blue * (1.0 - f)).clamp(0.0, 1.0),
    )
}

// ---- Generic helpers ------------------------------------------------------

/// A text node with the Noto Sans SC font.
pub fn ui_text(
    p: &mut ChildSpawnerCommands,
    s: &str,
    size: f32,
    color: Color,
    assets: &AssetServer,
    bold: bool,
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
    ));
}

/// A rounded "card" container with a heading.
pub fn card<F: FnOnce(&mut ChildSpawnerCommands)>(
    p: &mut ChildSpawnerCommands,
    padding: f32,
    build: F,
) {
    p.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(padding)),
            row_gap: px(6.0),
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::all(px(12.0)),
            ..default()
        },
        BackgroundColor(C_CARD),
        BorderColor::all(C_BORDER),
    ))
    .with_children(build);
}

/// A small colored "icon badge": rounded square with a single CJK glyph.
pub fn icon_badge(
    p: &mut ChildSpawnerCommands,
    glyph: &str,
    color: Color,
    size: f32,
    assets: &AssetServer,
) {
    p.spawn((
        Node {
            width: px(size),
            height: px(size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(size * 0.28)),
            ..default()
        },
        BackgroundColor(color),
    ))
    .with_children(|b| {
        b.spawn((
            Text::new(glyph),
            TextFont {
                font: font_handle(assets).into(),
                font_size: FontSize::Px(size * 0.62),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(Color::srgb(0.10, 0.09, 0.14)),
        ));
    });
}

/// A horizontal progress-bar track with an optional colored "ideal window"
/// segment positioned by percent. Returns the fill node entity via marker `F`
/// so systems can size it each frame.
#[derive(Component)]
pub struct BarFill;

#[derive(Component)]
pub struct BarWindow;

pub fn progress_bar<M: Component>(
    p: &mut ChildSpawnerCommands,
    width: f32,
    height: f32,
    window: Option<(f32, f32)>,
    _track_color: Color,
    _fill_marker: M,
) {
    p.spawn((
        Node {
            width: px(width),
            height: px(height),
            position_type: PositionType::Relative,
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::MAX,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.06, 0.055, 0.09)),
        BorderColor::all(C_BORDER),
    ))
    .with_children(|bar| {
        if let Some((lo, hi)) = window {
            bar.spawn((
                Node {
                    width: percent((hi - lo).max(4.0)),
                    height: percent(100.0),
                    position_type: PositionType::Absolute,
                    left: percent(lo),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.976, 0.780, 0.310, 0.22)),
                BarWindow,
            ));
        }
        bar.spawn((
            Node {
                width: percent(0.0),
                height: percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(C_GREEN),
            BarFill,
            _fill_marker,
        ));
    });
}
