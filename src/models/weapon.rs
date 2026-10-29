//! Weapon model for Warhammer 40k
//!
//! Represents the offensive characteristics of a weapon.

use anyhow::Context;
use std::str::FromStr;
use typed_builder::TypedBuilder;

use crate::core::stats::Result;
use crate::dice::DiceExpr;
use crate::error;
use crate::newrecruit;

/// Weapon profile
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Default,
    Hash,
    Debug,
    TypedBuilder,
)]
pub struct Weapon {
    /// Weapon name
    pub name: String,

    /// Range weapon
    pub range: u64,

    /// Number of attack throw
    pub attack: Result,

    /// Skill shot/close hit threshold
    pub skill: u64,

    /// Strength of weapons
    pub strength: u64,

    /// Armor perforation
    pub ap: u64,

    /// Damage for not save wound
    pub damage: Result,
}

impl TryFrom<&[newrecruit::Characteristic]> for Weapon {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &[newrecruit::Characteristic]) -> error::Result<Weapon> {
        let mut weapon = Weapon::default();

        for characteristic in other {
            match characteristic.name.as_str() {
                "Range" => {
                    if let Some(ref range) = characteristic.text {
                        weapon.range = range
                            .replace("Melee", "0")
                            .replace("\"", "")
                            .parse::<u64>()
                            .with_context(|| format!("Failed to parse range {}", range))?;
                    }
                }
                "A" => {
                    if let Some(ref attack) = characteristic.text {
                        weapon.attack = DiceExpr::from(tyche::Expr::from_str(attack).unwrap()).to_result();
                    }
                }
                "WS" => {
                    if let Some(ref skill) = characteristic.text {
                        weapon.skill = skill
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failed to parse skill WS {}", skill))?;
                    }
                }
                "BS" => {
                    if let Some(ref skill) = characteristic.text {
                        weapon.skill = skill
                            .replace("+", "")
                            .replace("N/A", "0")
                            .parse()
                            .with_context(|| format!("Failed to parse skill BS {}", skill))?;
                    }
                }
                "AP" => {
                    if let Some(ref ap) = characteristic.text {
                        weapon.ap = ap
                            .replace("-", "")
                            .parse()
                            .with_context(|| format!("Failed to parse AP {}", ap))?;
                    }
                }
                "S" => {
                    if let Some(ref strength) = characteristic.text {
                        weapon.strength = strength
                            .parse()
                            .with_context(|| format!("Failed to parse S {}", strength))?;
                    }
                }
                "D" => {
                    if let Some(ref damage) = characteristic.text {
                        weapon.damage = DiceExpr::from(tyche::Expr::from_str(&damage.to_lowercase()).unwrap()).to_result();
                    }
                }
                _ => (),
            }
        }

        Ok(weapon)
    }
}

impl Weapon {
    #[hotpath::measure]
    pub fn run(&self, n: u64, body: &crate::models::Body) -> Result {
        let full = crate::roll::FullRoll::builder()
            .weapons_number(n)
            .toughness(body.toughness)
            .save(body.save)
            .invulnerable_save(body.invuln)
            .fnp_threshold(body.fnp)
            .attack(self.attack)
            .skill(self.skill)
            .strength(self.strength)
            .ap(self.ap)
            .damage(self.damage)
            .build();

        full.roll()
    }
}
