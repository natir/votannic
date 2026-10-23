use fraction::ConstOne as _;
use fraction::ConstZero as _;
use petgraph::visit::EdgeRef as _;

use super::expr;
use super::profiles;
use super::rules;

/// What a Roll node produces.
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::marker::Copy,
    std::hash::Hash,
    std::fmt::Debug,
)]
pub enum RollKind {
    /// Number of attacks for the volley (value in the terminal).
    Attacks,
    /// Normal damage roll (after save + FNP failure).
    Damage,
    /// Mortal wounds roll (Devastating Wounds path).
    MortalWounds,
}

impl std::fmt::Display for RollKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A terminal outcome of the attack chain.
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::marker::Copy,
    std::hash::Hash,
    std::fmt::Debug,
)]
pub enum Outcome {
    /// The attack missed the to-hit roll.
    Miss,
    /// The hit produced no wound.
    NoWound,
    /// The save succeeded.
    Saved,
    /// Feel No Pain cancelled the damage.
    IgnoredByPain,
    /// Damage inflicted (in wounds).
    Damage(i32),
    /// Mortal wounds inflicted (Devastating Wounds).
    MortalWounds(i32),
    /// Raw value produced by a roll (attacks / damage dice).
    Value(i32),
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Outcome {
    /// Numeric value carried by a terminal outcome (wounds removed, or
    /// the raw value of a roll). Outcomes such as Miss / Saved carry 0.
    pub fn value_of(&self) -> fraction::Fraction {
        match self {
            Outcome::Damage(d) | Outcome::MortalWounds(d) => {
                fraction::Fraction::new(*d as u64, 1u64)
            }
            Outcome::Value(v) => fraction::Fraction::new(*v as u64, 1u64),
            _ => fraction::Fraction::new(0u64, 1u64),
        }
    }
}

#[derive(std::clone::Clone, std::cmp::PartialEq, std::cmp::PartialOrd, std::fmt::Debug)]
/// A node of the DAG: a random experiment, or a terminal outcome.
pub enum Phase {
    /// Dice roll node: one outgoing edge per possible total of the
    /// expression, each with its own probability.
    Roll { kind: RollKind },
    /// Bernoulli: "is it an unmodified 6?" on the to-hit roll. p = 1/6.
    HitCritical,
    /// Bernoulli: conditional to-hit roll (the 6 was already removed).
    Hit { p: fraction::Fraction },
    /// Bernoulli: "is it an unmodified 6?" on the to-wound roll. p = 1/6.
    WoundCritical,
    /// Bernoulli: conditional to-wound roll (the 6 was already removed).
    Wound { p: fraction::Fraction },
    /// Bernoulli: saving throw (armour or invulnerable).
    Save { p: fraction::Fraction },
    /// Bernoulli: Feel No Pain against normal damage.
    FeelNoPainNormal { p: fraction::Fraction },
    /// Bernoulli: Feel No Pain against mortal wounds.
    FeelNoPainMortal { p: fraction::Fraction },
    /// End of the attack chain.
    Terminal(Outcome),
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Roll { kind } => match kind {
                RollKind::Attacks => write!(f, "Attacks roll (one child per outcome)"),
                RollKind::Damage => write!(f, "Damage roll (one child per outcome)"),
                RollKind::MortalWounds => write!(f, "Mortal wounds roll (one child per outcome)"),
            },
            Phase::HitCritical => write!(f, "To-hit roll: critical (unmodified 6)?"),
            Phase::Hit { p } => write!(f, "To-hit roll (p={p:.3})"),
            Phase::WoundCritical => write!(f, "To-wound roll: critical (unmodified 6)?"),
            Phase::Wound { p } => write!(f, "To-wound roll (p={p:.3})"),
            Phase::Save { p } => write!(f, "Saving throw (p={p:.3})"),
            Phase::FeelNoPainNormal { p } => write!(f, "Feel No Pain, normal damage (p={p:})"),
            Phase::FeelNoPainMortal { p } => write!(f, "Feel No Pain, mortal wounds (p={p:})"),
            Phase::Terminal(o) => write!(f, "Terminal: {o:?}"),
        }
    }
}

/// Effect carried by an edge.
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::marker::Copy,
    std::hash::Hash,
    std::fmt::Debug,
)]
pub enum EdgeEffect {
    /// Plain edge, nothing special.
    None,
    /// Sustained Hits: the critical hit generates `count` extra hits that
    /// re-enter the wound chain at node `chain` (a downstream node, so
    /// the graph stays strictly acyclic).
    ExtraHits {
        count: u32,
        chain: petgraph::graph::NodeIndex,
    },
}

impl std::fmt::Display for EdgeEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Edge weight: probability of taking this edge from its source node,
/// plus an optional effect.
#[derive(
    std::clone::Clone, std::cmp::PartialEq, std::cmp::PartialOrd, std::marker::Copy, std::fmt::Debug,
)]
pub struct EdgeData {
    pub prob: fraction::Fraction,
    pub effect: EdgeEffect,
}

