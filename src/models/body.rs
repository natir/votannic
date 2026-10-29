//! Body model for Warhammer 40k units
//!
//! Represents the defensive characteristics of a model.

use anyhow::Context;
use typed_builder::TypedBuilder;

use crate::error;
use crate::newrecruit;

/// Body of Model
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    TypedBuilder,
)]
pub struct Body {
    /// Name
    pub name: String,

    /// Toughness
    pub toughness: u64,

    /// Armor save
    pub save: u64,

    /// invuln save
    pub invuln: u64,

    /// [FEEL NO PAIN]
    pub fnp: u64,

    /// Wounds
    pub wounds: u64,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            toughness: 0,
            save: 7,
            invuln: 7,
            fnp: 7,
            wounds: 0,
        }
    }
}

impl TryFrom<&[newrecruit::Characteristic]> for Body {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &[newrecruit::Characteristic]) -> error::Result<Body> {
        let mut body = Body::default();

        for characteristic in other {
            match characteristic.name.as_str() {
                "T" => {
                    if let Some(ref toughness) = characteristic.text {
                        body.toughness = toughness
                            .parse()
                            .with_context(|| format!("Failed to parse toughness {}", toughness))?;
                    }
                }
                "Sv" => {
                    if let Some(ref save) = characteristic.text {
                        body.save = save
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failed to parse save {}", save))?;
                    }
                }
                "InSv" => {
                    if let Some(ref invuln) = characteristic.text {
                        body.invuln = invuln
                            .replace("+", "")
                            .replace("-", "7")
                            .parse()
                            .with_context(|| format!("Failed to parse invuln {}", invuln))?;
                    }
                }
                "W" => {
                    if let Some(ref wounds) = characteristic.text {
                        body.wounds = wounds
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failed to parse wounds {}", wounds))?;
                    }
                }
                _ => (),
            }
        }

        Ok(body)
    }
}
