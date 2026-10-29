//! Unit model for Warhammer 40k
//!
//! Represents a collection of models with weapons and bodies.

use anyhow::Context;
use rustc_hash::FxHashSet;

use crate::error;
use crate::models::{Body, Weapon};
use crate::newrecruit;

pub const MAX_RANGE: usize = 60;

/// Type wrapper to improve static array of Vec
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
)]
pub struct Range2Weapons {
    pub data: [Vec<(String, Weapon)>; MAX_RANGE],
}

impl Range2Weapons {
    #[hotpath::measure]
    pub fn weapons<'a>(&'a self) -> RangeWeaponsIterator<'a> {
        RangeWeaponsIterator {
            range_idx: 0,
            subidx: 0,
            slice: &self.data,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.data.iter().any(Vec::is_empty)
    }
}

impl std::fmt::Debug for Range2Weapons {
    #[hotpath::measure]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for index in 0..MAX_RANGE {
            if !self.data[index].is_empty() {
                writeln!(f, "\t{:?}: {:?}", index, self.data[index])?;
            }
        }

        writeln!(f, "]")
    }
}

pub struct RangeWeaponsIterator<'a> {
    pub range_idx: usize,
    pub subidx: usize,
    pub slice: &'a [Vec<(String, Weapon)>],
}

impl<'a> std::iter::Iterator for RangeWeaponsIterator<'a> {
    type Item = (String, Weapon);

    #[hotpath::measure]
    fn next(&mut self) -> std::option::Option<Self::Item> {
        while self.range_idx < MAX_RANGE {
            if self.slice[self.range_idx].is_empty()
                || self.subidx >= self.slice[self.range_idx].len()
            {
                self.range_idx += 1;
                self.subidx = 0;
                continue;
            }

            let result = Some(self.slice[self.range_idx][self.subidx].clone());
            self.subidx += 1;
            return result;
        }

        None
    }
}

/// Define a model as list of list of attack index by range
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
)]
pub struct Model {
    range2weapons: Range2Weapons,
    body: Body,
    name: String,
}

impl Default for Model {
    #[hotpath::measure]
    fn default() -> Self {
        Model {
            range2weapons: Range2Weapons {
                data: [const { Vec::new() }; MAX_RANGE],
            },
            body: Body::default(),
            name: "".to_string(),
        }
    }
}

impl Model {
    #[hotpath::measure]
    pub fn range2weapons(&self) -> &Range2Weapons {
        &self.range2weapons
    }

    #[hotpath::measure]
    pub fn body(&self) -> &Body {
        &self.body
    }

    #[hotpath::measure]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Define a unit as a set of tuple Model and number of this model
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Unit {
    pub models: FxHashSet<(Model, u8)>,
}

impl Unit {
    #[hotpath::measure]
    pub fn weapons<'a>(&'a self) -> WeaponIterator<'a> {
        WeaponIterator {
            inner: self.models.iter(),
            current: None,
            current_n: 0,
        }
    }
}

pub struct WeaponIterator<'a> {
    inner: std::collections::hash_set::Iter<'a, (Model, u8)>,
    current: Option<RangeWeaponsIterator<'a>>,
    current_n: u8,
}

impl<'a> std::iter::Iterator for WeaponIterator<'a> {
    type Item = (u8, Weapon);

    #[hotpath::measure]
    fn next(&mut self) -> std::option::Option<Self::Item> {
        loop {
            if self.current.is_none() {
                let next = self.inner.next()?;

                self.current = Some(next.0.range2weapons().weapons());
                self.current_n = next.1;
            }

            if let Some(ref mut iterator) = self.current {
                if let Some((_name, weapon)) = iterator.next() {
                    return Some((self.current_n, weapon));
                } else {
                    self.current = None;
                    continue;
                }
            }

            return None;
        }
    }
}

fn parse_weapon_profile(
    profiles: &[newrecruit::Profile],
    range2weapons: &mut [Vec<(String, Weapon)>; MAX_RANGE],
) -> error::Result<()> {
    for profile in profiles {
        if profile.type_name.ends_with("Weapons") {
            let mut weapon = Weapon::try_from(profile.characteristics.as_slice())
                .with_context(|| format!("Weapon creation failed {}", profile.name))?;
            weapon.name = profile.name.to_string();

            range2weapons[weapon.range as usize].push((profile.name.clone(), weapon));
        }
    }

    Ok(())
}

fn parse_body_profile(profiles: &[newrecruit::Profile]) -> error::Result<(Body, String)> {
    let mut body = Body::default();
    let mut name = "".to_string();

    for profile in profiles {
        if profile.type_name == "Unit" {
            body = Body::try_from(profile.characteristics.as_slice())
                .with_context(|| format!("Body creation failed {}", profile.name))?;
            if !profile.name.is_empty() {
                name = profile.name.clone();
            }
        }
    }

    Ok((body, name))
}

impl TryFrom<&newrecruit::Selection> for Unit {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &newrecruit::Selection) -> error::Result<Unit> {
        let mut models = FxHashSet::default();
        let mut range2weapons = [const { Vec::new() }; MAX_RANGE];
        let mut body = Body::default();
        let mut name = "".to_string();
        let mut body_number = 1;

        if let Some(ref selections) = other.selections {
            for selection in selections {
                body_number = selection.number;
            }
        }

        if let Some(ref profiles) = other.profiles {
            let (b, n) = parse_body_profile(profiles)?;
            if !n.is_empty() {
                body = b;
                name = n;
            }

            if let Some(ref selections) = other.selections {
                for selection in selections {
                    if let Some(ref profiles) = selection.profiles {
                        parse_weapon_profile(profiles, &mut range2weapons)?;
                        let (b, n) = parse_body_profile(profiles)?;
                        if !n.is_empty() {
                            body = b;
                            name = n;
                        }
                    }

                    if let Some(ref subselections) = selection.selections {
                        for subselection in subselections {
                            if let Some(ref subprofiles) = subselection.profiles {
                                parse_weapon_profile(subprofiles, &mut range2weapons)?;
                            }
                        }
                    }
                }
            }

            if !range2weapons.iter().all(Vec::is_empty) {
                let obj = (
                    Model {
                        range2weapons: Range2Weapons {
                            data: range2weapons,
                        },
                        body,
                        name,
                    },
                    body_number,
                );

                models.insert(obj);
            }
        }

        Ok(Unit { models })
    }
}
