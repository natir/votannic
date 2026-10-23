//! Represent roll as tree

/* std use */

/* crate use */
use fraction::ToPrimitive as _;

/* project use */
use crate::profile;

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::marker::Copy,
    std::hash::Hash,
)]
#[enum_dispatch::enum_dispatch]
pub enum Operation {
    NumHit,
    Hit,
    Wound,
    Save,
    FNP,
    Success,
}

impl Operation {
    pub fn inner(&self) -> &dyn OperationTrait {
        match self {
            Operation::NumHit(n) => n,
            Operation::Hit(h) => h,
            Operation::Wound(w) => w,
            Operation::Save(s) => s,
            Operation::FNP(fnp) => fnp,
            Operation::Success(s) => s,
        }
    }
}

impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Operation::NumHit(n) => write!(f, "{:?}", n)?,
            Operation::Hit(h) => write!(f, "{:?}", h)?,
            Operation::Wound(w) => write!(f, "{:?}", w)?,
            Operation::Save(s) => write!(f, "{:?}", s)?,
            Operation::FNP(fnp) => write!(f, "{:?}", fnp)?,
            Operation::Success(s) => write!(f, "{:?}", s)?,
        }

        Ok(())
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Operation::NumHit(n) => write!(f, "{:?}", n)?,
            Operation::Hit(h) => write!(f, "{:?}", h)?,
            Operation::Wound(w) => write!(f, "{:?}", w)?,
            Operation::Save(s) => write!(f, "{:?}", s)?,
            Operation::FNP(fnp) => write!(f, "{:?}", fnp)?,
            Operation::Success(s) => write!(f, "{:?}", s)?,
        }

        Ok(())
    }
}

#[enum_dispatch::enum_dispatch(NodeOperationEnum)]
pub trait OperationTrait {
    fn success(&self) -> Vec<(Operation, u8)> {
        vec![]
    }

    fn failled(&self) -> u8 {
        0
    }

    fn critical_hit(&self) -> u8 {
        0
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct NumHit {
    /// Exact value
    #[builder(default = 0)]
    pub exact: u8,

    /// Number of sides of dices
    #[builder(default = 6)]
    pub sides: u8,

    /// Number of dices
    #[builder(default = 0)]
    pub number_of_dice: u8,

    /// Hit produce
    pub hit: Hit,
}

impl std::fmt::Debug for NumHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "NumHit {{ exact: {}, sides: {}, number_of_dice: {} }}",
            self.exact, self.sides, self.number_of_dice
        )
    }
}

impl std::fmt::Display for NumHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "NumHit({}, {}, {})",
            self.exact, self.sides, self.number_of_dice
        )
    }
}

impl OperationTrait for NumHit {
    fn success(&self) -> Vec<(Operation, u8)> {
        let mut success = vec![];

        if self.exact != 0 {
            let mut hit = self.hit;
            hit.n_hit = self.exact;

            success.push((Operation::Hit(hit), 0));
        }

        for _d in 0..self.number_of_dice {
            for value in 1..=self.sides {
                let mut hit = self.hit;
                hit.n_hit = value;

                success.push((Operation::Hit(hit), value));
            }
        }

        success
    }

    fn failled(&self) -> u8 {
        0
    }

    fn critical_hit(&self) -> u8 {
        0
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct Hit {
    /// Number of hit
    #[builder(default = 0)]
    n_hit: u8,

    /// Number of side of dice
    #[builder(default = 6)]
    sides: u8,

    /// Threshold to pass hit test
    #[builder(default = 6)]
    threshold: u8,

    /// Threshold to reach to get a critical hit
    #[builder(default = u8::MAX)]
    critical_threshold: u8,

    /// Effect of critical bonus
    #[builder(default = 0)]
    critical_bonus: u8,

    /// Wound produce
    pub wound: Wound,
}

impl std::fmt::Debug for Hit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Hit {{ n_hit: {}, sides: {}, threshold: {}, critical: {}+{} }}",
            self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus
        )
    }
}

impl OperationTrait for Hit {
    fn success(&self) -> Vec<(Operation, u8)> {
        let mut success = vec![];

        for _h in 0..self.n_hit {
            for value in 1..=self.sides {
                if value >= self.threshold {
                    let mut wound = self.wound;
                    wound.n_hit = 1;

                    success.push((Operation::Wound(wound), value));
                }
            }
        }

        success
    }

    fn failled(&self) -> u8 {
        self.n_hit * self.sides - self.success().len() as u8
    }

