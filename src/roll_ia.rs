//! Warhammer 40,000 (v11) attack resolution as a DAG of random experiments.
//!
//! Cargo.toml:
//! ```toml
//! [dependencies]
//! petgraph = "0.6"
//! ```
//!
//! Every node of the graph carries ONE outgoing edge per possible outcome
//! of the experiment it models:
//!   - Bernoulli experiment -> 2 children: success (p) / failure (1 - p),
//!   - N-faced dice roll    -> one child per possible total, each with its
//!     own probability,
//!   - terminal node        -> 0 child (the concrete outcome).
//!
//! The number of attacks and the damage of a weapon may both be dice
//! expressions (e.g. A = 1d4+2, D = 1d3), modeled with the same
//! "one child per outcome" rule.
//!
//! Module layout (single-file crate, inline modules):
//! ```text
//! wh40k_attack_dag
//! +-- expr     : dice expressions and outcome distributions
//! +-- rules    : v11 resolution tables and critical-hit abilities
//! +-- profiles : the opposing unit inputs (Weapon / UnitProfile)
//! +-- dag      : DAG types (nodes, edges) and graph builders
//! +-- sim      : expected-value computation and Monte Carlo simulation
//! ```

/* module declaration */
pub mod dag;
pub mod expr;
pub mod profiles;
pub mod rules;
pub mod sim;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn main() {
        // Example 1: bolt rifle (Sustained Hits 1) against Ork Boyz.
        let bolt_rifle = profiles::Weapon {
            name: "Bolt rifle".into(),
            attacks: expr::DiceExpr::fixed(2),
            skill: 3, // 3+
            strength: 4,
            ap: -1,
            damage: expr::DiceExpr::fixed(1),
            abilities: vec![rules::CritAbility::SustainedHits(1)],
        };
        let ork_boyz = profiles::UnitProfile {
            name: "Ork Boyz".into(),
            toughness: 5,
            save: 6, // 6+
            invulnerable: None,
            feel_no_pain: None,
            wounds: 2,
            models: 10,
        };

        // Example 2: melee weapon with Lethal Hits + Devastating Wounds and
        // random profiles (A = 1d4+2, D = 1d3), against a unit with a 4++
        // invulnerable save and a 6+ Feel No Pain.
        let relic_blade = profiles::Weapon {
            name: "Relic blade (demo)".into(),
            attacks: expr::DiceExpr::dice(1, 4, 2), // 1d4+2 attacks
            skill: 2,                               // 2+
            strength: 8,
            ap: -3,
            damage: expr::DiceExpr::dice(1, 3, 0), // 1d3 damage
            abilities: vec![
                rules::CritAbility::LethalHits,
                rules::CritAbility::DevastatingWounds,
                rules::CritAbility::SustainedHits(1),
            ],
        };
        let terminators = profiles::UnitProfile {
            name: "Terminators (demo)".into(),
            toughness: 5,
            save: 2,               // 2+
            invulnerable: Some(4), // 4++ ignores AP
            feel_no_pain: Some(6), // 6+ FNP
            wounds: 3,
            models: 5,
        };

        let matchups = [(&bolt_rifle, &ork_boyz), (&relic_blade, &terminators)];

        for (weapon, target) in matchups {
            println!("==========================================================");
            println!(
                "{} (A={} {}+ S{} AP{} D={}) vs {} (T{} Sv{}{} FNP {})",
                weapon.name,
                weapon.attacks.label(),
                weapon.skill,
                weapon.strength,
                weapon.ap,
                weapon.damage.label(),
                target.name,
                target.toughness,
                target.save,
                target
                    .invulnerable
                    .map(|i| format!("{i}++"))
                    .unwrap_or_default(),
                target
                    .feel_no_pain
                    .map(|t| format!("{t}+"))
                    .unwrap_or_else(|| "-".into()),
            );
            println!(
                "Wound requirement: {}+",
                rules::wound_requirement(weapon.strength, target.toughness)
            );

            let simulation = sim::build_attack_simulation(weapon, target);

            let dot = petgraph::dot::Dot::new(&simulation.attack.graph);
            assert_eq!("digraph {
    0 [ label = \"To-hit roll: critical (unmodified 6)?\" ]
    1 [ label = \"To-hit roll (p=0.6)\" ]
    2 [ label = \"To-wound roll: critical (unmodified 6)?\" ]
    3 [ label = \"To-wound roll (p=0.2)\" ]
    4 [ label = \"Saving throw (p=0)\" ]
    5 [ label = \"Feel No Pain, normal damage (p=0)\" ]
    6 [ label = \"Feel No Pain, mortal wounds (p=0)\" ]
    7 [ label = \"Damage roll (one child per outcome)\" ]
    8 [ label = \"Terminal: Damage(1)\" ]
    9 [ label = \"Mortal wounds roll (one child per outcome)\" ]
    10 [ label = \"Terminal: MortalWounds(1)\" ]
    11 [ label = \"Terminal: Miss\" ]
    12 [ label = \"Terminal: NoWound\" ]
    13 [ label = \"Terminal: Saved\" ]
    14 [ label = \"Terminal: IgnoredByPain\" ]
    7 -> 8 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 1 }), effect: None }\" ]
    9 -> 10 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 1 }), effect: None }\" ]
    0 -> 2 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 6 }), effect: ExtraHits { count: 1, chain: NodeIndex(2) } }\" ]
    0 -> 1 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 5, denom: 6 }), effect: None }\" ]
    1 -> 2 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 3, denom: 5 }), effect: None }\" ]
    1 -> 11 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 2, denom: 5 }), effect: None }\" ]
    2 -> 4 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 6 }), effect: None }\" ]
    2 -> 3 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 5, denom: 6 }), effect: None }\" ]
    3 -> 4 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 5 }), effect: None }\" ]
    3 -> 12 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 4, denom: 5 }), effect: None }\" ]
    4 -> 13 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 0, denom: 1 }), effect: None }\" ]
    4 -> 5 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 1 }), effect: None }\" ]
    5 -> 14 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 0, denom: 1 }), effect: None }\" ]
    5 -> 7 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 1 }), effect: None }\" ]
    6 -> 14 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 0, denom: 1 }), effect: None }\" ]
    6 -> 9 [ label = \"EdgeData { prob: Rational(Plus, Ratio { numer: 1, denom: 1 }), effect: None }\" ]
}

}
".to_string(), format!("{}", dot));

            let ev = sim::expected_damage(&simulation, weapon);
            let mc = sim::simulate_damage(&simulation, weapon, 200_000, 0xDEADBEEF);
            assert_eq!(
                sim::expected_attacks(&simulation, weapon),
                fraction::Fraction::new(2u64, 1u64),
                "Expected attacks",
            );
            assert_eq!(ev, fraction::Fraction::new(5u64, 9u64), "Expected damage");
            assert_eq!(
                mc,
                fraction::Fraction::new(110973u64, 200000u64),
                "Monte Carlo estimate"
            );
        }

        panic!();
    }
}
