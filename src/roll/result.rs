//! Define result of roll

/* std use */

/* crate use */
use fraction::ToPrimitive as _;

/* project use */
use crate::error;
use crate::roll;

/// Containts result of roll
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::hash::Hash,
    std::fmt::Debug,
    std::default::Default,
)]
pub struct Result {
    pub expectation: fraction::Fraction,
    pub variance: fraction::Fraction,
}

impl Result {
    pub fn new(expectation: fraction::Fraction, variance: fraction::Fraction) -> Result {
        Result {
            expectation,
            variance,
        }
    }
}

impl std::convert::From<u64> for Result {
    fn from(other: u64) -> Result {
        Result {
            expectation: fraction::Fraction::new(other, 1u64),
            variance: fraction::Fraction::new(1u64, 1u64),
        }
    }
}

impl std::str::FromStr for Result {
    type Err = anyhow::Error;

    #[hotpath::measure]
    fn from_str(s: &str) -> error::Result<Self> {
        let expr = tyche::Expr::from_str(s)?;
        Ok(roll::Result::from(expr))
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
            variance: self.variance - rhs.variance,
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
            expectation: self.expectation * rhs,
            variance: self.variance,
        }
    }
}

impl std::ops::Mul<fraction::Fraction> for Result {
    type Output = Result;

    fn mul(self, rhs: fraction::Fraction) -> Self::Output {
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
            expectation: rhs.expectation / rhs.expectation,
            variance: rhs.variance / rhs.variance,
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
    fn from(f: tyche::Expr) -> Result {
        match f {
            tyche::Expr::Num(a) => Result::new(
                fraction::Fraction::new(a as u64, 1u64),
                fraction::Fraction::new(1u64, 1u64),
            ),
            tyche::Expr::Dice(dice) => Result::new(
                fraction::Fraction::new(dice.count, dice.sides),
                fraction::Fraction::new(dice.sides.pow(2) - 1, 12u64),
            ),
            tyche::Expr::Neg(expr) => {
                let neg = fraction::Fraction::new_neg(1u64, 1u64);
                let result = Result::from(*expr);

                Result {
                    expectation: result.expectation * neg,
                    variance: result.variance,
                }
            }
            tyche::Expr::Add(rhs, lhs) => Result::from(*rhs) + Result::from(*lhs),
            tyche::Expr::Sub(rhs, lhs) => Result::from(*rhs) - Result::from(*lhs),
            tyche::Expr::Mul(rhs, lhs) => Result::from(*rhs) * Result::from(*lhs),
            tyche::Expr::DivDown(rhs, lhs) => Result::from(*rhs) / Result::from(*lhs),
            tyche::Expr::DivUp(rhs, lhs) => Result::from(*rhs) / Result::from(*lhs),
            _ => error::unreachable!(file!(), line!()),
        }
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
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
                    fraction::Fraction::new(1u64, 6u64),
                    fraction::Fraction::new(35u64, 12u64),
                )
            ),
        );
    }
}
