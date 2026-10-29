//! Roll module - Résolution des lancers de dés pour Warhammer 40k
//!
//! Ce module implémente la résolution complète des attaques avec des lancers de dés.

use fraction::{Fraction, ConstOne, ConstZero, ToPrimitive};
use petgraph::graph::{Graph, NodeIndex};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

use crate::core::expectation::Expectation;
use crate::core::stats::Result;
use crate::core::theorical_dice::TheoricalDice;
use crate::models::{Body, Weapon};
use crate::utils;

/// Lancer de toucher.
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct HitRoll {
    /// Nombre de dés (attaques)
    pub number: Result,

    /// Seuil de réussite
    pub threshold: u64,

    /// Seuil critique
    #[builder(default = u64::MAX)]
    pub critical_threshold: u64,

    /// Bonus sur critique
    #[builder(default = Expectation(Fraction::new(0u8, 1u8)))]
    pub critical_bonus: Expectation,
}

impl HitRoll {
    #[hotpath::measure]
    pub fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = dice.upper_than(self.threshold);

        let bonus = Result::new(
            self.critical_bonus.0 * dice.upper_than(self.critical_threshold).expectation,
            if self.critical_threshold != u64::MAX {
                dice.variance()
            } else {
                Fraction::new(1u64, 1u64)
            },
        );

        result * self.number + bonus
    }
}

/// Lancer de blessure.
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct WoundRoll {
    /// Lancer de toucher
    pub hit_roll: HitRoll,

    /// Seuil de réussite
    pub threshold: u64,

    /// Seuil critique
    #[builder(default = u64::MAX)]
    pub critical_threshold: u64,

    /// Bonus sur critique
    #[builder(default = Expectation(Fraction::new(0u8, 1u8)))]
    pub critical_bonus: Expectation,
}

impl WoundRoll {
    #[hotpath::measure]
    pub fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let base = dice.upper_than(self.threshold);
        let bonus = Result::new(
            self.critical_bonus.0 * dice.upper_than(self.critical_threshold).expectation,
            if self.critical_threshold == u64::MAX {
                dice.variance()
            } else {
                Fraction::new(1u64, 1u64)
            },
        );

        Result {
            expectation: base.expectation + bonus.expectation,
            variance: base.variance + bonus.variance,
        }
    }
}

/// Lancer de sauvegarde.
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct SaveRoll {
    /// Lancer de blessure
    pub wound_roll: WoundRoll,

    /// Seuil de réussite
    pub threshold: u64,
}

impl SaveRoll {
    #[hotpath::measure]
    pub fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = self.wound_roll.roll();

        Result {
            expectation: result.expectation * dice.lower_than(self.threshold).expectation,
            variance: result.expectation * dice.variance(),
        }
    }
}

/// Lancer de FNP.
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct FNPRoll {
    /// Lancer de sauvegarde
    pub save_roll: SaveRoll,

    /// Seuil de réussite
    pub threshold: u64,

    /// Dégâts de l'arme
    pub damage: Result,
}

impl FNPRoll {
    #[hotpath::measure]
    pub fn roll(&self) -> Result {
        let dice = TheoricalDice::<6>::new();

        let result = self.save_roll.roll();
        let damage = result * self.damage;
        let fnp_result = dice.upper_than(self.threshold) * damage;

        Result {
            expectation: damage.expectation - fnp_result.expectation,
            variance: damage.variance + fnp_result.variance,
        }
    }
}

/// Lancer complet.
#[derive(
    std::marker::Copy,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    typed_builder::TypedBuilder,
)]
pub struct FullRoll {
    // Défense
    /// Nombre d'armes
    pub weapons_number: u64,

    // Blessure
    /// Seuil d'endurance
    pub toughness: u64,

    // Sauvegarde
    /// Sauvegarde
    #[builder(default = u64::MAX)]
    pub save: u64,
    /// Sauvegarde invulnérable
    #[builder(default = u64::MAX)]
    pub invulnerable_save: u64,

    // FNP
    /// Seuil FNP
    #[builder(default = u64::MAX)]
    pub fnp_threshold: u64,

    // Attaque
    /// Attaques par arme
    pub attack: Result,

    // Toucher
    /// Compétence
    pub skill: u64,
    /// Seuil critique toucher
    #[builder(default = u64::MAX)]
    pub hit_critical_threshold: u64,
    /// Bonus critique toucher
    #[builder(default = Expectation(Fraction::new(0u64, 1u64)))]
    pub hit_critical_bonus: Expectation,

