//! Core statistics types for dice rolls
//!
//! This module provides the fundamental types for representing dice roll results
//! with precise fraction arithmetic.

use fraction::{Fraction, ToPrimitive};
use std::hash::Hash;

use crate::error;

/// Result of a dice roll or computation.
/// Contains both expectation and variance as fractions.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Default,
    Debug,
    Hash,
)]
pub struct Result {
    pub expectation: Fraction,
    pub variance: Fraction,
}

impl Result {
    pub fn new(expectation: Fraction, variance: Fraction) -> Result {
        Result {
            expectation,
            variance,
        }
    }
}

impl std::convert::From<u64> for Result {
    fn from(other: u64) -> Result {
        Result {
            expectation: Fraction::new(other, 1u64),
            variance: Fraction::new(1u64, 1u64),
        }
    }
}

impl std::str::FromStr for Result {
    type Err = anyhow::Error;

    #[hotpath::measure]
    fn from_str(s: &str) -> error::Result<Self> {
        let expr = tyche::Expr::from_str(s)?;
        Ok(crate::dice::DiceExpr::from(expr).to_result())
    }
}

impl std::ops::Add for Result {
    type Output = Result;

    fn add(self, rhs: Self) -> Self::Output {
        Result {
            expectation: self.expectation + rhs.expectation,
            variance: self.variance + rhs.variance,
        }
    }
}

impl std::ops::Sub for Result {
    type Output = Result;

    fn sub(self, rhs: Self) -> Self::Output {
        Result {
            expectation: self.expectation - rhs.expectation,
            variance: self.variance + rhs.variance,
        }
    }
}

impl std::ops::Mul for Result {
    type Output = Result;

    fn mul(self, rhs: Self) -> Self::Output {
        Result {
            expectation: self.expectation * rhs.expectation,
            variance: self.variance * rhs.variance,
        }
    }
}

impl std::ops::Mul<u64> for Result {
    type Output = Result;

    fn mul(self, rhs: u64) -> Self::Output {
        Result {
            expectation: self.expectation * Fraction::from(rhs),
            variance: self.variance,
        }
    }
}

impl std::ops::Mul<Fraction> for Result {
    type Output = Result;

    fn mul(self, rhs: Fraction) -> Self::Output {
        Result {
            expectation: self.expectation * rhs,
            variance: self.variance,
        }
    }
}

impl std::ops::Div for Result {
    type Output = Result;

    fn div(self, rhs: Self) -> Self::Output {
        Result {
            expectation: self.expectation / rhs.expectation,
            variance: self.variance / rhs.variance,
        }
    }
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({:.4}) +-{}({:.4})",
            self.expectation,
            self.expectation.to_f64().unwrap_or(0.0),
            self.variance,
            self.variance.to_f64().unwrap_or(0.0),
        )
    }
}

impl std::convert::From<tyche::Expr> for Result {
    #[hotpath::measure]
    fn from(expr: tyche::Expr) -> Result {
        crate::dice::DiceExpr::from(expr).to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(
            "0(0.0000) +-0(0.0000)".to_string(),
            format!("{}", Result::default()),
        );

        assert_eq!(
            "1/6(0.1667) +-35/12(2.9167)".to_string(),
            format!(
                "{}",
                Result::new(
                    Fraction::new(1u64, 6u64),
                    Fraction::new(35u64, 12u64),
                )
            ),
        );
    }
}
