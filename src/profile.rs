//! Define all struct required to modelise opposition of army.

/* std use */

/* crate use */
use rustc_hash;

use anyhow::Context as _;

/* module declaration */

/* project use */
use crate::error;
use crate::newrecruit;
use crate::roll;

use crate::roll::Roll as _;

/// Define an army as an associative map between name and unit
#[derive(std::clone::Clone, std::cmp::Eq, std::cmp::PartialEq, std::fmt::Debug)]
pub struct Army {
    pub units: rustc_hash::FxHashMap<String, Unit>,
}

impl std::convert::TryFrom<&newrecruit::NewRecruit> for Army {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &newrecruit::NewRecruit) -> error::Result<Army> {
        let mut units = rustc_hash::FxHashMap::default();

        for force in other.roster.forces.iter() {
            for selection in force.selections.iter() {
                let unit = Unit::try_from(selection)
                    .with_context(|| format!("Unit creation failled {}", selection.name))?;

                if !unit.models.is_empty() {
                    units.insert(selection.name.to_string(), unit);
                }
            }
        }

        Ok(Army { units })
    }
}

/// Define a unit as a set of tuple Model and number of this model
#[derive(std::clone::Clone, std::cmp::Eq, std::cmp::PartialEq, std::fmt::Debug)]
pub struct Unit {
    pub models: rustc_hash::FxHashSet<(Model, u8)>,
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
        if self.current.is_none() {
            let next = self.inner.next()?;

            self.current = Some(next.0.range2weapons().weapons());
            self.current_n = next.1;

            if let Some(ref mut iterator) = self.current {
                if let Some((_name, weapon)) = iterator.next() {
                    return Some((self.current_n, weapon));
                } else {
                    return self.next();
                }
            }
        }

        None
    }
}

fn parse_weapon_profile(
    profiles: &[newrecruit::Profile],
    range2weapons: &mut [Vec<(String, Weapon)>; MAX_RANGE],
) -> error::Result<()> {
    for profile in profiles {
        if profile.type_name.ends_with("Weapons") {
            let mut weapon = Weapon::try_from(profile.characteristics.as_slice())
                .with_context(|| format!("Weapon creation failled {}", profile.name))?;
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
                .with_context(|| format!("Body creation failled {}", profile.name))?;
            if !profile.name.is_empty() {
                name = profile.name.clone();
            }
        }
    }

    Ok((body, name))
}

impl std::convert::TryFrom<&newrecruit::Selection> for Unit {
    type Error = anyhow::Error;

    #[hotpath::measure]
    fn try_from(other: &newrecruit::Selection) -> error::Result<Unit> {
        let mut models = rustc_hash::FxHashSet::default();
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

            if !range2weapons.is_empty() {
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

const MAX_RANGE: usize = 60;
/// Type wrapper to improve static array of Vec
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::hash::Hash,
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
                || self.subidx > self.slice[self.range_idx].len()
            {
                self.range_idx += 1;
                self.subidx = 0;
                continue;
            }

            return Some(self.slice[self.range_idx][self.subidx].clone());
        }

        None
    }
}

/// Define a model as list of list of attack index by range
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::hash::Hash,
    std::fmt::Debug,
)]
pub struct Model {
    range2weapons: Range2Weapons,
    body: Body,
    name: String,
}