impl std::fmt::Display for EdgeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The DAG itself (acyclic by construction).
pub type Dag = petgraph::graph::Graph<Phase, EdgeData>;

#[derive(std::clone::Clone, std::fmt::Debug)]
/// A built DAG with its entry node.
pub struct AttackDag {
    pub graph: Dag,
    /// Entry node: the critical to-hit Bernoulli of one attack, or the
    /// roll node for the attacks / damage expression.
    pub entry: petgraph::graph::NodeIndex,
}

impl std::fmt::Display for AttackDag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Adds a Roll node for `expr` with ONE outgoing edge per possible total
/// of the expression, each weighted with the probability of that total.
/// The kind decides which terminal outcome each child leads to.
/// Example: 2d6 -> 11 children with the triangular distribution.
pub fn add_roll_node(
    graph: &mut Dag,
    expr: &expr::DiceExpr,
    kind: RollKind,
) -> petgraph::graph::NodeIndex {
    let node = graph.add_node(Phase::Roll { kind });
    for (value, prob) in expr.distribution() {
        let outcome = match kind {
            RollKind::Attacks => Outcome::Value(value),
            RollKind::Damage => Outcome::Damage(value),
            RollKind::MortalWounds => Outcome::MortalWounds(value),
        };
        let terminal = graph.add_node(Phase::Terminal(outcome));
        graph.add_edge(
            node,
            terminal,
            EdgeData {
                prob: prob.into(),
                effect: EdgeEffect::None,
            },
        );
    }
    node
}

