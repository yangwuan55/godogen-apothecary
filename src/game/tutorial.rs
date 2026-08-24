//! Lightweight in-game tutorial: 7 hint steps that advance automatically from
//! observed game events. Hints never block the player — they only guide.

use super::actions::{InputSet, UiAction};
use super::data::RECIPES;
use super::resources::{
    Brewing, Customer, CustomerState, Economy, GameScreen, Paused, TutorialSettings, UpgradesState,
};
use super::ui::{TutorialRoot, TutorialText};
use super::ui_kit::ButtonKind;
use bevy::prelude::*;

#[derive(Resource)]
pub struct Tutorial {
    pub step: usize, // 0..8 (8 = done)
    pub started: bool,
    pub seen_waiting: bool,
    pub seen_served_before: u32,
    pub timer: f32,
}

impl Default for Tutorial {
    fn default() -> Self {
        Self {
            step: 0,
            started: false,
            seen_waiting: false,
            seen_served_before: 0,
            timer: 0.0,
        }
    }
}

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tutorial>();
        app.add_systems(
            Update,
            (advance_tutorial, apply_tutorial_highlight)
                .after(InputSet)
                .run_if(tutorial_enabled),
        );
    }
}

fn tutorial_enabled(settings: Res<TutorialSettings>) -> bool {
    settings.enabled
}

fn advance_tutorial(
    mut tut: ResMut<Tutorial>,
    settings: Res<TutorialSettings>,
    state: Res<State<GameScreen>>,
    brewing: Res<Brewing>,
    econ: Res<Economy>,
    up: Res<UpgradesState>,
    customers: Query<&Customer>,
    paused: Res<Paused>,
    time: Res<Time>,
    mut actions: MessageReader<UiAction>,
) {
    if !settings.enabled || paused.0 {
        return;
    }
    if *state.get() == GameScreen::Playing {
        tut.started = true;
    }
    if !tut.started {
        return;
    }

    let waiting = customers.iter().any(|c| c.state == CustomerState::Waiting);

    // Advance the step machine.
    match tut.step {
        0 => {
            if waiting {
                tut.seen_waiting = true;
            }
            if brewing.active || tut.seen_waiting && waiting {
                tut.step = 1;
            }
        }
        1 => {
            if econ.served > tut.seen_served_before {
                tut.seen_served_before = econ.served;
                tut.step = 3; // finished first brew
            } else if brewing.active {
                tut.timer += time.delta_secs();
                if tut.timer > 4.0 {
                    tut.step = 2; // enough temp-time, hint about stir
                }
            }
        }
        2 => {
            if econ.served > tut.seen_served_before {
                tut.seen_served_before = econ.served;
                tut.step = 3;
            }
        }
        3 => {
            tut.timer += time.delta_secs();
            if tut.timer > 3.0 {
                tut.timer = 0.0;
                tut.step = 4;
            }
        }
        4 => {
            for a in actions.read() {
                if *a == UiAction::OpenPanel(GameScreen::Market) {
                    tut.step = 5;
                }
            }
        }
        5 => {
            if econ.purchases >= 1 {
                tut.step = 6;
            }
        }
        6 => {
            for a in actions.read() {
                if *a == UiAction::OpenPanel(GameScreen::Upgrades)
                    || matches!(*a, UiAction::BuyUpgrade(_))
                {
                    tut.step = 7;
                }
            }
            if up.levels.iter().any(|&l| l > 0) {
                tut.step = 7;
            }
        }
        7 => {
            tut.timer += time.delta_secs();
            if tut.timer > 6.0 {
                tut.step = 8;
            }
        }
        _ => {}
    }
}

/// Text + highlight target for the current step.
fn step_text(tut: &Tutorial, brewing: &Brewing) -> (String, Option<ButtonKind>) {
    match tut.step {
        0 => (
            "顾客来了！头顶气泡会说明他们要买什么。点「接单」接下订单。".to_string(),
            Some(ButtonKind::AcceptOrder),
        ),
        1 => {
            let r = &RECIPES[brewing.recipe_idx];
            (
                format!(
                    "熬制中：用 ↑↓ 或「升温/降温」把温度保持在 {}°-{}° 的金色区间。",
                    r.temp_min as i32, r.temp_max as i32
                ),
                Some(ButtonKind::TempUp),
            )
        }
        2 => (
            "进度条到搅拌点时会提示！按 Space 或点「搅拌」。".to_string(),
            Some(ButtonKind::Stir),
        ),
        3 => (
            "出货成功！金币与声望到账。留意右侧的收益飘字。".to_string(),
            None,
        ),
        4 => (
            "原料会越用越少。点 HUD 上方的「市场」去进货。".to_string(),
            Some(ButtonKind::OpenPanel(GameScreen::Market)),
        ),
        5 => (
            "点击原料行的「购入」按钮即可补货，用 +/− 调整每次数量。".to_string(),
            Some(ButtonKind::BuyMaterial(0)),
        ),
        6 => (
            "攒够金币后去「升级」工坊：坩埚、熔炉、货架、招牌都会派上用场。".to_string(),
            Some(ButtonKind::OpenPanel(GameScreen::Upgrades)),
        ),
        7 => (
            "祝你经营顺利！声望越高，客人越尊贵，配方越珍贵。".to_string(),
            None,
        ),
        _ => (String::new(), None),
    }
}

fn apply_tutorial_highlight(
    tut: Res<Tutorial>,
    settings: Res<TutorialSettings>,
    state: Res<State<GameScreen>>,
    brewing: Res<Brewing>,
    mut banner_vis: Query<&mut Visibility, With<TutorialRoot>>,
    mut text_q: Query<&mut Text, With<TutorialText>>,
    time: Res<Time>,
    mut buttons: Query<(&ButtonKind, &mut BorderColor)>,
) {
    if !settings.enabled {
        return;
    }
    let (text, target) = step_text(&tut, &brewing);
    let show = *state.get() == GameScreen::Playing && tut.step < 8;
    if let Ok(mut v) = banner_vis.single_mut() {
        *v = if show && !text.is_empty() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if show {
        if let Ok(mut t) = text_q.single_mut() {
            t.0 = text.clone();
        }
    }

    // Pulse-highlight the target button.
    let pulse = (time.elapsed_secs() * 4.0).sin() * 0.5 + 0.5;
    let hi = Color::srgba(1.0, 0.72, 0.25, 0.55 + 0.45 * pulse);
    for (kind, mut border) in &mut buttons {
        if let Some(target) = &target {
            if *kind == *target {
                *border = BorderColor::all(hi);
            }
        }
    }
}
