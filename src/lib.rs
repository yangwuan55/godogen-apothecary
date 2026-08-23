//! Apothecary Sim — 2D potion-shop management sim.
//! Single App-wiring point; gameplay lives in `game`.

pub mod game;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

pub fn app() -> App {
    build_app(|p| p.build())
}

/// Build the game app; `modify` lets callers (e.g. the offscreen capture binary)
/// reconfigure `DefaultPlugins` before the game plugins are added.
pub fn build_app<F: FnOnce(DefaultPlugins) -> PluginGroupBuilder>(modify: F) -> App {
    let mut app = App::new();
    app.add_plugins(modify(DefaultPlugins))
        .add_plugins(game::core::CorePlugin);
    app
}
