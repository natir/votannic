//! Dice parsing module using tyche and fraction
//!
//! This module provides dice expression parsing using the `tyche` crate
//! and precise arithmetic using the `fraction` crate.

use fraction::{Fraction, ToPrimitive};
use std::collections::BTreeMap;
use std::str::FromStr;
use tyche::{Expr, Dice as TycheDice};

use crate::core::stats::Result;
use crate::error;

/// Dice expression that can be parsed from a string.
#[derive(Clone, Debug, PartialEq)]
pub struct DiceExpr {
    pub inner: Expr,
}

impl DiceExpr {
    /// Parse a dice expression from a string.
    pub fn parse(input: &str) -> error::Result<Self> {
        Ok(Self { inner: Expr::from_str(input)? })
    }
    
    /// Create from a tyche expression.
    pub fn new(expr: Expr) -> Self {
        Self { inner: expr }
    }
    
    /// Evaluate and return (expectation, variance).
    pub fn eval(&self) -> (Fraction, Fraction) {
        Self::eval_expr(&self.inner)
    }
    
    /// Recursively evaluate a tyche expression.
    fn eval_expr(expr: &Expr) -> (Fraction, Fraction) {
        match expr {
            Expr::Num(n) => (Fraction::from(*n as i64), Fraction::from(0u64)),
            Expr::Dice(dice) => {
                let count = u64::from(dice.count);
                let sides = u64::from(dice.sides);
                // E[nX] = n * (sides + 1) / 2
                let expectation = Fraction::from(count) * Fraction::from(sides + 1) / Fraction::from(2u64);
                // Var[nX] = n * (sides^2 - 1) / 12
                let variance = Fraction::from(count) * Fraction::new(sides * sides - 1, 12u64);
                (expectation, variance)
            }
            Expr::Neg(e) => {
                let (exp, var) = Self::eval_expr(e);
                (-exp, var)
            }
            Expr::Add(l, r) => {
                let (le, lv) = Self::eval_expr(l);
                let (re, rv) = Self::eval_expr(r);
                (le + re, lv + rv)
            }
            Expr::Sub(l, r) => {
                let (le, lv) = Self::eval_expr(l);
                let (re, rv) = Self::eval_expr(r);
                (le - re, lv + rv)
            }
            Expr::Mul(l, r) => {
                let (le, lv) = Self::eval_expr(l);
                let (re, rv) = Self::eval_expr(r);
                // Var[a*b] = E[a]^2 * Var[b] + E[b]^2 * Var[a] + Var[a] * Var[b]
                let expectation = le.clone() * re.clone();
                let variance = le.clone() * le * rv.clone() + re.clone() * re * lv.clone() + lv * rv;
                (expectation, variance)
            }
            Expr::DivDown(l, r) | Expr::DivUp(l, r) => {
                let (le, _) = Self::eval_expr(l);
                let (re, _) = Self::eval_expr(r);
                if re != Fraction::from(0u64) {
                    (le / re, Fraction::from(0u64))
                } else {
                    (Fraction::from(0u64), Fraction::from(0u64))
                }
            }
            _ => (Fraction::from(0u64), Fraction::from(0u64)),
        }
    }
    
    /// Get expectation value.
    pub fn expected_value(&self) -> Fraction {
        self.eval().0
    }
    
    /// Get variance.
    pub fn variance(&self) -> Fraction {
        self.eval().1
    }
    
    /// Convert to Result (expectation, variance).
    pub fn to_result(&self) -> Result {
        let (expectation, variance) = self.eval();
        Result::new(expectation, variance)
    }
    
    /// Get all possible outcomes with probabilities.
    pub fn distribution(&self) -> Vec<(i32, f64)> {
        Self::compute_distribution(&self.inner)
    }
    
