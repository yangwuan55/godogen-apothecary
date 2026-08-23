# The Alchemist's Apothecary

A complete 2D potion-shop management sim built with **Bevy 0.19** and generated
through the **godogen** pipeline (`publish.sh --engine bevy --agent prime`).

You run a struggling alchemist's shop: buy ingredients, serve a queue of
customers, brew potions by managing temperature and stirring, and grow your
reputation until the nobility — and the Alchemist's Guild — come calling.

## Core loop

```
Buy materials -> customers order -> brew (temp + stir) -> deliver -> gold + reputation
      ^                                                              |
      +------------------ unlock recipes / upgrade shop <------------+
```

- **Day cycle**: a fixed-length day ticks by; each served potion earns gold and
  reputation, and every night **rent is due**.
- **Reputation gates content**: richer, more demanding customers and higher-tier
  recipes unlock as your reputation grows.
- **Lose & win conditions**: run out of gold at day end => bankruptcy;
  reach reputation level 10 => victory.

## Controls

| Key                | Action                                  |
|--------------------|-----------------------------------------|
| `Enter`            | Start game / accept a customer's order  |
| `Tab`              | Cycle panels (Market / Upgrades / Recipe Book / shop) |
| `1..9`             | Buy material / buy upgrade (in panels)  |
| `↑` / `↓`          | Raise / lower cauldron temperature      |
| `Space`            | Stir the cauldron                       |

## Game content

- **12 ingredients** across 3 tiers (Mandrake Root … Abyss Salt).
- **14 recipes** across 3 tiers (Healing Draught … Phoenix Tears), each with an
  ideal temperature window, brew time and stir requirements.
- **7 customer archetypes** (Farmer, Child, Merchant, Knight, Mage, Noble,
  Alchemist) with different budgets, patience and unlocked tiers.
- **4 shop upgrades** (Cauldron, Furnace, Shelf, Sign), each up to level 3.
- **Quality system**: Perfect / Good / Normal / Poor potions pay more or less,
  driven by temperature discipline and stirring coverage.

## Procedural art (no external assets)

Everything is drawn at runtime with Bevy primitives:

| Element       | Built from                                              |
|---------------|---------------------------------------------------------|
| Shopfront     | Wood-panelled walls, floor, shelves of colour-coded bottles |
| Counter       | Wood counter with hinged sections and a serving window  |
| Cauldron      | Iron pot, liquid surface, floating ingredient colour    |
| Customers     | Body, head, hat (3 styles) tinted per archetype         |
| Effects       | Bubbles, sparkles, fizzle puffs, floating gold/quality text |
| UI panels     | Rounded rects, progress bars (temp + brew), HUD         |

## Run it

```sh
# Requires Rust stable (tested with 1.97)
cargo run --release
```

## Proof video

`docs/proof.mp4` (16 s, 720p) shows an autopilot playing a full shop day:
accepting orders, brewing to the temperature window, auto-stirring, and
cycling the Market / Upgrades / Recipe Book panels. Regenerate with:

```sh
cargo run --bin capture && ffmpeg -y -framerate 30 -i screenshots/result/frame%05d.png -c:v libx264 -pix_fmt yuv420p docs/proof.mp4
```

## Architecture

```
src/lib.rs          App wiring (build_app exposes plugin tweaks for capture)
src/main.rs         Normal windowed entry point
src/bin/capture.rs  Offscreen 30 fps recorder + autopilot (proof video)
src/game/
  core.rs           Plugin assembly, states, title/game-over screens
  data.rs           Materials, recipes, customer kinds, upgrades, quality
  resources.rs      Economy, Inventory, Brewing, Customer, FX resources
  customers.rs      Spawning, queues, patience, order accept/deliver
  brewing.rs        Temperature/stir mechanics, quality scoring, delivery
  economy.rs        Day cycle, rent, unlock gating, end conditions
  panels.rs         Market / Upgrades / Recipe Book tabbed panels
  ui.rs             HUD, brew panel, day report, end screens
  visual.rs         Procedural shopfront, cauldron, customer sprites
  particles.rs      FX spawner + floaters
```

Built with the **godogen** publish pipeline (Prime Agent layout:
`AGENTS.md` + `.prime/agent/skills/asset-gen`).
