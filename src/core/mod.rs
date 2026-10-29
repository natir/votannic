//! Core module - Types fondamentaux pour les calculs probabilistes

pub mod expectation;
pub mod stats;
pub mod theorical_dice;

pub use expectation::Expectation;
pub use stats::Result;
pub use theorical_dice::TheoricalDice;
