//! Define a ast of dice parser thank to https://github.com/marcell-ziegler/dice-parser/.

/* std use */

use std::str::FromStr as _;

/* crate use */

/* module declaration */

/* project use */
use crate::dice_parser;
use crate::error;

/// A parsed dice expression that can be evaluated to produce a result.
///
/// This is the main type for working with dice expressions. It represents
/// a tree structure of dice rolls, literals, and arithmetic operations.
///
/// # Variants
///
/// - `Sum`: Addition of two sub-expressions (e.g., "2d6 + 3")
/// - `Difference`: Subtraction of two sub-expressions (e.g., "1d20 - 2")
/// - `Roll`: A dice roll specification (e.g., "2d6")
/// - `Literal`: A constant integer value (e.g., "5")
///
/// # Examples
///
/// ## Parsing from a string
///
/// ```
/// use votannic::dice_parser::ast::DiceExpr;
///
/// let expr = DiceExpr::parse("2d6+3").unwrap();
/// ```
#[derive(Debug, Clone)]
pub enum DiceExpr {
    /// Addition of two dice expressions.
    Sum(Box<DiceExpr>, Box<DiceExpr>),
    /// Subtraction of two dice expressions.
    Difference(Box<DiceExpr>, Box<DiceExpr>),
    /// A dice roll specification.
    Roll(RollSpec),
    /// A constant integer literal.
    Literal(i32),
}

impl std::str::FromStr for DiceExpr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> error::Result<Self> {
        let mut parser = dice_parser::Parser::new(s);
        parser.parse()
    }
}

impl DiceExpr {
    /// Parse a dice expression from a string.
    ///
    /// This is the prefred way to create a `DiceExpr`. This is especially useful when parsin user input.
    /// The parser supports a subset standard dice notation with addition and subtraction.
    ///
    /// # Supported Syntax
    ///
    /// - Dice rolls: `NdS` where N is the number of dice and S is the number of sides
    /// - Literals: Any integer (positive or negative)
    /// - Addition: `expr + expr`
    /// - Subtraction: `expr - expr`
    /// - Whitespace is ignored
    ///
    /// Note: Keep mechanics (e.g., "2d20kh" for keep highest, "6d6kl3" for keep lowest 3)
    /// are planned for a future release. Currently, use manual construction with
    /// `RollSpec` and `Keep` for keep functionality.
    ///
    /// # Parameters
    ///
    /// - `input`: A string slice containing the dice expression
    ///
    /// # Returns
    ///
    /// - `Ok(DiceExpr)`: The parsed expression
    /// - `Err(DiceError)`: If the input is malformed or contains syntax errors
    ///
    /// # Examples
    ///
    /// ```
    /// use votannic::dice_parser::ast::DiceExpr;
    ///
    /// // Simple dice roll
    /// let expr = DiceExpr::parse("2d6").unwrap();
    ///
    /// // With addition
    /// let expr = DiceExpr::parse("1d20 + 5").unwrap();
    ///
    /// // Complex expression
    /// let expr = DiceExpr::parse("2d6 + 1d4 - 2").unwrap();
    ///
    /// // Negative dice count is not allowed
    /// assert!(DiceExpr::parse("-2d6").is_err());
    /// ```
    pub fn parse(input: &str) -> error::Result<Self> {
        DiceExpr::from_str(input)
    }
}

/// A specification for rolling one or more dice.
///
/// This struct defines how many dice to roll, how many sides each die has,
/// and optionally which dice to keep (for advantage/disadvantage mechanics).
///
/// # Fields
///
/// - `count`: The number of dice to roll
/// - `sides`: The number of sides on each die
/// - `keep`: Optional keep modifier (keep highest N or lowest N)
///
/// # Examples
///
/// ## Basic dice roll
///
/// ```
/// use votannic::dice_parser::ast::RollSpec;
///
/// // Roll 2 six-sided dice
/// let spec = RollSpec::new(2, 6, None);
/// ```
///
/// ## Keep highest (advantage)
///
/// ```
/// use votannic::dice_parser::ast::{RollSpec, Keep};
///
/// // Roll 4d6, keep highest 3 (common for D&D ability scores)
/// let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
/// ```
///
/// ## Keep lowest (disadvantage)
///
/// ```
/// use votannic::dice_parser::ast::{RollSpec, Keep};
///
/// // Roll 2d20, keep lowest 1 (disadvantage in D&D)
/// let spec = RollSpec::new(2, 20, Some(Keep::Lowest(1)));
/// ```
#[derive(Debug, Clone)]
pub struct RollSpec {
    /// The number of dice to roll.
    pub count: u32,
    /// The number of sides on each die.
    pub sides: u32,
    /// Optional modifier to keep only highest or lowest N dice.
    pub keep: Option<Keep>,
}

impl RollSpec {
    /// Create a new roll specification.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of dice to roll
    /// - `sides`: The number of sides on each die
    /// - `keep`: Optional keep modifier
    ///
    /// # Examples
    ///
    /// ```
    /// use votannic::dice_parser::ast::{RollSpec, Keep};
    ///
    /// // Simple 2d6 roll
    /// let spec = RollSpec::new(2, 6, None);
    ///
    /// // 4d6 keep highest 3
    /// let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    /// ```
    pub fn new(count: u32, sides: u32, keep: Option<Keep>) -> Self {
        RollSpec { count, sides, keep }
    }
}

/// Specifies which dice to keep from a roll.
///
/// This enum is used with `RollSpec` to implement advantage/disadvantage
/// mechanics or other "keep best/worst N" scenarios.
///
/// # Variants
///
/// - `Highest(N)`: Keep the N highest dice from the roll
/// - `Lowest(N)`: Keep the N lowest dice from the roll
///
/// # Examples
///
/// ```
/// use votannic::dice_parser::ast::{DiceExpr, RollSpec, Keep};
///
/// // D&D 5e advantage: roll 2d20, keep highest 1
/// let advantage = RollSpec::new(2, 20, Some(Keep::Highest(1)));
/// let expr = DiceExpr::Roll(advantage);
///
/// // D&D ability scores: roll 4d6, keep highest 3
/// let ability_roll = RollSpec::new(4, 6, Some(Keep::Highest(3)));
/// let expr = DiceExpr::Roll(ability_roll);
///
/// // Keep the lowest roll (disadvantage)
/// let disadvantage = RollSpec::new(2, 20, Some(Keep::Lowest(1)));
/// let expr = DiceExpr::Roll(disadvantage);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keep {
    /// Keep the N highest dice from the roll.
    Highest(u32),
    /// Keep the N lowest dice from the roll.
    Lowest(u32),
}
