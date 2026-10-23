//! Define the base of statistic

/* std use */

/* crate use */
use fraction;

/* project use */
use crate::roll;

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
)]
pub struct TheoricalDice<const S: usize> {}

impl<const S: usize> TheoricalDice<S> {
    #[hotpath::measure]
    pub fn new() -> Self {
        TheoricalDice {}
    }

    #[hotpath::measure]
    pub fn upper_than(&self, threshold: u64) -> roll::Result {
        roll::Result::new(
            // 2 -> 5/6 -> s - t + 1 -> 6 - 2 + 1 -> 5
            // 3 -> 4/6 -> s - t + 1 -> 6 - 3 + 1 -> 4
            // 4 -> 3/6 -> s - t + 1 -> 6 - 4 + 1 -> 3
            // 5 -> 2/6 -> s - t + 1 -> 6 - 5 + 1 -> 2
            // 6 -> 1/6 -> s - t + 1 -> 6 - 2 + 1 -> 1
            fraction::Fraction::new(1u8, 1u8) - self.lower_than(threshold).expectation,
            self.variance(),
        )
    }

    #[hotpath::measure]
    pub fn lower_than(&self, threshold: u64) -> roll::Result {
        roll::Result::new(
            // 2 -> 1/6 -> t - 1 -> 2 - 1 -> 1
            // 3 -> 2/6 -> t - 1 -> 3 - 1 -> 2
            // 4 -> 3/6 -> t - 1 -> 4 - 1 -> 3
            // 5 -> 4/6 -> t - 1 -> 5 - 1 -> 4
            // 6 -> 5/6 -> t - 1 -> 6 - 1 -> 5
            self.expectation_exact_match() * threshold.saturating_sub(1),
            self.variance(),
        )
    }

    pub fn expectation_exact_match(&self) -> fraction::Fraction {
        fraction::Fraction::new(1u64, { S } as u64)
    }

    pub fn variance(&self) -> fraction::Fraction {
        fraction::Fraction::new((S as u64).pow(2) - 1, 12u64)
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    #[test]
    fn theorical_dice() {
        let dice = TheoricalDice::<6>::new();

        assert_eq!(
            dice.expectation_exact_match(),
            fraction::Fraction::new(1u64, 6u64)
        );

        assert_eq!(
            dice.upper_than(2),
            roll::Result::new(
                fraction::Fraction::new(5u64, 6u64),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.upper_than(3),
            roll::Result::new(
                fraction::Fraction::new(4u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.upper_than(4),
            roll::Result::new(
                fraction::Fraction::new(3u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.upper_than(5),
            roll::Result::new(
                fraction::Fraction::new(2u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.upper_than(6),
            roll::Result::new(
                fraction::Fraction::new(1u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );

        assert_eq!(
            dice.lower_than(2),
            roll::Result::new(
                fraction::Fraction::new(1u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.lower_than(3),
            roll::Result::new(
                fraction::Fraction::new(2u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.lower_than(4),
            roll::Result::new(
                fraction::Fraction::new(3u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.lower_than(5),
            roll::Result::new(
                fraction::Fraction::new(4u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
        assert_eq!(
            dice.lower_than(6),
            roll::Result::new(
                fraction::Fraction::new(5u8, 6u8),
                fraction::Fraction::new(35u64, 12u64)
            )
        );
    }

    #[test]
    fn variance() {
        let dice = TheoricalDice::<6>::new();

        assert_eq!(dice.variance(), fraction::Fraction::new(35u64, 12u64))
    }
}
