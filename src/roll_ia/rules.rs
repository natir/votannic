use super::profiles;

/// Abilities triggered on an unmodified "critical" roll (a natural 6).
#[derive(Clone, Debug, PartialEq)]
pub enum CritAbility {
    /// Critical hit: the hit automatically wounds (skips the Wound phase).
    LethalHits,
    /// Critical hit: the critical hit generates `n` additional hits.
    SustainedHits(u32),
    /// Critical wound: inflicts mortal wounds instead of normal damage
    /// (no save allowed, Feel No Pain still applies).
    DevastatingWounds,
}

/// Wound requirement from the S vs T table:
/// 2+ if S >= 2T, 3+ if S > T, 4+ if S == T, 5+/6+ otherwise.
pub fn wound_requirement(strength: u32, toughness: u32) -> u32 {
    if strength >= 2 * toughness {
        2
    } else if strength > toughness {
        3
    } else if strength == toughness {
        4
    } else if 2 * strength >= toughness {
        5
    } else {
        6
    }
}

/// Effective save: armour save worsened by AP, or the invulnerable save
/// if it is better. None when no save can be attempted.
pub fn effective_save(weapon: &profiles::Weapon, target: &profiles::UnitProfile) -> Option<u32> {
    // Negative AP worsens the armour save: Sv 3+ with AP -1 -> 4+.
    let armour = (target.save as i32).saturating_sub(weapon.ap).clamp(2, 7);
    match target.invulnerable {
        Some(inv) => Some(armour.min(inv as i32) as u32),
        None if armour >= 7 => None,
        None => Some(armour as u32),
    }
}
