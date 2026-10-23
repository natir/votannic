//! Define struct for modelise Roll.

/* std use */

/* crate use */
use fraction;

/* module declaration */
mod expectation;
mod result;
mod theorical_dice;

/* pub reexport */
pub use expectation::Expectation;
pub use result::Result;
pub use theorical_dice::TheoricalDice;

/* project use */
use crate::utils;

pub trait Roll {
    fn roll(&self) -> Result;
}

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct HitRoll {
    /// Number of dices
    number: result::Result,

    /// Success threshold
    threshold: u64,

    /// Critical threshold
    #[builder(default = u64::MAX)]
    critical_threshold: u64,

    /// Bonus add on critical
    #[builder(default = Expectation(fraction::Fraction::new(0u8, 1u8)))]
    critical_bonus: Expectation,
}

impl Roll for HitRoll {
    #[hotpath::measure]
    fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = dice.upper_than(self.threshold);

        let bonus = Result::new(
            self.critical_bonus.0 * dice.upper_than(self.critical_threshold).expectation,
            if self.critical_threshold != u64::MAX {
                dice.variance()
            } else {
                fraction::Fraction::new(1u64, 1u64)
            },
        );

        result * self.number + bonus
    }
}

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct WoundRoll {
    /// HitRoll
    hit_roll: HitRoll,

    /// Success threshold2
    threshold: u64,

    /// Critical threshold
    #[builder(default = u64::MAX)]
    critical_threshold: u64,

    /// Bonus add on critical
    #[builder(default = Expectation(fraction::Fraction::new(0u8, 1u8)))]
    critical_bonus: Expectation,
}

impl Roll for WoundRoll {
    #[hotpath::measure]
    fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let base = dice.upper_than(self.threshold);
        let bonus = Result::new(
            self.critical_bonus.0 * dice.upper_than(self.critical_threshold).expectation,
            if self.critical_threshold == u64::MAX {
                dice.variance()
            } else {
                fraction::Fraction::new(1u64, 1u64)
            },
        );

        Result {
            expectation: base.expectation + bonus.expectation,
            variance: base.variance + bonus.variance,
        }
    }
}

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct SaveRoll {
    /// WoundRoll
    wound_roll: WoundRoll,

    /// Success threshold
    threshold: u64,
}

impl Roll for SaveRoll {
    #[hotpath::measure]
    fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = self.wound_roll.roll();

        Result {
            expectation: result.expectation * dice.lower_than(self.threshold).expectation,
            variance: result.expectation * dice.variance(),
        }
    }
}

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct FNPRoll {
    /// SaveRoll
    save_roll: SaveRoll,

    /// Success threshold
    threshold: u64,

    /// Damage of weapons
    damage: Result,
}

impl Roll for FNPRoll {
    #[hotpath::measure]
    fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = self.save_roll.roll();
        let damage = result * self.damage;
        let fnp_result = dice.upper_than(self.threshold) * damage;

        Result {
            expectation: damage.expectation - fnp_result.expectation,
            variance: damage.variance + fnp_result.variance,
        }
    }
}

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct FullRoll {
    // Defend part
    /// Number of weapons
    weapons_number: u64,

    // wound part
    /// toughness threshold
    toughness: u64,

    // save part
    /// save
    #[builder(default = u64::MAX)]
    save: u64,

    /// Invulnerable save
    #[builder(default = u64::MAX)]
    invulnerable_save: u64,

    // fnp part
    /// fnp threshold
    #[builder(default = u64::MAX)]
    fnp_threshold: u64,

    // Attack part
    /// attack per weapon
    attack: Result,

    // hit part
    /// Skill
    skill: u64,

    /// Hit critical threshold
    #[builder(default = u64::MAX)]
    hit_critical_threshold: u64,

    /// Hit Bonus add on critical
    #[builder(default = Expectation(fraction::Fraction::new(0u64, 1u64)))]
    hit_critical_bonus: Expectation,

    // wound part
    /// strength
    strength: u64,

    /// Wound critical threshold
    #[builder(default = u64::MAX)]
    wound_critical_threshold: u64,

    /// Wound Bonus add on critical
    #[builder(default = Result::new(fraction::Fraction::new(0u64, 1u64), fraction::Fraction::new(1u64, 1u64)))]
    wound_critical_bonus: Result,

    // save part
    /// Damage of weapons
    #[builder(default = 0)]
    ap: u64,

    // fnp part
    /// Damage of weapons
    #[builder(default = Result::new(fraction::Fraction::new(1u64, 1u64), fraction::Fraction::new(1u64, 1u64)))]
    damage: Result,
}

