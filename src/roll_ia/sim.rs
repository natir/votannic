use fraction::ConstZero as _;
use petgraph::visit::EdgeRef as _;

use super::dag;
use super::profiles;

/// Tiny deterministic LCG so the crate needs no `rand` dependency.
struct Lcg(u64);

impl Lcg {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as f64 / (1u64 << 31) as f64
    }
}

/// Random walk through the DAG. Each node has one child per possible
/// outcome: draw a uniform point and walk the edges, accumulating
/// probabilities until the point falls below the running total.
fn simulate_walk(
    graph: &dag::Dag,
    node: petgraph::graph::NodeIndex,
    rng: &mut Lcg,
) -> fraction::Fraction {
    match &graph[node] {
        dag::Phase::Terminal(outcome) => outcome.value_of(),
        _ => {
            let r = fraction::Fraction::from(rng.next_f64());
            let mut acc = fraction::Fraction::ZERO;
            let mut chosen = None;
            for edge in graph.edges(node) {
                acc += edge.weight().prob;
                if r < acc {
                    chosen = Some(edge);
                    break;
                }
            }
            let chosen = chosen.expect("edge probabilities must sum to 1");
            let mut damage = match &chosen.weight().effect {
                dag::EdgeEffect::None => fraction::Fraction::ZERO,
                dag::EdgeEffect::ExtraHits { count, chain } => {
                    let (n, chain) = (*count, *chain);
                    (0..n).map(|_| simulate_walk(graph, chain, rng)).sum()
                }
            };
            damage += simulate_walk(graph, chosen.target(), rng);
            damage
        }
    }
}

/// Full simulation: the attack-chain DAG plus the (optional) DAG of the
/// attacks roll.
pub struct AttackSimulation {
    /// Resolution of ONE attack (hit -> wound -> save -> FNP -> damage).
    pub attack: dag::AttackDag,
    /// Attacks roll, None when the profile is deterministic.
    pub attacks_roll: Option<dag::AttackDag>,
}

pub fn build_attack_simulation(
    weapon: &profiles::Weapon,
    target: &profiles::UnitProfile,
) -> AttackSimulation {
    let attack = dag::build_attack_dag(weapon, target);
    let mut graph = dag::Dag::new();
    let attacks_roll = if weapon.attacks.is_deterministic() {
        None
    } else {
        let entry = dag::add_roll_node(&mut graph, &weapon.attacks, dag::RollKind::Attacks);
        Some(dag::AttackDag { graph, entry })
    };
    AttackSimulation {
        attack,
        attacks_roll,
    }
}

/// Expected number of attacks per volley, from the attacks-roll DAG.
pub fn expected_attacks(
    simulation: &AttackSimulation,
    weapon: &profiles::Weapon,
) -> fraction::Fraction {
    match &simulation.attacks_roll {
        Some(roll) => {
            let mut memo = std::collections::HashMap::new();
            dag::expected_from(&roll.graph, roll.entry, &mut memo)
        }
        None => weapon.attacks.flat.into(),
    }
}

/// Expected damage (wounds removed) for the full volley.
///
/// The number of attacks N is independent of the fate of each attack:
///     E[damage] = E[N] * E[damage | one attack].
pub fn expected_damage(
    simulation: &AttackSimulation,
    weapon: &profiles::Weapon,
) -> fraction::Fraction {
    let mut memo = std::collections::HashMap::new();
    let per_attack =
        dag::expected_from(&simulation.attack.graph, simulation.attack.entry, &mut memo);
    expected_attacks(simulation, weapon).max(fraction::Fraction::ZERO) * per_attack
}

/// Average damage observed by simulating `iterations` full volleys.
/// The attacks roll (when present) is sampled by a random walk through
/// its DAG; one walk covers the whole expression, multi-dice totals
/// included.
pub fn simulate_damage(
    simulation: &AttackSimulation,
    weapon: &profiles::Weapon,
    iterations: u32,
    seed: u64,
) -> fraction::Fraction {
    let mut rng = Lcg(seed);
    let mut total = fraction::Fraction::ZERO;
    for _ in 0..iterations {
        let n: usize = match &simulation.attacks_roll {
            Some(roll) => {
                simulate_walk(&roll.graph, roll.entry, &mut rng).max(fraction::Fraction::ZERO)
            }
            None => weapon.attacks.flat.max(0).into(),
        }
        .floor()
        .try_into()
        .unwrap();
        for _ in 0..n {
            total += simulate_walk(&simulation.attack.graph, simulation.attack.entry, &mut rng);
        }
    }
    total / iterations
}
