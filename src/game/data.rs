//! Static game data: materials, recipes, customer kinds, upgrades.

use bevy::prelude::*;

pub const NUM_MATERIALS: usize = 12;
pub const NUM_RECIPES: usize = 14;

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------

pub struct MaterialDef {
    pub name: &'static str,
    pub cost: u32, // restock price per unit
    pub tier: u8,  // 1..3
    pub color: Color,
    pub emoji: char, // simple icon glyph used in UI lists
}

pub const MATERIALS: [MaterialDef; NUM_MATERIALS] = [
    MaterialDef {
        name: "Mandrake Root",
        cost: 3,
        tier: 1,
        color: Color::srgb(0.55, 0.85, 0.35),
        emoji: '🌿',
    },
    MaterialDef {
        name: "Moonpetal",
        cost: 4,
        tier: 1,
        color: Color::srgb(0.80, 0.60, 0.95),
        emoji: '🌸',
    },
    MaterialDef {
        name: "Iron Moss",
        cost: 5,
        tier: 1,
        color: Color::srgb(0.45, 0.55, 0.50),
        emoji: '🍄',
    },
    MaterialDef {
        name: "Foxglove",
        cost: 6,
        tier: 1,
        color: Color::srgb(0.95, 0.55, 0.40),
        emoji: '🪻',
    },
    MaterialDef {
        name: "Basilisk Scale",
        cost: 12,
        tier: 2,
        color: Color::srgb(0.35, 0.80, 0.60),
        emoji: '🦎',
    },
    MaterialDef {
        name: "Phoenix Feather",
        cost: 14,
        tier: 2,
        color: Color::srgb(0.95, 0.45, 0.25),
        emoji: '🔥',
    },
    MaterialDef {
        name: "Void Crystal",
        cost: 16,
        tier: 2,
        color: Color::srgb(0.35, 0.30, 0.70),
        emoji: '🔮',
    },
    MaterialDef {
        name: "Griffin Claw",
        cost: 18,
        tier: 2,
        color: Color::srgb(0.85, 0.75, 0.40),
        emoji: '🦅',
    },
    MaterialDef {
        name: "Dragon's Blood",
        cost: 30,
        tier: 3,
        color: Color::srgb(0.75, 0.15, 0.20),
        emoji: '🐉',
    },
    MaterialDef {
        name: "Star Shard",
        cost: 34,
        tier: 3,
        color: Color::srgb(0.60, 0.80, 1.00),
        emoji: '⭐',
    },
    MaterialDef {
        name: "Unicorn Horn",
        cost: 38,
        tier: 3,
        color: Color::srgb(0.95, 0.90, 0.80),
        emoji: '🦄',
    },
    MaterialDef {
        name: "Abyss Salt",
        cost: 42,
        tier: 3,
        color: Color::srgb(0.25, 0.30, 0.45),
        emoji: '🧂',
    },
];

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

pub struct RecipeDef {
    pub name: &'static str,
    pub mats: &'static [u8], // MaterialId indices
    pub base_price: u32,
    pub temp_min: f32, // ideal temperature window (0..100)
    pub temp_max: f32,
    pub brew_time: f32,   // seconds
    pub stir_points: u32, // number of stir moments
    pub tier: u8,         // unlock: recipe tier <= reputation level
    pub color: Color,     // potion color
}