    fn critical_hit(&self) -> u8 {
        (0..self.n_hit)
            .into_iter()
            .map(|_| {
                (1..=self.sides)
                    .into_iter()
                    .filter(|v| *v >= self.critical_threshold)
                    .map(|_| self.critical_bonus)
                    .sum::<u8>()
            })
            .sum::<u8>()
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct Wound {
    /// Number of hit
    #[builder(default = 0)]
    n_hit: u8,

    /// Number of side of dice
    #[builder(default = 6)]
    sides: u8,

    #[builder(default = 6)]
    /// Success threshold
    threshold: u8,

    /// Critical threshold
    #[builder(default = u8::MAX)]
    critical_threshold: u8,

    /// Bonus add on critical
    #[builder(default = 0)]
    critical_bonus: u8,

    /// Save produce
    pub save: Save,
}

impl std::fmt::Debug for Wound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Wound {{ n_hit: {}, sides: {}, threshold: {}, critical: {}+{} }}",
            self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus
        )
    }
}

impl std::fmt::Display for Wound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Wound({}, {}, {}, {}+{})",
            self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus
        )
    }
}

impl OperationTrait for Wound {
    fn success(&self) -> Vec<(Operation, u8)> {
        let mut success = vec![];

        for _h in 0..self.n_hit {
            for value in 1..=self.sides {
                if value >= self.threshold {
                    let mut save = self.save;
                    save.n_wound = 1;
                    success.push((Operation::Save(save), value));
                }
            }
        }

        success
    }

    fn failled(&self) -> u8 {
        self.n_hit * self.sides - self.success().len() as u8
    }

    fn critical_hit(&self) -> u8 {
        (0..self.n_hit)
            .into_iter()
            .map(|_| {
                (1..=self.sides)
                    .into_iter()
                    .filter(|v| *v >= self.critical_threshold)
                    .map(|_| self.critical_bonus)
                    .sum::<u8>()
            })
            .sum::<u8>()
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct Save {
    /// Number of wound
    #[builder(default = 0)]
    n_wound: u8,

    /// Number of side of dice
    #[builder(default = 6)]
    sides: u8,

    #[builder(default = u8::MAX)]
    /// Save
    save: u8,

    /// Invulnerable save
    #[builder(default = u8::MAX)]
    invulnerable_save: u8,

    /// Armor penetration
    #[builder(default = 0)]
    armor_penetration: u8,

    /// Fnp produce
    pub fnp: FNP,
}

impl std::fmt::Debug for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Save {{ n_wound: {}, sides: {}, save: {}, invulnerable: {} ap: {} }}",
            self.n_wound, self.sides, self.save, self.invulnerable_save, self.armor_penetration
        )
    }
}

impl std::fmt::Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Save({}, {}, {}, {} {})",
            self.n_wound, self.sides, self.save, self.invulnerable_save, self.armor_penetration
        )
    }
}

impl OperationTrait for Save {
    fn success(&self) -> Vec<(Operation, u8)> {
        let mut success = vec![];

        let threshold = self
            .invulnerable_save
            .min(self.save + self.armor_penetration);

        for _w in 0..self.n_wound {
            for value in 1..=self.sides {
                if value < threshold {
                    success.push((Operation::FNP(self.fnp), value));
                }
            }
        }

        success
    }

    fn failled(&self) -> u8 {
        self.n_wound * self.sides - self.success().len() as u8
    }

    fn critical_hit(&self) -> u8 {
        0
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct FNP {
    /// Number of side of dice
    #[builder(default = 6)]
    sides: u8,

    #[builder(default = 7)]
    /// Success threshold
    threshold: u8,

    /// Damage
    #[builder(default = 0)]
    damage: u8,

    /// Success
    success: Success,
}

impl std::fmt::Display for FNP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "FNP({}, {}, {}, {})",
            self.sides, self.threshold, self.damage, self.success
        )
    }
}

impl OperationTrait for FNP {
    fn success(&self) -> Vec<(Operation, u8)> {
        let mut success = vec![];
        for value in 1..=self.damage {
            if value <= self.threshold {
                success.push((
                    Operation::Success(Success::builder().wound(1).build()),
                    value,
                ));
            }
        }

        success
    }

    fn failled(&self) -> u8 {
        self.damage - self.success().len() as u8
    }

    fn critical_hit(&self) -> u8 {
        0
    }

    fn exact_value(&self) -> u8 {
        0
    }
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    std::hash::Hash,
    std::marker::Copy,
    typed_builder::TypedBuilder,
)]
pub struct Success {
    #[builder(default = 0)]
    wound: u8,
}

impl std::fmt::Display for Success {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Success {{ wound: {} }}", self.wound,)
    }
}

impl OperationTrait for Success {}

#[derive(std::clone::Clone, std::default::Default, std::fmt::Debug)]
pub struct RollTree {
    pub graph: petgraph::graph::Graph<Operation, u8>,
    pub failled_node: petgraph::graph::NodeIndex,
    pub success_node: petgraph::graph::NodeIndex,
}