/// Builds the resolution DAG of ONE attack (to be scaled by the number
/// of attacks).
///
/// Every to-hit / to-wound roll is decomposed into two successive
/// Bernoulli experiments:
///   1. "critical?" (the unmodified 6, p = 1/6)
///   2. if failed, "conditional success" over the 5 remaining faces.
///
/// This gives natural 6s a distinct destiny (Lethal, Sustained,
/// Devastating) without skewing the total probability of the roll.
pub fn build_attack_dag(weapon: &profiles::Weapon, target: &profiles::UnitProfile) -> AttackDag {
    let mut graph = Dag::new();

    // --- Probabilities derived from the profiles ---
    let p_crit = fraction::Fraction::new(1u64, 6u64); // an unmodified 6

    // To hit: faces BS..=5 among the 5 non-critical faces.
    let p_hit_cond = fraction::Fraction::new((6 - weapon.skill).clamp(0, 5), 5u64);

    // To wound: requirement from the S vs T table, faces req..=5 when
    // the roll is not a critical.
    let req = rules::wound_requirement(weapon.strength, target.toughness);
    let p_wound_cond = fraction::Fraction::new((6 - req as u64).clamp(0, 5), 5u64);

    // Save: armour vs AP, or invulnerable.
    let p_save = rules::effective_save(weapon, target)
        .map(|sv| fraction::Fraction::new((7 - sv) as u64, 6u64))
        .unwrap_or(fraction::Fraction::ZERO);

    // Feel No Pain.
    let p_fnp = target
        .feel_no_pain
        .map(|t| fraction::Fraction::new(7 - t, 6u64))
        .unwrap_or(fraction::Fraction::ZERO);

    let lethal = weapon.has(&rules::CritAbility::LethalHits);
    let devastating = weapon.has(&rules::CritAbility::DevastatingWounds);
    let sustained = weapon.sustained_hits();

    // --- Phase nodes (one child per possible outcome each) ---
    let hit_crit = graph.add_node(Phase::HitCritical);
    let hit = graph.add_node(Phase::Hit { p: p_hit_cond });
    let wound_crit = graph.add_node(Phase::WoundCritical);
    let wound = graph.add_node(Phase::Wound { p: p_wound_cond });
    let save = graph.add_node(Phase::Save { p: p_save });
    let fnp_normal = graph.add_node(Phase::FeelNoPainNormal { p: p_fnp });
    let fnp_mortal = graph.add_node(Phase::FeelNoPainMortal { p: p_fnp });

    // Damage rolls: the damage itself may be a dice expression; the
    // normal-damage and mortal-wounds paths get distinct roll nodes so
    // they lead to distinct terminals.
    let damage_roll = add_roll_node(&mut graph, &weapon.damage, RollKind::Damage);
    let mortal_roll = add_roll_node(&mut graph, &weapon.damage, RollKind::MortalWounds);

    // --- Terminal nodes ---
    let miss = graph.add_node(Phase::Terminal(Outcome::Miss));
    let no_wound = graph.add_node(Phase::Terminal(Outcome::NoWound));
    let saved = graph.add_node(Phase::Terminal(Outcome::Saved));
    let ignored = graph.add_node(Phase::Terminal(Outcome::IgnoredByPain));

    // --- Phase 2: to-hit roll ---
    // Critical hit:
    //   - Lethal Hits -> jump straight to the saving throw.
    //   - otherwise   -> normal wound chain.
    //   - Sustained Hits -> expected-value bonus: n extra hits
    //     re-entering the wound chain (a downstream edge, no cycle).
    let crit_effect = if sustained > 0 {
        EdgeEffect::ExtraHits {
            count: sustained,
            chain: wound_crit,
        }
    } else {
        EdgeEffect::None
    };
    graph.add_edge(
        hit_crit,
        if lethal { save } else { wound_crit },
        EdgeData {
            prob: p_crit,
            effect: crit_effect,
        },
    );
    graph.add_edge(
        hit_crit,
        hit,
        EdgeData {
            prob: fraction::Fraction::ONE - p_crit,
            effect: EdgeEffect::None,
        },
    );

    // Normal hit: success -> wound chain; failure -> miss.
    graph.add_edge(
        hit,
        wound_crit,
        EdgeData {
            prob: p_hit_cond,
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        hit,
        miss,
        EdgeData {
            prob: fraction::Fraction::ONE - p_hit_cond,
            effect: EdgeEffect::None,
        },
    );

    // --- Phase 3: to-wound roll ---
    // Critical wound:
    //   - Devastating Wounds -> mortal wounds: no save, straight to the
    //     Feel No Pain against mortal wounds.
    //   - otherwise -> normal saving throw.
    graph.add_edge(
        wound_crit,
        if devastating { fnp_mortal } else { save },
        EdgeData {
            prob: p_crit,
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        wound_crit,
        wound,
        EdgeData {
            prob: fraction::Fraction::ONE - p_crit,
            effect: EdgeEffect::None,
        },
    );

    // Normal wound: success -> save; failure -> no wound.
    graph.add_edge(
        wound,
        save,
        EdgeData {
            prob: p_wound_cond.into(),
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        wound,
        no_wound,
        EdgeData {
            prob: fraction::Fraction::ONE - p_wound_cond,
            effect: EdgeEffect::None,
        },
    );

    // --- Phase 4: saving throw ---
    graph.add_edge(
        save,
        saved,
        EdgeData {
            prob: p_save.into(),
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        save,
        fnp_normal,
        EdgeData {
            prob: fraction::Fraction::ONE - p_save,
            effect: EdgeEffect::None,
        },
    );

    // --- Phase 5: Feel No Pain ---
    graph.add_edge(
        fnp_normal,
        ignored,
        EdgeData {
            prob: p_fnp,
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        fnp_normal,
        damage_roll,
        EdgeData {
            prob: fraction::Fraction::ONE - p_fnp,
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        fnp_mortal,
        ignored,
        EdgeData {
            prob: p_fnp,
            effect: EdgeEffect::None,
        },
    );
    graph.add_edge(
        fnp_mortal,
        mortal_roll,
        EdgeData {
            prob: fraction::Fraction::ONE - p_fnp,
            effect: EdgeEffect::None,
        },
    );

    AttackDag {
        graph,
        entry: hit_crit,
    }
}

/// Exact expected value from a node, by weighted traversal of the DAG
/// (memoized). The ExtraHits effect adds `count` re-entries of the wound
/// chain, evaluated on demand from the memo table.
pub fn expected_from(
    graph: &Dag,
    node: petgraph::graph::NodeIndex,
    memo: &mut std::collections::HashMap<petgraph::graph::NodeIndex, fraction::Fraction>,
) -> fraction::Fraction {
    if let Some(v) = memo.get(&node) {
        return *v;
    }
    let value = match &graph[node] {
        Phase::Terminal(outcome) => outcome.value_of(),
        _ => {
            let mut sum = fraction::Fraction::ZERO;
            for edge in graph.edges(node) {
                let data = edge.weight();
                let bonus = match &data.effect {
                    EdgeEffect::None => fraction::Fraction::ZERO,
                    // `count` extra hits walking the wound chain.
                    EdgeEffect::ExtraHits { count, chain } => {
                        fraction::Fraction::new(*count as u64, 1u64)
                            * expected_from(graph, *chain, memo)
                    }
                };
                sum += data.prob * (bonus + expected_from(graph, edge.target(), memo));
            }
            sum
        }
    };
    memo.insert(node, value);
    value
}

/// Pretty-prints every node with its outgoing edges.
pub fn print_dag(dag: &AttackDag) {
    println!("DAG (petgraph::Graph, acyclic):");
    for node in dag.graph.node_indices() {
        let phase = &dag.graph[node];
        println!("- [{}] {}", node.index(), phase);
        for edge in dag.graph.edges(node) {
            let effect = match &edge.weight().effect {
                EdgeEffect::None => String::new(),
                EdgeEffect::ExtraHits { count, chain } => {
                    format!(" (+{count} extra hits -> wound chain #{})", chain.index())
                }
            };
            println!(
                "    --p={:.4}--> [{}] {}{}",
                edge.weight().prob,
                edge.target().index(),
                dag.graph[edge.target()],
                effect
            );
        }
    }
}