impl std::default::Default for Model {
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

/// Define a weapons profile
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::hash::Hash,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct Weapon {
    /// Weapon name
    pub name: String,

    /// Range weapon
    pub range: u64,

    /// Number of attack trow
    pub attack: roll::Result,

    /// Skill shot/close hit threshold
    pub skill: u64,

    /// Strength of weapons
    pub strength: u64,

    /// Armor perforation
    pub ap: u64,

    /// Damage for not save wound
    pub damage: roll::Result,
}

impl std::convert::TryFrom<&[newrecruit::Characteristic]> for Weapon {
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
                            .parse()
                            .with_context(|| format!("Failled to parse range {}", range))?;
                    }
                }
                "A" => {
                    if let Some(ref attack) = characteristic.text {
                        weapon.attack = roll::Result::from(
                            attack
                                .parse::<tyche::Expr>()
                                .with_context(|| format!("Failled to parse attack {}", attack))?,
                        );
                    }
                }
                "WS" => {
                    if let Some(ref skill) = characteristic.text {
                        weapon.skill = skill
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failled to parse skill wS {}", skill))?;
                    }
                }
                "BS" => {
                    if let Some(ref skill) = characteristic.text {
                        weapon.skill = skill
                            .replace("+", "")
                            .replace("N/A", "0")
                            .parse()
                            .with_context(|| format!("Failled to parse skill BS {}", skill))?;
                    }
                }
                "AP" => {
                    if let Some(ref ap) = characteristic.text {
                        weapon.ap = ap
                            .replace("-", "")
                            .parse()
                            .with_context(|| format!("Failled to parse AP {}", ap))?;
                    }
                }
                "S" => {
                    if let Some(ref strength) = characteristic.text {
                        weapon.strength = strength
                            .parse()
                            .with_context(|| format!("Failled to parse S {}", strength))?;
                    }
                }
                "D" => {
                    if let Some(ref damage) = characteristic.text {
                        weapon.damage = roll::Result::from(
                            damage
                                .to_lowercase()
                                .parse::<tyche::Expr>()
                                .with_context(|| format!("Failled to parse D {}", damage))?,
                        );
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
    pub fn run(&self, n: u64, body: &Body) -> roll::Result {
        let full = roll::FullRoll::builder()
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

/// Body of Model
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::hash::Hash,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
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

impl std::default::Default for Body {
    #[hotpath::measure]
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

impl std::convert::TryFrom<&[newrecruit::Characteristic]> for Body {
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
                            .with_context(|| format!("Failled to parse toughness {}", toughness))?;
                    }
                }
                "Sv" => {
                    if let Some(ref save) = characteristic.text {
                        body.save = save
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failled to parse save {}", save))?;
                    }
                }
                "InSv" => {
                    if let Some(ref invuln) = characteristic.text {
                        body.invuln = invuln
                            .replace("+", "")
                            .replace("-", "7")
                            .parse()
                            .with_context(|| format!("Failled to parse invuln {}", invuln))?;
                    }
                }
                "W" => {
                    if let Some(ref wounds) = characteristic.text {
                        body.wounds = wounds
                            .replace("+", "")
                            .parse()
                            .with_context(|| format!("Failled to parse wounds {}", wounds))?;
                    }
                }
                _ => (),
            }
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    fn get_weapon() -> error::Result<Weapon> {
        let mut weapon = Weapon::try_from(get_weapon_characteristic().as_slice())?;
        weapon.name = "Weapons".to_string();

        Ok(weapon)
    }

    fn get_weapon_characteristic() -> Vec<newrecruit::Characteristic> {
        vec![
            newrecruit::Characteristic {
                text: Some("Melee".to_string()),
                name: "Range".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("3d6".to_string()),
                name: "A".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("4".to_string()),
                name: "WS".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("4".to_string()),
                name: "BS".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("2".to_string()),
                name: "AP".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("4".to_string()),
                name: "S".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("2".to_string()),
                name: "D".to_string(),
                type_id: "type_id".to_string(),
            },
        ]
    }

    fn get_body() -> error::Result<Body> {
        Body::try_from(get_body_characteristic().as_slice())
    }

    fn get_body_characteristic() -> Vec<newrecruit::Characteristic> {
        vec![
            newrecruit::Characteristic {
                text: Some("6".to_string()),
                name: "T".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("5".to_string()),
                name: "Sv".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("6".to_string()),
                name: "InSv".to_string(),
                type_id: "type_id".to_string(),
            },
            newrecruit::Characteristic {
                text: Some("2".to_string()),
                name: "W".to_string(),
                type_id: "type_id".to_string(),
            },
        ]
    }

    fn get_range2weapons() -> error::Result<Range2Weapons> {
        let mut raw_data = [const { Vec::new() }; MAX_RANGE];
        raw_data[0].push(("Weapons".to_string(), get_weapon()?));

        Ok(Range2Weapons { data: raw_data })
    }

    fn get_selection() -> newrecruit::Selection {
        newrecruit::Selection {
            profiles: Some(vec![newrecruit::Profile {
                id: "test_id".to_string(),
                name: "name_id".to_string(),
                hidden: false,
                type_id: "test_type_id".to_string(),
                type_name: "Unit".to_string(),
                from: "test_from".to_string(),
                characteristics: get_body_characteristic(),
            }]),
            selections: Some(vec![newrecruit::Selection {
                profiles: Some(vec![newrecruit::Profile {
                    id: "test_id".to_string(),
                    name: "Weapons".to_string(),
                    hidden: false,
                    type_id: "test_type_id".to_string(),
                    type_name: "Weapons".to_string(),
                    from: "test_from".to_string(),
                    characteristics: get_weapon_characteristic(),
                }]),
                selections: None,
                rules: None,
                costs: None,
                categories: None,
                id: "selection".to_string(),
                name: "selection_test".to_string(),
                entry_id: "id_test".to_string(),
                entry_group_id: None,
                number: 1,
                type_id: Some("test".to_string()),
                type_name: Some("model".to_string()),
                from: "from".to_string(),
                group: None,
            }]),
            rules: None,
            costs: None,
            categories: None,
            id: "selection".to_string(),
            name: "test_unit".to_string(),
            entry_id: "id_test".to_string(),
            entry_group_id: None,
            number: 1,
            type_id: Some("test".to_string()),
            type_name: Some("model".to_string()),
            from: "from".to_string(),
            group: None,
        }
    }

    fn get_unit() -> error::Result<Unit> {
        let selection = get_selection();

        Unit::try_from(&selection)
    }

    fn get_army() -> error::Result<Army> {
        let mut units = rustc_hash::FxHashMap::default();

        units.insert("test_unit".to_string(), get_unit()?);

        Ok(Army { units })
    }

    #[test]
    fn army() -> error::Result<()> {
        let army = get_army()?;

        assert_eq!(
            army,
            Army::try_from(&newrecruit::NewRecruit {
                roster: newrecruit::Roster {
                    id: None,
                    name: "test_army".to_string(),
                    battle_scribe_version: 2.1,
                    generated_by: "test".to_string(),
                    game_system_id: "test".to_string(),
                    game_system_name: "test".to_string(),
                    game_system_revision: 0,
                    xmlns: "test".to_string(),
                    costs: Vec::new(),
                    cost_limits: Vec::new(),
                    forces: vec![newrecruit::Force {
                        id: "test".to_string(),
                        name: "test".to_string(),
                        entry_id: "test".to_string(),
                        catalogue_id: "test".to_string(),
                        catalogue_revision: 0,
                        catalogue_name: "test".to_string(),
                        selections: vec![get_selection()]
                    }]
                }
            })?
        );

        Ok(())
    }

    #[test]
    fn unit() -> error::Result<()> {
        let mut truth = rustc_hash::FxHashSet::default();
        truth.insert((
            Model {
                range2weapons: get_range2weapons()?,
                body: get_body()?,
                name: "name_id".to_string(),
            },
            1,
        ));
        let unit = get_unit()?;

        assert_eq!(unit, Unit { models: truth });

        Ok(())
    }

    #[test]
    fn range2weapons_check() -> error::Result<()> {
        let mut raw_data = [const { Vec::new() }; MAX_RANGE];

        raw_data[0].push(("Weapons".to_string(), get_weapon()?));

        let range2weapons = Range2Weapons { data: raw_data };

        assert_eq!(format!("{:?}", range2weapons), "[
\t0: [(\"Weapons\", Weapon { name: \"Weapons\", range: 0, attack: Result { expectation: Rational(Plus, Ratio { numer: 1, denom: 2 }), variance: Rational(Plus, Ratio { numer: 35, denom: 12 }) }, skill: 4, strength: 4, ap: 2, damage: Result { expectation: Rational(Plus, Ratio { numer: 2, denom: 1 }), variance: Rational(Plus, Ratio { numer: 1, denom: 1 }) } })]
]
".to_string());

        Ok(())
    }

    #[test]
    fn model() -> error::Result<()> {
        let model = Model::default();

        assert_eq!(
            model.range2weapons(),
            &Range2Weapons {
                data: [const { Vec::new() }; MAX_RANGE]
            }
        );
        assert_eq!(model.body(), &Body::default());
        assert_eq!(model.name(), "");

        Ok(())
    }

    #[test]
    fn weapon() -> error::Result<()> {
        let weapon = get_weapon()?;

        assert_eq!(weapon.range, 0);
        assert_eq!(
            weapon.attack,
            roll::Result::new(
                fraction::Fraction::new(1u64, 2u64),
                fraction::Fraction::new(35u64, 12u64),
            )
        );
        assert_eq!(weapon.skill, 4);
        assert_eq!(weapon.ap, 2);
        assert_eq!(
            weapon.damage,
            roll::Result::new(
                fraction::Fraction::new(2u8, 1u8),
                fraction::Fraction::new(1u64, 1u64),
            )
        );

        Ok(())
    }

    #[test]
    fn body() -> error::Result<()> {
        let body = Body::try_from(
            vec![
                newrecruit::Characteristic {
                    text: Some("6".to_string()),
                    name: "T".to_string(),
                    type_id: "type_id".to_string(),
                },
                newrecruit::Characteristic {
                    text: Some("5".to_string()),
                    name: "Sv".to_string(),
                    type_id: "type_id".to_string(),
                },
                newrecruit::Characteristic {
                    text: Some("6".to_string()),
                    name: "InSv".to_string(),
                    type_id: "type_id".to_string(),
                },
                newrecruit::Characteristic {
                    text: Some("2".to_string()),
                    name: "W".to_string(),
                    type_id: "type_id".to_string(),
                },
            ]
            .as_slice(),
        )?;

        assert_eq!(body.toughness, 6);
        assert_eq!(body.save, 5);
        assert_eq!(body.invuln, 6);
        assert_eq!(body.wounds, 2);

        Ok(())
    }
}
