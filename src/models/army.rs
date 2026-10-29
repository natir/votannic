//! Army model for Warhammer 40k
//!
//! Represents a collection of units.

use rustc_hash::FxHashMap;
use anyhow::Context as _;

use crate::error;
use crate::newrecruit;
use crate::models::Unit;

/// Define an army as an associative map between name and unit
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Army {
    pub units: FxHashMap<String, Unit>,
}

impl TryFrom<&newrecruit::NewRecruit> for Army {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &newrecruit::NewRecruit) -> error::Result<Army> {
        let mut units = FxHashMap::default();

        for force in other.roster.forces.iter() {
            for selection in force.selections.iter() {
                let unit = Unit::try_from(selection)
                    .with_context(|| format!("Unit creation failed {}", selection.name))?;

                if !unit.models.is_empty() {
                    units.insert(selection.name.to_string(), unit);
                }
            }
        }

        Ok(Army { units })
    }
}