    // Blessure
    /// Force
    pub strength: u64,
    /// Seuil critique blessure
    #[builder(default = u64::MAX)]
    pub wound_critical_threshold: u64,
    /// Bonus critique blessure
    #[builder(default = Result::new(Fraction::new(0u64, 1u64), Fraction::new(1u64, 1u64)))]
    pub wound_critical_bonus: Result,

    // Sauvegarde
    /// Perforation
    #[builder(default = 0)]
    pub ap: u64,

    // Dégâts
    #[builder(default = Result::new(Fraction::new(1u64, 1u64), Fraction::new(1u64, 1u64)))]
    pub damage: Result,
}

impl FullRoll {
    #[hotpath::measure]
    pub fn build_hitroll(&self) -> HitRoll {
        log::info!(
            "Hitroll parameter: number: {}, attack: {}, threshold: {}",
            self.weapons_number,
            self.attack,
            self.skill
        );

        HitRoll::builder()
            .number(self.attack * self.weapons_number)
            .threshold(self.skill)
            .build()
    }

    #[hotpath::measure]
    pub fn build_woundroll(&self, hitroll: HitRoll) -> WoundRoll {
        log::info!(
            "Woundroll parameter: hitroll: {}, strength: {}, toughness: {}, threshold: {}",
            hitroll.roll(),
            self.strength,
            self.toughness,
            utils::wound_threshold(self.strength, self.toughness)
        );
        WoundRoll::builder()
            .hit_roll(hitroll)
            .threshold(utils::wound_threshold(self.strength, self.toughness))
            .build()
    }

    #[hotpath::measure]
    pub fn build_saveroll(&self, woundroll: WoundRoll) -> SaveRoll {
        log::info!(
            "Saveroll parameter: woundroll: {}, save: {}, inv: {}, ap: {}",
            woundroll.roll(),
            self.save,
            self.invulnerable_save,
            self.ap,
        );
        SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(self.invulnerable_save.max(self.save - self.ap))
            .build()
    }

    #[hotpath::measure]
    pub fn build_fnproll(&self, saveroll: SaveRoll) -> FNPRoll {
        log::info!(
            "FNProll parameter: saveroll: {}, damage: {:?}, threshold: {}",
            saveroll.roll(),
            self.damage,
            self.fnp_threshold
        );
        FNPRoll::builder()
            .save_roll(saveroll)
            .threshold(self.fnp_threshold)
            .damage(self.damage)
            .build()
    }
}

impl FullRoll {
    #[hotpath::measure]
    pub fn roll(&self) -> Result {
        log::info!("Start a full roll");

        let hitroll = self.build_hitroll();
        log::info!("Hitroll result: {}", hitroll.roll());

        let woundroll = self.build_woundroll(hitroll);
        log::info!("Woundroll result: {}", woundroll.roll());

        let saveroll = self.build_saveroll(woundroll);
        log::info!("Saveroll result: {}", saveroll.roll());

        let fnproll = self.build_fnproll(saveroll);
        log::info!("FNProll result: {}", fnproll.roll());

        log::info!("End full roll: {}", fnproll.roll());
        fnproll.roll()
    }
}

/// Représentation DAG pour les lancers (version moderne).
pub mod dag {
    use super::*;
    use petgraph::visit::EdgeRef;
    use std::collections::HashMap;

