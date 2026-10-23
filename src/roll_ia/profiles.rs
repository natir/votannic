use super::expr;
use super::rules;

/// Weapon profile (shooting or melee).
#[derive(Clone, Debug)]
pub struct Weapon {
    pub name: String,
    /// Attacks: deterministic, or the result of a dice roll DAG
    /// (e.g. 1d4+2) if the profile contains dice.
    pub attacks: expr::DiceExpr,
    /// To-hit skill BS (shooting) or WS (melee): 3 == "3+".
    pub skill: u32,
    /// Strength (S).
    pub strength: u32,
    /// Armour penetration (AP), negative: -1 == "AP -1".
    pub ap: i32,
    /// Damage per unsaved wounding attack: a fixed value (e.g. 1) or a
    /// dice expression (e.g. 1d3, 2d6) rolled per attack.
    pub damage: expr::DiceExpr,
    /// Abilities of the weapon triggered on an unmodified 6.
    pub abilities: Vec<rules::CritAbility>,
}

/// Profile of the targeted unit.
#[derive(Clone, Debug)]
pub struct UnitProfile {
    pub name: String,
    /// Toughness (T).
    pub toughness: u32,
    /// Armour save (Sv): 3 == "3+", 7 == no armour.
    pub save: u32,
    /// Invulnerable save, if any (ignores AP).
    pub invulnerable: Option<u32>,
    /// Feel No Pain threshold, if any: 6 == "6+", 5 == "5+".
    pub feel_no_pain: Option<u32>,
    /// Wounds (W) per model.
    pub wounds: u32,
    /// Number of models in the unit.
    pub models: u32,
}

impl Weapon {
    pub fn has(&self, ability: &rules::CritAbility) -> bool {
        self.abilities.iter().any(|a| a == ability)
    }

    /// Sustained Hits X value of the weapon, 0 if absent.
    pub fn sustained_hits(&self) -> u32 {
        self.abilities
            .iter()
            .find_map(|a| match a {
                rules::CritAbility::SustainedHits(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0)
    }
}
