//! Models module - Domain models for Warhammer 40k
//!
//! This module contains all the domain models: Body, Weapon, Unit, Army

pub mod army;
pub mod body;
pub mod unit;
pub mod weapon;

pub use army::Army;
pub use body::Body;
pub use unit::{Model, Range2Weapons, RangeWeaponsIterator, WeaponIterator, Unit, MAX_RANGE};
pub use weapon::Weapon;