impl Roll for FullRoll {
    #[hotpath::measure]
    fn roll(&self) -> Result {
        log::info!("Start a full roll");

        let hitroll = self.build_hitroll();
        log::info!("Hitroll result: {}", hitroll.roll());

        let woundroll = self.build_woundroll(hitroll);
        log::info!("Woundroll result: {}", woundroll.roll());

        let saveroll = self.build_saveroll(woundroll);
        log::info!("Saveroll result: {}", saveroll.roll());

        let fnproll = self.build_fnproll(saveroll);
        log::info!("FNProll result: {}", fnproll.roll());

        log::info!("End full roll: {}", fnproll.roll());
        fnproll.roll()
    }
}

impl FullRoll {
    #[hotpath::measure]
    pub fn build_hitroll(&self) -> HitRoll {
        log::info!(
            "Hitroll parameter: number: {}, attack: {}, threshold: {}",
            self.weapons_number,
            self.attack,
            self.skill
        );

        HitRoll::builder()
            .number(self.attack * self.weapons_number)
            .threshold(self.skill)
            .build()
    }

    #[hotpath::measure]
    pub fn build_woundroll(&self, hitroll: HitRoll) -> WoundRoll {
        log::info!(
            "Woundroll parameter: hitroll: {}, strength: {}, toughness: {}, threshold: {}",
            hitroll.roll(),
            self.strength,
            self.toughness,
            utils::wound_threshold(self.strength, self.toughness)
        );
        WoundRoll::builder()
            .hit_roll(hitroll)
            .threshold(utils::wound_threshold(self.strength, self.toughness))
            .build()
    }

    #[hotpath::measure]
    pub fn build_saveroll(&self, woundroll: WoundRoll) -> SaveRoll {
        log::info!(
            "Saveroll parameter: woundroll: {}, save: {}, inv: {}, ap: {}",
            woundroll.roll(),
            self.save,
            self.invulnerable_save,
            self.ap,
        );
        SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(self.invulnerable_save.max(self.save - self.ap))
            .build()
    }

    #[hotpath::measure]
    pub fn build_fnproll(&self, saveroll: SaveRoll) -> FNPRoll {
        log::info!(
            "FNProll parameter: saveroll: {}, damage: {:?}, threshold: {}",
            saveroll.roll(),
            self.damage,
            self.fnp_threshold
        );
        FNPRoll::builder()
            .save_roll(saveroll)
            .threshold(self.fnp_threshold)
            .damage(self.damage)
            .build()
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    #[test]
    fn hit_roll() {
        let roll = HitRoll::builder().number(6.into()).threshold(4).build();
        assert_eq!(
            roll.roll(),
            Result::new(
                fraction::Fraction::new(3u64, 1u64),
                fraction::Fraction::new(47u64, 12u64)
            )
        );

        let roll = HitRoll::builder()
            .number(6.into())
            .threshold(4)
            .critical_threshold(6)
            .critical_bonus(1u64.into())
            .build();

        assert_eq!(
            roll.roll(),
            Result::new(
                fraction::Fraction::new(19u64, 6u64),
                fraction::Fraction::new(35u64, 6u64)
            )
        );
    }

    #[test]
    fn wound_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();

        assert_eq!(
            woundroll.roll(),
            Result::new(
                fraction::Fraction::new(1u8, 2u8),
                fraction::Fraction::new(35u64, 6u64)
            )
        );

        let woundroll = WoundRoll::builder()
            .hit_roll(hitroll)
            .threshold(4)
            .critical_threshold(6)
            .critical_bonus(1u64.into())
            .build();
        assert_eq!(
            woundroll.roll(),
            Result::new(
                fraction::Fraction::new(2u8, 3u8),
                fraction::Fraction::new(47u64, 12u64)
            )
        );
    }

    #[test]
    fn save_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();
        let saveroll = SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(4)
            .build();

        assert_eq!(
            saveroll.roll(),
            Result::new(
                fraction::Fraction::new(1u8, 4u8),
                fraction::Fraction::new(35u64, 24u64)
            )
        );
    }

    #[test]
    fn fnp_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();
        let saveroll = SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(4)
            .build();
        let fnproll = FNPRoll::builder()
            .save_roll(saveroll)
            .threshold(6)
            .damage(1u64.into())
            .build();

        assert_eq!(
            fnproll.roll(),
            Result::new(
                fraction::Fraction::new(5u8, 24u8),
                fraction::Fraction::new(1645u64, 288u64)
            )
        );
    }
}
