/// A dice expression: `flat + dice d faces`.
/// Examples: "2" -> fixed(2); "1d4+2" -> dice(1, 4, 2); "2d3+1" -> dice(2, 3, 1).
#[derive(Clone, Debug)]
pub struct DiceExpr {
    /// Number of dice rolled (0 = deterministic profile).
    pub dice: u32,
    /// Number of faces on each die (0 = deterministic profile).
    pub faces: u32,
    /// Flat modifier (may be negative).
    pub flat: i32,
}

impl DiceExpr {
    /// A deterministic value (no dice involved).
    pub const fn fixed(value: i32) -> Self {
        Self {
            dice: 0,
            faces: 0,
            flat: value,
        }
    }

    /// `count` dice with `faces` faces, plus a flat modifier.
    pub const fn dice(count: u32, faces: u32, flat: i32) -> Self {
        Self {
            dice: count,
            faces,
            flat,
        }
    }

    /// True when the expression always yields the same value.
    pub fn is_deterministic(&self) -> bool {
        self.dice == 0 || self.faces == 0
    }

    /// Human-readable form, e.g. "1d4+2", "3", "2d6-1".
    pub fn label(&self) -> String {
        let dice = if self.is_deterministic() {
            String::new()
        } else {
            format!("{}d{}", self.dice, self.faces)
        };
        match (dice.is_empty(), self.flat) {
            (true, f) => format!("{f}"),
            (false, 0) => dice,
            (false, f) => format!("{dice}{f:+}"),
        }
    }

    /// Full outcome distribution: one entry per possible total, with
    /// its probability (multi-dice expressions are convolved).
    pub fn distribution(&self) -> Vec<(i32, f64)> {
        if self.is_deterministic() {
            return vec![(self.flat, 1.0)];
        }
        let mut dist: Vec<(i32, f64)> = vec![(self.flat, 1.0)];
        let die_p = 1.0 / self.faces as f64;
        for _ in 0..self.dice {
            let mut next = std::collections::BTreeMap::new();
            for (value, p) in &dist {
                for face in 1..=self.faces {
                    *next.entry(value + face as i32).or_insert(0.0) += p * die_p;
                }
            }
            dist = next.into_iter().collect();
        }
        dist
    }

    /// Expected value of the expression.
    pub fn expected(&self) -> f64 {
        if self.is_deterministic() {
            return self.flat as f64;
        }
        self.distribution()
            .iter()
            .map(|(value, p)| *value as f64 * p)
            .sum()
    }
}