impl RollTree {
    pub fn from_weapon_body(weapon: profile::Weapon, body: profile::Body) -> RollTree {
        let success = Success::builder().wound(1).build();

        let fnp = FNP::builder()
            .success(success)
            .threshold(body.fnp as u8)
            .damage(1)
            .build();

        let save = Save::builder()
            .save(body.save as u8)
            .invulnerable_save(body.invuln as u8)
            .armor_penetration(weapon.ap as u8)
            .fnp(fnp)
            .n_wound(body.wounds as u8)
            .build();

        let wound = Wound::builder()
            .threshold(crate::utils::wound_threshold(weapon.skill, body.toughness) as u8)
            .save(save)
            .n_hit(1)
            .build();

        let hit = Hit::builder()
            .threshold(weapon.skill as u8)
            .wound(wound)
            .build();

        let num_hit = NumHit::builder()
            .exact(weapon.attack.expectation.to_f32().unwrap_or(0.0) as u8)
            .hit(hit)
            .build();

        let mut graph: petgraph::graph::Graph<Operation, u8> = petgraph::graph::Graph::new();
        let failled_node = graph.add_node(Operation::Success(
            Success::builder().wound(u8::MAX).build(),
        ));
        let success_node = graph.add_node(Operation::Success(Success::builder().wound(1).build()));

        let mut tree: RollTree = RollTree {
            graph,
            failled_node,
            success_node,
        };

        let mut not_insert_node = vec![(Operation::NumHit(num_hit), None)];
        tree.add_operations(&mut not_insert_node);

        tree
    }

    fn add_operations(
        &mut self,
        operations: &mut Vec<(Operation, Option<petgraph::graph::NodeIndex>)>,
    ) {
        if let Some((parent, node_index)) = operations.pop() {
            let parent_node = if let Some(ni) = node_index {
                ni
            } else {
                self.graph.add_node(parent)
            };

            match parent {
                Operation::FNP(f) => {
                    for (_child, weight) in f.success() {
                        self.graph
                            .update_edge(parent_node, self.success_node, weight);
                    }

                    self.add_operations(operations);
                }
                other => {
                    for (child, weight) in other.inner().success() {
                        let child_node = self.graph.add_node(child);

                        self.graph.update_edge(parent_node, child_node, weight);
                        self.graph.update_edge(
                            parent_node,
                            self.failled_node,
                            other.inner().failled(),
                        );

                        operations.push((child, Some(child_node)));
                    }

                    self.add_operations(operations)
                }
            }
        }
    }

