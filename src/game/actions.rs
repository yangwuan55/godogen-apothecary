//! Unified input channel. Keyboard and mouse are *equal* input paths: both are
//! translated here into `UiAction` messages (plus the `TempControl` hold state),
//! and each gameplay domain consumes those messages once. This guarantees every
//! critical action works with mouse and keyboard, with one code path.

use super::data::NUM_MATERIALS;
use super::resources::*;
use super::ui_kit::ButtonKind;
use bevy::prelude::*;

/// One logical player action. Written by `collect_input` (keyboard + mouse),
/// consumed by the owning gameplay system (customers / brewing / economy / ui).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    StartGame,
    AcceptOrder,
    Stir,
    QtyInc,
    QtyDec,
    BuyMaterial(usize),
    BuyUpgrade(usize),
    OpenPanel(GameScreen),
    Pause,
    Resume,
    Continue, // day report -> next day / victory
    Restart,  // back to title (end screens / pause menu / day report)
    Quit,
}

/// Systems that collect raw input into `UiAction`s. Consumers schedule
/// themselves `.after(InputSet)` so the message is readable the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiAction>();
        app.add_systems(Update, collect_input.in_set(InputSet));
    }
}

const PANEL_ORDER: [GameScreen; 4] = [
    GameScreen::Playing,
    GameScreen::Market,
    GameScreen::Upgrades,
    GameScreen::RecipeBook,
];

/// Translate keyboard + mouse into `UiAction`s and the temperature hold state.
fn collect_input(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameScreen>>,
    paused: Res<Paused>,
    mut tc: ResMut<TempControl>,
    mut writer: MessageWriter<UiAction>,
    clicked: Query<(&ButtonKind, &Interaction), Changed<Interaction>>,
    held: Query<(&ButtonKind, &Interaction), With<ButtonKind>>,
) {
    let cur = *state.get();

    // --- Temperature hold (keyboard OR mouse buttons) ---------------------
    let key_up = input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW);
    let key_down = input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS);
    let mut btn_up = false;
    let mut btn_down = false;
    for (kind, inter) in &held {
        if *inter != Interaction::Pressed {
            continue;
        }
        match kind {
            ButtonKind::TempUp => btn_up = true,
            ButtonKind::TempDown => btn_down = true,
            _ => {}
        }
    }
    tc.up = key_up || btn_up;
    tc.down = key_down || btn_down;

    // --- Mouse clicks ------------------------------------------------------
    for (kind, inter) in &clicked {
        if *inter != Interaction::Pressed {
            continue;
        }
        match kind {
            ButtonKind::Start => { writer.write(UiAction::StartGame); }
            ButtonKind::AcceptOrder => { writer.write(UiAction::AcceptOrder); }
            ButtonKind::Stir => { writer.write(UiAction::Stir); }
            ButtonKind::TempUp | ButtonKind::TempDown => {} // handled via hold
            ButtonKind::QtyInc => { writer.write(UiAction::QtyInc); }
            ButtonKind::QtyDec => { writer.write(UiAction::QtyDec); }
            ButtonKind::BuyMaterial(i) => { writer.write(UiAction::BuyMaterial(*i)); }
            ButtonKind::BuyUpgrade(i) => { writer.write(UiAction::BuyUpgrade(*i)); }
            ButtonKind::OpenPanel(p) => { writer.write(UiAction::OpenPanel(*p)); }
            ButtonKind::Pause => { writer.write(UiAction::Pause); }
            ButtonKind::Resume => { writer.write(UiAction::Resume); }
            ButtonKind::Continue => { writer.write(UiAction::Continue); }
            ButtonKind::Restart => { writer.write(UiAction::Restart); }
            ButtonKind::Quit => { writer.write(UiAction::Quit); }
        }
    }

    // --- Keyboard ----------------------------------------------------------
    // Enter / Space double as context-sensitive confirm keys.
    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
        match cur {
            GameScreen::Title => {
                writer.write(UiAction::StartGame);
            }
            GameScreen::Playing => {
                writer.write(UiAction::AcceptOrder);
            }
            GameScreen::DayReport => {
                writer.write(UiAction::Continue);
            }
            GameScreen::GameOver | GameScreen::Victory => {
                writer.write(UiAction::Restart);
            }
            _ => {}
        }
    }
    if input.just_pressed(KeyCode::KeyE) && cur == GameScreen::Playing {
        writer.write(UiAction::AcceptOrder);
    }
    if input.just_pressed(KeyCode::Space) && cur == GameScreen::Playing {
        writer.write(UiAction::Stir);
    }
    if input.just_pressed(KeyCode::Tab) {
        let next = match PANEL_ORDER.iter().position(|&p| p == cur) {
            Some(pos) => PANEL_ORDER[(pos + 1) % PANEL_ORDER.len()],
            None => GameScreen::Playing,
        };
        writer.write(UiAction::OpenPanel(next));
    }
    if cur == GameScreen::Market {
        for i in 0..NUM_MATERIALS {
            if input.just_pressed(key_for_index(i)) {
                writer.write(UiAction::BuyMaterial(i));
            }
        }
        if input.just_pressed(KeyCode::BracketRight) {
            writer.write(UiAction::QtyInc);
        }
        if input.just_pressed(KeyCode::BracketLeft) {
            writer.write(UiAction::QtyDec);
        }
    }
    if cur == GameScreen::Upgrades {
        for i in 0..4 {
            let key = match i {
                0 => KeyCode::Digit1,
                1 => KeyCode::Digit2,
                2 => KeyCode::Digit3,
                _ => KeyCode::Digit4,
            };
            if input.just_pressed(key) {
                writer.write(UiAction::BuyUpgrade(i));
            }
        }
    }
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::KeyP) {
        match cur {
            GameScreen::Title
            | GameScreen::DayReport
            | GameScreen::GameOver
            | GameScreen::Victory => {}
            _ => {
                if paused.0 {
                    writer.write(UiAction::Resume);
                } else {
                    writer.write(UiAction::Pause);
                }
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