    /// Phase de la résolution d'attaque.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum Phase {
        AttacksRoll,
        HitCritical,
        Hit { threshold: u64 },
        WoundCritical,
        Wound { threshold: u64 },
        Save { threshold: u64 },
        FeelNoPainNormal { threshold: u64 },
        FeelNoPainMortal { threshold: u64 },
        DamageRoll,
        MortalWoundsRoll,
        TerminalMiss,
        TerminalNoWound,
        TerminalSaved,
        TerminalIgnoredByPain,
        TerminalDamage(i32),
        TerminalMortalWounds(i32),
    }

    impl std::fmt::Display for Phase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Phase::AttacksRoll => write!(f, "Attacks roll"),
                Phase::HitCritical => write!(f, "To-hit roll: critical (unmodified 6)?"),
                Phase::Hit { threshold } => write!(f, "To-hit roll (threshold: {})", threshold),
                Phase::WoundCritical => write!(f, "To-wound roll: critical (unmodified 6)?"),
                Phase::Wound { threshold } => write!(f, "To-wound roll (threshold: {})", threshold),
                Phase::Save { threshold } => write!(f, "Saving throw (threshold: {})", threshold),
                Phase::FeelNoPainNormal { threshold } => {
                    write!(f, "Feel No Pain, normal damage (threshold: {})", threshold)
                }
                Phase::FeelNoPainMortal { threshold } => {
                    write!(f, "Feel No Pain, mortal wounds (threshold: {})", threshold)
                }
                Phase::DamageRoll => write!(f, "Damage roll"),
                Phase::MortalWoundsRoll => write!(f, "Mortal wounds roll"),
                Phase::TerminalMiss => write!(f, "Terminal: Miss"),
                Phase::TerminalNoWound => write!(f, "Terminal: NoWound"),
                Phase::TerminalSaved => write!(f, "Terminal: Saved"),
                Phase::TerminalIgnoredByPain => write!(f, "Terminal: IgnoredByPain"),
                Phase::TerminalDamage(d) => write!(f, "Terminal: Damage({})", d),
                Phase::TerminalMortalWounds(mw) => write!(f, "Terminal: MortalWounds({})", mw),
            }
        }
    }

    /// Effet porté par une arête.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum EdgeEffect {
        None,
        ExtraHits { count: u32, chain: NodeIndex },
    }

    /// Données d'une arête.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EdgeData {
        pub prob: Fraction,
        pub effect: EdgeEffect,
    }

    /// Type du graphe.
    pub type RollGraph = Graph<Phase, EdgeData>;

    /// DAG avec point d'entrée.
    #[derive(Clone, Debug)]
    pub struct RollDag {
        pub graph: RollGraph,
        pub entry: NodeIndex,
    }

    impl RollDag {
        pub fn from_weapon_body(weapon: &Weapon, body: &Body) -> Self {
            let mut graph = RollGraph::new();
            
            let hit_threshold = weapon.skill;
            let wound_threshold = utils::wound_threshold(weapon.strength, body.toughness);
            let save_threshold = body.save.max(body.invuln).saturating_sub(weapon.ap);
            let fnp_threshold = body.fnp;
            
            let hit_crit = graph.add_node(Phase::HitCritical);
            let hit = graph.add_node(Phase::Hit { threshold: hit_threshold });
            let wound_crit = graph.add_node(Phase::WoundCritical);
            let wound = graph.add_node(Phase::Wound { threshold: wound_threshold });
            let save = graph.add_node(Phase::Save { threshold: save_threshold });
            let fnp_normal = graph.add_node(Phase::FeelNoPainNormal { threshold: fnp_threshold });
            let fnp_mortal = graph.add_node(Phase::FeelNoPainMortal { threshold: fnp_threshold });
            
            let miss = graph.add_node(Phase::TerminalMiss);
            let no_wound = graph.add_node(Phase::TerminalNoWound);
            let saved = graph.add_node(Phase::TerminalSaved);
            let ignored = graph.add_node(Phase::TerminalIgnoredByPain);
            
            let damage_value = weapon.damage.expectation.to_i64().unwrap_or(0) as i32;
            let damage = graph.add_node(Phase::TerminalDamage(damage_value));
            let mortal = graph.add_node(Phase::TerminalMortalWounds(damage_value));
            
            let p_crit = Fraction::new(1u64, 6u64);
            let p_hit = Fraction::new((6u64).saturating_sub(hit_threshold) + 1, 6u64);
            let p_wound = Fraction::new((6u64).saturating_sub(wound_threshold) + 1, 6u64);
            let p_save = Fraction::new((7u64).saturating_sub(save_threshold), 6u64);
            let p_fnp = Fraction::new((7u64).saturating_sub(fnp_threshold), 6u64);
            
            // Hit phase
            graph.add_edge(hit_crit, wound_crit, EdgeData { prob: p_crit, effect: EdgeEffect::None });
            graph.add_edge(hit_crit, hit, EdgeData { prob: Fraction::ONE - p_crit, effect: EdgeEffect::None });
            graph.add_edge(hit, wound_crit, EdgeData { prob: p_hit, effect: EdgeEffect::None });
            graph.add_edge(hit, miss, EdgeData { prob: Fraction::ONE - p_hit, effect: EdgeEffect::None });
            
            // Wound phase
            graph.add_edge(wound_crit, save, EdgeData { prob: p_crit, effect: EdgeEffect::None });
            graph.add_edge(wound_crit, wound, EdgeData { prob: Fraction::ONE - p_crit, effect: EdgeEffect::None });
            graph.add_edge(wound, save, EdgeData { prob: p_wound, effect: EdgeEffect::None });
            graph.add_edge(wound, no_wound, EdgeData { prob: Fraction::ONE - p_wound, effect: EdgeEffect::None });
            
            // Save phase
            graph.add_edge(save, saved, EdgeData { prob: p_save, effect: EdgeEffect::None });
            graph.add_edge(save, fnp_normal, EdgeData { prob: Fraction::ONE - p_save, effect: EdgeEffect::None });
            
            // FNP phase
            graph.add_edge(fnp_normal, ignored, EdgeData { prob: p_fnp, effect: EdgeEffect::None });
            graph.add_edge(fnp_normal, damage, EdgeData { prob: Fraction::ONE - p_fnp, effect: EdgeEffect::None });
            graph.add_edge(fnp_mortal, ignored, EdgeData { prob: p_fnp, effect: EdgeEffect::None });
            graph.add_edge(fnp_mortal, mortal, EdgeData { prob: Fraction::ONE - p_fnp, effect: EdgeEffect::None });
            
            Self { graph, entry: hit_crit }
        }
        
        pub fn expected_from(
            &self,
            node: NodeIndex,
            memo: &mut HashMap<NodeIndex, Fraction>,
        ) -> Fraction {
            if let Some(v) = memo.get(&node) {
                return *v;
            }
            
            let value = match &self.graph[node] {
                Phase::TerminalDamage(d) | Phase::TerminalMortalWounds(d) => Fraction::from(*d as u64),
                Phase::TerminalMiss | Phase::TerminalNoWound | Phase::TerminalSaved | Phase::TerminalIgnoredByPain => Fraction::ZERO,
                _ => {
                    let mut sum = Fraction::ZERO;
                    for edge in self.graph.edges(node) {
                        let data = edge.weight();
                        let bonus = match &data.effect {
                            EdgeEffect::None => Fraction::ZERO,
                            EdgeEffect::ExtraHits { count, chain } => Fraction::from(*count as u64) * self.expected_from(*chain, memo),
                        };
                        sum += data.prob * (bonus + self.expected_from(edge.target(), memo));
                    }
                    sum
                }
            };
            
            memo.insert(node, value);
            value
        }
        
        pub fn outcome(&self) -> (u8, u8) {
            let mut success = 0u8;
            let mut failure = 0u8;
            
            for node in self.graph.node_indices() {
                match &self.graph[node] {
                    Phase::TerminalDamage(d) | Phase::TerminalMortalWounds(d) => success += *d as u8,
                    Phase::TerminalMiss | Phase::TerminalNoWound | Phase::TerminalSaved | Phase::TerminalIgnoredByPain => failure += 1,
                    _ => {}
                }
            }
            
            (success, failure)
        }
    }
}