    /// Compute full distribution of an expression.
    fn compute_distribution(expr: &Expr) -> Vec<(i32, f64)> {
        match expr {
            Expr::Num(n) => vec![(*n as i32, 1.0)],
            Expr::Dice(dice) => {
                if dice.count == 1 {
                    (1..=dice.sides as i32)
                        .map(|v| (v, 1.0 / dice.sides as f64))
                        .collect()
                } else {
                    let single: Vec<(i32, f64)> = (1..=dice.sides as i32)
                        .map(|v| (v, 1.0 / dice.sides as f64))
                        .collect();
                    convolve_n(&single, dice.count as usize - 1)
                }
            }
            Expr::Neg(e) => {
                Self::compute_distribution(e)
                    .into_iter()
                    .map(|(v, p)| (-v, p))
                    .collect()
            }
            Expr::Add(l, r) => {
                convolve_two(
                    &Self::compute_distribution(l),
                    &Self::compute_distribution(r)
                )
            }
            Expr::Sub(l, r) => {
                let rhs_dist: Vec<(i32, f64)> = Self::compute_distribution(r)
                    .into_iter()
                    .map(|(v, p)| (-v, p))
                    .collect();
                convolve_two(
                    &Self::compute_distribution(l),
                    &rhs_dist
                )
            }
            Expr::Mul(l, r) => {
                multiply_dist(
                    &Self::compute_distribution(l),
                    &Self::compute_distribution(r)
                )
            }
            Expr::DivDown(l, r) | Expr::DivUp(l, r) => {
                let lhs_val = Self::eval_expr(l).0;
                let rhs_val = Self::eval_expr(r).0;
                if rhs_val != Fraction::from(0u64) {
                    let result = lhs_val / rhs_val;
                    vec![(result.to_i64().unwrap_or(0) as i32, 1.0)]
                } else {
                    vec![(0, 1.0)]
                }
            }
            _ => vec![(0, 1.0)],
        }
    }
    
    /// Create a fixed value expression.
    pub fn fixed(value: i32) -> Self {
        Self::new(Expr::Num(value))
    }
    
    /// Create a dice expression: count d sides.
    pub fn dice(count: u8, sides: u8) -> Self {
        Self::new(Expr::Dice(TycheDice {
            count,
            sides,
            modifiers: vec![],
        }))
    }
}

/// Convolve two distributions.
fn convolve_two(dist1: &[(i32, f64)], dist2: &[(i32, f64)]) -> Vec<(i32, f64)> {
    let mut result = BTreeMap::new();
    for (v1, p1) in dist1 {
        for (v2, p2) in dist2 {
            *result.entry(v1 + v2).or_insert(0.0) += p1 * p2;
        }
    }
    result.into_iter().collect()
}

/// Convolve distribution n times.
fn convolve_n(dist: &[(i32, f64)], n: usize) -> Vec<(i32, f64)> {
    if n == 0 {
        return dist.to_vec();
    }
    let mut result = dist.to_vec();
    for _ in 0..n {
        result = convolve_two(dist, &result);
    }
    result
}

/// Multiply two distributions.
fn multiply_dist(dist1: &[(i32, f64)], dist2: &[(i32, f64)]) -> Vec<(i32, f64)> {
    let mut result = BTreeMap::new();
    for (v1, p1) in dist1 {
        for (v2, p2) in dist2 {
            *result.entry(v1 * v2).or_insert(0.0) += p1 * p2;
        }
    }
    result.into_iter().collect()
}

impl From<Expr> for DiceExpr {
    fn from(expr: Expr) -> Self {
        Self::new(expr)
    }
}

impl FromStr for DiceExpr {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> error::Result<Self> {
        Self::parse(s)
    }
}

impl From<DiceExpr> for Result {
    fn from(dice: DiceExpr) -> Self {
        dice.to_result()
    }
}

impl From<TycheDice> for Result {
    fn from(dice: TycheDice) -> Self {
        let count = u64::from(dice.count);
        let sides = u64::from(dice.sides);
        let expectation = Fraction::from(count) * Fraction::from(sides + 1) / Fraction::from(2u64);
        let variance = Fraction::from(count) * Fraction::new(sides * sides - 1, 12u64);
        Result::new(expectation, variance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let expr = DiceExpr::parse("2d6").unwrap();
        assert_eq!(expr.expected_value(), Fraction::new(7u64, 1u64));
        assert_eq!(expr.variance(), Fraction::new(35u64, 6u64));
    }

    #[test]
    fn parse_with_flat() {
        let expr = DiceExpr::parse("2d6+3").unwrap();
        assert_eq!(expr.expected_value(), Fraction::new(10u64, 1u64));
    }

    #[test]
    fn distribution_single_die() {
        let expr = DiceExpr::parse("1d6").unwrap();
        let dist = expr.distribution();
        assert_eq!(dist.len(), 6);
        for (value, prob) in dist {
            assert!((1..=6).contains(&value));
            assert!((prob - 1.0/6.0).abs() < 1e-10);
        }
    }

    #[test]
    fn fixed_value() {
        let expr = DiceExpr::fixed(5);
        assert!(expr.expected_value() == Fraction::from(5i64));
        assert!(expr.variance() == Fraction::from(0u64));
    }

    #[test]
    fn to_result() {
        let expr = DiceExpr::parse("2d6").unwrap();
        let result: Result = expr.into();
        assert_eq!(result.expectation, Fraction::new(7u64, 1u64));
        assert_eq!(result.variance, Fraction::new(35u64, 6u64));
    }
}