pub const RECIPES: [RecipeDef; NUM_RECIPES] = [
    RecipeDef {
        name: "Healing Draught",
        mats: &[0, 1],
        base_price: 15,
        temp_min: 40.0,
        temp_max: 70.0,
        brew_time: 6.0,
        stir_points: 2,
        tier: 1,
        color: Color::srgb(0.30, 0.85, 0.45),
    },
    RecipeDef {
        name: "Mana Elixir",
        mats: &[1, 2],
        base_price: 18,
        temp_min: 50.0,
        temp_max: 80.0,
        brew_time: 6.5,
        stir_points: 2,
        tier: 1,
        color: Color::srgb(0.35, 0.50, 0.95),
    },
    RecipeDef {
        name: "Strength Tonic",
        mats: &[2, 3],
        base_price: 20,
        temp_min: 45.0,
        temp_max: 75.0,
        brew_time: 7.0,
        stir_points: 2,
        tier: 1,
        color: Color::srgb(0.85, 0.40, 0.30),
    },
    RecipeDef {
        name: "Night Vision Brew",
        mats: &[0, 4],
        base_price: 30,
        temp_min: 35.0,
        temp_max: 65.0,
        brew_time: 7.5,
        stir_points: 3,
        tier: 2,
        color: Color::srgb(0.30, 0.25, 0.55),
    },
    RecipeDef {
        name: "Fire Salve",
        mats: &[4, 5],
        base_price: 40,
        temp_min: 65.0,
        temp_max: 95.0,
        brew_time: 8.0,
        stir_points: 3,
        tier: 2,
        color: Color::srgb(0.95, 0.50, 0.15),
    },
    RecipeDef {
        name: "Frost Balm",
        mats: &[2, 6],
        base_price: 42,
        temp_min: 15.0,
        temp_max: 45.0,
        brew_time: 8.0,
        stir_points: 3,
        tier: 2,
        color: Color::srgb(0.55, 0.85, 0.95),
    },
    RecipeDef {
        name: "Swiftness Syrup",
        mats: &[3, 7],
        base_price: 45,
        temp_min: 30.0,
        temp_max: 60.0,
        brew_time: 8.5,
        stir_points: 3,
        tier: 2,
        color: Color::srgb(0.95, 0.80, 0.30),
    },
    RecipeDef {
        name: "Heart's Bloom",
        mats: &[1, 11],
        base_price: 55,
        temp_min: 25.0,
        temp_max: 55.0,
        brew_time: 9.0,
        stir_points: 3,
        tier: 3,
        color: Color::srgb(0.95, 0.45, 0.70),
    },
    RecipeDef {
        name: "Courage Draught",
        mats: &[7, 8],
        base_price: 70,
        temp_min: 55.0,
        temp_max: 85.0,
        brew_time: 9.5,
        stir_points: 3,
        tier: 3,
        color: Color::srgb(0.80, 0.30, 0.50),
    },
    RecipeDef {
        name: "Stone Skin Elixir",
        mats: &[4, 3, 6],
        base_price: 85,
        temp_min: 40.0,
        temp_max: 70.0,
        brew_time: 10.0,
        stir_points: 4,
        tier: 3,
        color: Color::srgb(0.60, 0.55, 0.50),
    },
    RecipeDef {
        name: "Invisibility Potion",
        mats: &[6, 5],
        base_price: 90,
        temp_min: 20.0,
        temp_max: 50.0,
        brew_time: 10.5,
        stir_points: 4,
        tier: 3,
        color: Color::srgb(0.50, 0.60, 0.70),
    },
    RecipeDef {
        name: "Dragon's Breath",
        mats: &[8, 9],
        base_price: 120,
        temp_min: 70.0,
        temp_max: 100.0,
        brew_time: 11.0,
        stir_points: 4,
        tier: 3,
        color: Color::srgb(0.95, 0.35, 0.10),
    },
    RecipeDef {
        name: "True Sight",
        mats: &[6, 11, 9],
        base_price: 150,
        temp_min: 30.0,
        temp_max: 60.0,
        brew_time: 12.0,
        stir_points: 4,
        tier: 3,
        color: Color::srgb(0.75, 0.55, 1.00),
    },
    RecipeDef {
        name: "Phoenix Tears",
        mats: &[5, 9, 8],
        base_price: 220,
        temp_min: 50.0,
        temp_max: 80.0,
        brew_time: 13.0,
        stir_points: 5,
        tier: 3,
        color: Color::srgb(1.00, 0.70, 0.30),
    },
];

// ---------------------------------------------------------------------------
// Customer kinds
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct CustomerKind {
    pub name: &'static str,
    pub budget_min: u32,
    pub budget_max: u32,
    pub patience: f32, // seconds before they leave
    pub min_tier: u8,  // preferred recipe tier range
    pub max_tier: u8,
    pub unlock_reputation: u8, // reputation level required to spawn
    pub body_color: Color,
    pub hat_color: Color,
    pub hat_style: u8, // 0 none, 1 cap, 2 pointy, 3 crown
}