/// Arbre de lancers pour compatibilité ascendante.
pub mod tree {
    use super::*;
    
    use petgraph::graph::Graph as PetGraph;
    use std::fmt::Display;
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash)]
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
    
    impl Debug for Operation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Operation::NumHit(n) => write!(f, "{:?}", n),
                Operation::Hit(h) => write!(f, "{:?}", h),
                Operation::Wound(w) => write!(f, "{:?}", w),
                Operation::Save(s) => write!(f, "{:?}", s),
                Operation::FNP(fnp) => write!(f, "{:?}", fnp),
                Operation::Success(s) => write!(f, "{:?}", s),
            }
        }
    }
    
    impl Display for Operation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Operation::NumHit(n) => write!(f, "{:?}", n),
                Operation::Hit(h) => write!(f, "{:?}", h),
                Operation::Wound(w) => write!(f, "{:?}", w),
                Operation::Save(s) => write!(f, "{:?}", s),
                Operation::FNP(fnp) => write!(f, "{:?}", fnp),
                Operation::Success(s) => write!(f, "{:?}", s),
            }
        }
    }
    
    #[enum_dispatch::enum_dispatch(NodeOperationEnum)]
    pub trait OperationTrait {
        fn success(&self) -> Vec<(Operation, u8)> { vec![] }
        fn failled(&self) -> u8 { 0 }
        fn critical_hit(&self) -> u8 { 0 }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Hash, TypedBuilder)]
    pub struct NumHit {
        #[builder(default = 0)]
        pub exact: u8,
        #[builder(default = 6)]
        pub sides: u8,
        #[builder(default = 0)]
        pub number_of_dice: u8,
        pub hit: Hit,
    }
    
    impl Debug for NumHit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "NumHit {{ exact: {}, sides: {}, number_of_dice: {} }}", self.exact, self.sides, self.number_of_dice)
        }
    }
    
    impl Display for NumHit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "NumHit({}, {}, {})", self.exact, self.sides, self.number_of_dice)
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
        fn failled(&self) -> u8 { 0 }
        fn critical_hit(&self) -> u8 { 0 }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Hash, TypedBuilder)]
    pub struct Hit {
        #[builder(default = 0)]
        pub n_hit: u8,
        #[builder(default = 6)]
        pub sides: u8,
        #[builder(default = 6)]
        pub threshold: u8,
        #[builder(default = u8::MAX)]
        pub critical_threshold: u8,
        #[builder(default = 0)]
        pub critical_bonus: u8,
        pub wound: Wound,
    }
    
    impl Debug for Hit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Hit {{ n_hit: {}, sides: {}, threshold: {}, critical: {}+{} }}", self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus)
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
        fn failled(&self) -> u8 { self.n_hit * self.sides - self.success().len() as u8 }
        fn critical_hit(&self) -> u8 {
            (0..self.n_hit).into_iter().map(|_| (1..=self.sides).into_iter().filter(|v| *v >= self.critical_threshold).map(|_| self.critical_bonus).sum::<u8>()).sum::<u8>()
        }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Hash, TypedBuilder)]
    pub struct Wound {
        #[builder(default = 0)]
        pub n_hit: u8,
        #[builder(default = 6)]
        pub sides: u8,
        #[builder(default = 6)]
        pub threshold: u8,
        #[builder(default = u8::MAX)]
        pub critical_threshold: u8,
        #[builder(default = 0)]
        pub critical_bonus: u8,
        pub save: Save,
    }
    
    impl Debug for Wound {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Wound {{ n_hit: {}, sides: {}, threshold: {}, critical: {}+{} }}", self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus)
        }
    }
    
    impl Display for Wound {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Wound({}, {}, {}, {}+{})", self.n_hit, self.sides, self.threshold, self.critical_threshold, self.critical_bonus)
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
        fn failled(&self) -> u8 { self.n_hit * self.sides - self.success().len() as u8 }
        fn critical_hit(&self) -> u8 {
            (0..self.n_hit).into_iter().map(|_| (1..=self.sides).into_iter().filter(|v| *v >= self.critical_threshold).map(|_| self.critical_bonus).sum::<u8>()).sum::<u8>()
        }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Hash, TypedBuilder)]
    pub struct Save {
        #[builder(default = 0)]
        pub n_wound: u8,
        #[builder(default = 6)]
        pub sides: u8,
        #[builder(default = u8::MAX)]
        pub save: u8,
        #[builder(default = u8::MAX)]
        pub invulnerable_save: u8,
        #[builder(default = 0)]
        pub armor_penetration: u8,
        pub fnp: FNP,
    }
    
    impl Debug for Save {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Save {{ n_wound: {}, sides: {}, save: {}, invulnerable: {} ap: {} }}", self.n_wound, self.sides, self.save, self.invulnerable_save, self.armor_penetration)
        }
    }
    
    impl Display for Save {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Save({}, {}, {}, {} {})", self.n_wound, self.sides, self.save, self.invulnerable_save, self.armor_penetration)
        }
    }
    
    impl OperationTrait for Save {
        fn success(&self) -> Vec<(Operation, u8)> {
            let mut success = vec![];
            let threshold = self.invulnerable_save.min(self.save + self.armor_penetration);
            for _w in 0..self.n_wound {
                for value in 1..=self.sides {
                    if value < threshold {
                        success.push((Operation::FNP(self.fnp), value));
                    }
                }
            }
            success
        }
        fn failled(&self) -> u8 { self.n_wound * self.sides - self.success().len() as u8 }
        fn critical_hit(&self) -> u8 { 0 }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Debug, Hash, TypedBuilder)]
    pub struct FNP {
        #[builder(default = 6)]
        pub sides: u8,
        #[builder(default = 7)]
        pub threshold: u8,
        #[builder(default = 0)]
        pub damage: u8,
        pub success: Success,
    }
    
    impl Display for FNP {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FNP({}, {}, {}, {})", self.sides, self.threshold, self.damage, self.success)
        }
    }
    
    impl OperationTrait for FNP {
        fn success(&self) -> Vec<(Operation, u8)> {
            let mut success = vec![];
            for value in 1..=self.damage {
                if value <= self.threshold {
                    success.push((Operation::Success(Success::builder().wound(1).build()), value));
                }
            }
            success
        }
        fn failled(&self) -> u8 { self.damage - self.success().len() as u8 }
        fn critical_hit(&self) -> u8 { 0 }
        fn exact_value(&self) -> u8 { 0 }
    }
    
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Default, Debug, Hash, TypedBuilder)]
    pub struct Success {
        #[builder(default = 0)]
        pub wound: u8,
    }
    
    impl Display for Success {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Success {{ wound: {} }}", self.wound)
        }
    }
    
    impl OperationTrait for Success {}
    
    #[derive(Clone, Default, Debug)]
    pub struct RollTree {
        pub graph: PetGraph<Operation, u8>,
        pub failled_node: petgraph::graph::NodeIndex,
        pub success_node: petgraph::graph::NodeIndex,
    }
    
    impl RollTree {
        pub fn from_weapon_body(weapon: crate::models::Weapon, body: crate::models::Body) -> RollTree {
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
                .threshold(utils::wound_threshold(weapon.skill, body.toughness) as u8)
                .save(save)
                .n_hit(1)
                .build();
            let hit = Hit::builder()
                .threshold(weapon.skill as u8)
                .wound(wound)
                .build();
            let attack_value = weapon.attack.expectation.to_i64().unwrap_or(0) as u8;
            let num_hit = NumHit::builder()
                .exact(attack_value)
                .hit(hit)
                .build();

            let mut graph: PetGraph<Operation, u8> = PetGraph::new();
            let failled_node = graph.add_node(Operation::Success(Success::builder().wound(u8::MAX).build()));
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
                            self.graph.update_edge(parent_node, self.success_node, weight);
                        }
                        self.add_operations(operations);
                    }
                    other => {
                        for (child, weight) in other.inner().success() {
                            let child_node = self.graph.add_node(child);
                            self.graph.update_edge(parent_node, child_node, weight);
                            self.graph.update_edge(parent_node, self.failled_node, other.inner().failled());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_roll() {
        let roll = HitRoll::builder().number(6.into()).threshold(4).build();
        assert_eq!(
            roll.roll(),
            Result::new(
                Fraction::new(3u64, 1u64),
                Fraction::new(47u64, 12u64)
            )
        );

        let roll = HitRoll::builder()
            .number(6.into())
            .threshold(4)
            .critical_threshold(6)
            .critical_bonus(1u64.into())
            .build();

        assert_eq!(
            roll.roll(),
            Result::new(
                Fraction::new(19u64, 6u64),
                Fraction::new(35u64, 6u64)
            )
        );
    }

    #[test]
    fn wound_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();

        assert_eq!(
            woundroll.roll(),
            Result::new(
                Fraction::new(1u8, 2u8),
                Fraction::new(35u64, 6u64)
            )
        );

        let woundroll = WoundRoll::builder()
            .hit_roll(hitroll)
            .threshold(4)
            .critical_threshold(6)
            .critical_bonus(1u64.into())
            .build();
        assert_eq!(
            woundroll.roll(),
            Result::new(
                Fraction::new(2u8, 3u8),
                Fraction::new(47u64, 12u64)
            )
        );
    }

    #[test]
    fn save_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();
        let saveroll = SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(4)
            .build();

        assert_eq!(
            saveroll.roll(),
            Result::new(
                Fraction::new(1u8, 4u8),
                Fraction::new(35u64, 24u64)
            )
        );
    }

    #[test]
    fn fnp_roll() {
        let hitroll = HitRoll::builder().number(6.into()).threshold(4).build();
        let woundroll = WoundRoll::builder().hit_roll(hitroll).threshold(4).build();
        let saveroll = SaveRoll::builder()
            .wound_roll(woundroll)
            .threshold(4)
            .build();
        let fnproll = FNPRoll::builder()
            .save_roll(saveroll)
            .threshold(6)
            .damage(1u64.into())
            .build();

        assert_eq!(
            fnproll.roll(),
            Result::new(
                Fraction::new(5u8, 24u8),
                Fraction::new(1645u64, 288u64)
            )
        );
    }
}
