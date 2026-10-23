//! Define struct for modelise Roll.

/* std use */

/* crate use */
use fraction;

/* module declaration */

/* pub reexport */

/* project use */

#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub struct Expectation(pub fraction::Fraction);

impl std::default::Default for Expectation {
    #[hotpath::measure]
    fn default() -> Self {
        Expectation(fraction::Fraction::new(1u64, 1u64))
    }
}

impl std::convert::From<u64> for Expectation {
    fn from(f: u64) -> Expectation {
        Expectation(fraction::Fraction::new(f, 1u64))
    }
}

impl std::convert::From<i8> for Expectation {
    #[hotpath::measure]
    fn from(f: i8) -> Expectation {
        if f < 0 {
            Expectation(fraction::Fraction::new_neg(f.unsigned_abs(), 1u64))
        } else {
            Expectation(fraction::Fraction::new(f.unsigned_abs(), 1u64))
        }
    }
}

impl std::convert::From<fraction::Fraction> for Expectation {
    #[hotpath::measure]
    fn from(f: fraction::Fraction) -> Expectation {
        Expectation(f)
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;
    use crate::error;

    #[test]
    fn expectation() -> error::Result<()> {
        assert_eq!(
            Expectation::default(),
            Expectation(fraction::Fraction::new(1u64, 1u64))
        );
        assert_eq!(
            fraction::Fraction::new(2u64, 1u64),
            Expectation::from(2u64).0
        );
        assert_eq!(
            fraction::Fraction::new_neg(2u64, 1u64),
            Expectation::from(-2i8).0,
        );
        assert_eq!(
            fraction::Fraction::new(1u64, 4u64),
            Expectation(fraction::Fraction::new(1u64, 4u64)).0,
        );

        assert_eq!(
            Expectation(fraction::Fraction::new(1u64, 1u64)),
            Expectation::from(1i8)
        );

        assert_eq!(
            Expectation(fraction::Fraction::new(1u64, 2u64)),
            Expectation::from(fraction::Fraction::new(1u64, 2u64))
        );

        Ok(())
    }
}