    pub fn outcome(&self) -> (u8, u8) {
        (
            self.graph
                .edges_directed(self.success_node, petgraph::Direction::Incoming)
                .map(|e| e.weight())
                .sum(),
            self.graph
                .edges_directed(self.failled_node, petgraph::Direction::Incoming)
                .map(|e| e.weight())
                .sum(),
        )
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;
    use crate::roll;

    #[test]
    fn roll_tree() {
        let weapon = profile::Weapon::builder()
            .name("Choppa".to_string())
            .range(0)
            .attack(roll::Result::new(
                fraction::Fraction::new(1u32, 1u32),
                fraction::Fraction::new(1u32, 1u32),
            ))
            .skill(3)
            .strength(5)
            .ap(1)
            .damage(roll::Result::new(
                fraction::Fraction::new(1u32, 1u32),
                fraction::Fraction::new(1u32, 1u32),
            ))
            .build();

        let body = profile::Body::builder()
            .name("Boyz".to_string())
            .toughness(5)
            .save(5)
            .invuln(7)
            .fnp(7)
            .wounds(1)
            .build();

        let tree = RollTree::from_weapon_body(weapon, body);

        assert_eq!(tree.outcome(), (30, 21));

        let dot = petgraph::dot::Dot::new(&tree.graph);

        assert_eq!("digraph {\n    0 [ label = \"Success { wound: 255 }\" ]\n    1 [ label = \"Success { wound: 1 }\" ]\n    2 [ label = \"NumHit { exact: 1, sides: 6, number_of_dice: 0 }\" ]\n    3 [ label = \"Hit { n_hit: 1, sides: 6, threshold: 3, critical: 255+0 }\" ]\n    4 [ label = \"Wound { n_hit: 1, sides: 6, threshold: 5, critical: 255+0 }\" ]\n    5 [ label = \"Wound { n_hit: 1, sides: 6, threshold: 5, critical: 255+0 }\" ]\n    6 [ label = \"Wound { n_hit: 1, sides: 6, threshold: 5, critical: 255+0 }\" ]\n    7 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    8 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    9 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    10 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    11 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    12 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    13 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    14 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    15 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    16 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    17 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    18 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    19 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    20 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    21 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    22 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    23 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    24 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    25 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    26 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    27 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    28 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    29 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    30 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    31 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    32 [ label = \"Save { n_wound: 1, sides: 6, save: 5, invulnerable: 7 ap: 1 }\" ]\n    33 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    34 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    35 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    36 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    37 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    38 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    39 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    40 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    41 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    42 [ label = \"FNP { sides: 6, threshold: 7, damage: 1, success: Success { wound: 1 } }\" ]\n    2 -> 3 [ label = \"0\" ]\n    2 -> 0 [ label = \"0\" ]\n    3 -> 4 [ label = \"4\" ]\n    3 -> 0 [ label = \"3\" ]\n    3 -> 5 [ label = \"5\" ]\n    3 -> 6 [ label = \"6\" ]\n    6 -> 7 [ label = \"5\" ]\n    6 -> 0 [ label = \"4\" ]\n    6 -> 8 [ label = \"6\" ]\n    8 -> 9 [ label = \"1\" ]\n    8 -> 0 [ label = \"1\" ]\n    8 -> 10 [ label = \"2\" ]\n    8 -> 11 [ label = \"3\" ]\n    8 -> 12 [ label = \"4\" ]\n    8 -> 13 [ label = \"5\" ]\n    13 -> 1 [ label = \"1\" ]\n    12 -> 1 [ label = \"1\" ]\n    11 -> 1 [ label = \"1\" ]\n    10 -> 1 [ label = \"1\" ]\n    9 -> 1 [ label = \"1\" ]\n    7 -> 14 [ label = \"1\" ]\n    7 -> 0 [ label = \"1\" ]\n    7 -> 15 [ label = \"2\" ]\n    7 -> 16 [ label = \"3\" ]\n    7 -> 17 [ label = \"4\" ]\n    7 -> 18 [ label = \"5\" ]\n    18 -> 1 [ label = \"1\" ]\n    17 -> 1 [ label = \"1\" ]\n    16 -> 1 [ label = \"1\" ]\n    15 -> 1 [ label = \"1\" ]\n    14 -> 1 [ label = \"1\" ]\n    5 -> 19 [ label = \"5\" ]\n    5 -> 0 [ label = \"4\" ]\n    5 -> 20 [ label = \"6\" ]\n    20 -> 21 [ label = \"1\" ]\n    20 -> 0 [ label = \"1\" ]\n    20 -> 22 [ label = \"2\" ]\n    20 -> 23 [ label = \"3\" ]\n    20 -> 24 [ label = \"4\" ]\n    20 -> 25 [ label = \"5\" ]\n    25 -> 1 [ label = \"1\" ]\n    24 -> 1 [ label = \"1\" ]\n    23 -> 1 [ label = \"1\" ]\n    22 -> 1 [ label = \"1\" ]\n    21 -> 1 [ label = \"1\" ]\n    19 -> 26 [ label = \"1\" ]\n    19 -> 0 [ label = \"1\" ]\n    19 -> 27 [ label = \"2\" ]\n    19 -> 28 [ label = \"3\" ]\n    19 -> 29 [ label = \"4\" ]\n    19 -> 30 [ label = \"5\" ]\n    30 -> 1 [ label = \"1\" ]\n    29 -> 1 [ label = \"1\" ]\n    28 -> 1 [ label = \"1\" ]\n    27 -> 1 [ label = \"1\" ]\n    26 -> 1 [ label = \"1\" ]\n    4 -> 31 [ label = \"5\" ]\n    4 -> 0 [ label = \"4\" ]\n    4 -> 32 [ label = \"6\" ]\n    32 -> 33 [ label = \"1\" ]\n    32 -> 0 [ label = \"1\" ]\n    32 -> 34 [ label = \"2\" ]\n    32 -> 35 [ label = \"3\" ]\n    32 -> 36 [ label = \"4\" ]\n    32 -> 37 [ label = \"5\" ]\n    37 -> 1 [ label = \"1\" ]\n    36 -> 1 [ label = \"1\" ]\n    35 -> 1 [ label = \"1\" ]\n    34 -> 1 [ label = \"1\" ]\n    33 -> 1 [ label = \"1\" ]\n    31 -> 38 [ label = \"1\" ]\n    31 -> 0 [ label = \"1\" ]\n    31 -> 39 [ label = \"2\" ]\n    31 -> 40 [ label = \"3\" ]\n    31 -> 41 [ label = \"4\" ]\n    31 -> 42 [ label = \"5\" ]\n    42 -> 1 [ label = \"1\" ]\n    41 -> 1 [ label = \"1\" ]\n    40 -> 1 [ label = \"1\" ]\n    39 -> 1 [ label = \"1\" ]\n    38 -> 1 [ label = \"1\" ]\n}\n".to_string(), format!("{}", dot));
    }
}