pub const CUSTOMER_KINDS: [CustomerKind; 7] = [
    CustomerKind {
        name: "Farmer",
        budget_min: 8,
        budget_max: 25,
        patience: 32.0,
        min_tier: 1,
        max_tier: 1,
        unlock_reputation: 1,
        body_color: Color::srgb(0.65, 0.45, 0.25),
        hat_color: Color::srgb(0.55, 0.35, 0.15),
        hat_style: 1,
    },
    CustomerKind {
        name: "Child",
        budget_min: 5,
        budget_max: 16,
        patience: 40.0,
        min_tier: 1,
        max_tier: 1,
        unlock_reputation: 1,
        body_color: Color::srgb(0.95, 0.75, 0.60),
        hat_color: Color::srgb(0.90, 0.25, 0.25),
        hat_style: 1,
    },
    CustomerKind {
        name: "Merchant",
        budget_min: 20,
        budget_max: 45,
        patience: 26.0,
        min_tier: 1,
        max_tier: 2,
        unlock_reputation: 2,
        body_color: Color::srgb(0.55, 0.45, 0.65),
        hat_color: Color::srgb(0.35, 0.25, 0.45),
        hat_style: 0,
    },
    CustomerKind {
        name: "Knight",
        budget_min: 35,
        budget_max: 80,
        patience: 22.0,
        min_tier: 2,
        max_tier: 2,
        unlock_reputation: 3,
        body_color: Color::srgb(0.75, 0.70, 0.60),
        hat_color: Color::srgb(0.60, 0.60, 0.65),
        hat_style: 0,
    },
    CustomerKind {
        name: "Mage",
        budget_min: 50,
        budget_max: 120,
        patience: 16.0,
        min_tier: 2,
        max_tier: 3,
        unlock_reputation: 4,
        body_color: Color::srgb(0.40, 0.35, 0.65),
        hat_color: Color::srgb(0.25, 0.30, 0.70),
        hat_style: 2,
    },
    CustomerKind {
        name: "Noble",
        budget_min: 80,
        budget_max: 200,
        patience: 12.0,
        min_tier: 3,
        max_tier: 3,
        unlock_reputation: 6,
        body_color: Color::srgb(0.85, 0.75, 0.90),
        hat_color: Color::srgb(0.95, 0.80, 0.20),
        hat_style: 3,
    },
    CustomerKind {
        name: "Alchemist",
        budget_min: 150,
        budget_max: 300,
        patience: 10.0,
        min_tier: 3,
        max_tier: 3,
        unlock_reputation: 8,
        body_color: Color::srgb(0.55, 0.75, 0.55),
        hat_color: Color::srgb(0.25, 0.50, 0.30),
        hat_style: 2,
    },
];

// ---------------------------------------------------------------------------
// Upgrades
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UpgradeId {
    Cauldron,
    Furnace,
    Shelf,
    Sign,
}

pub struct UpgradeDef {
    pub id: UpgradeId,
    pub name: &'static str,
    pub desc: &'static str,
    pub max_level: u8,
    pub costs: [u32; 3], // cost per level (0..max_level)
}

pub const UPGRADES: [UpgradeDef; 4] = [
    UpgradeDef {
        id: UpgradeId::Cauldron,
        name: "Cauldron",
        desc: "Wider temp window + faster brew",
        max_level: 3,
        costs: [30, 75, 160],
    },
    UpgradeDef {
        id: UpgradeId::Furnace,
        name: "Furnace",
        desc: "More precise temp control",
        max_level: 3,
        costs: [25, 60, 130],
    },
    UpgradeDef {
        id: UpgradeId::Shelf,
        name: "Shelf",
        desc: "Bigger inventory capacity",
        max_level: 3,
        costs: [20, 50, 110],
    },
    UpgradeDef {
        id: UpgradeId::Sign,
        name: "Sign",
        desc: "More customers per day + rep",
        max_level: 3,
        costs: [35, 85, 180],
    },
];

// Capacity per shelf level: [base, lvl1, lvl2, lvl3]
pub const SHELF_CAPACITY: [u32; 4] = [6, 10, 15, 24];
// Max customers per day per sign level
pub const SIGN_CUSTOMERS: [u32; 4] = [6, 9, 13, 18];

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quality {
    Poor,
    Normal,
    Good,
    Perfect,
}

impl Quality {
    pub fn label(&self) -> &'static str {
        match self {
            Quality::Poor => "Poor",
            Quality::Normal => "Normal",
            Quality::Good => "Good",
            Quality::Perfect => "Perfect",
        }
    }
    pub fn price_mult(&self) -> f32 {
        match self {
            Quality::Poor => 0.6,
            Quality::Normal => 1.0,
            Quality::Good => 1.35,
            Quality::Perfect => 1.7,
        }
    }
    pub fn rep_gain(&self) -> u32 {
        match self {
            Quality::Poor => 1,
            Quality::Normal => 2,
            Quality::Good => 3,
            Quality::Perfect => 5,
        }
    }
}

pub fn quality_from_score(score: f32) -> Quality {
    if score >= 0.85 {
        Quality::Perfect
    } else if score >= 0.60 {
        Quality::Good
    } else if score >= 0.35 {
        Quality::Normal
    } else {
        Quality::Poor
    }
}

// Reputation level thresholds (points needed to reach next level)
pub const REP_THRESHOLDS: [u32; 10] = [0, 20, 50, 90, 140, 200, 270, 350, 440, 540];
