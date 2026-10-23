//! A tools to generate all possible outcomes in Warhammer 40k system
//!
//! Attack sequence modelise (Core Rules v11):
//! Hit -> Woods -> Save -> Dégats -> [FNP]

/* std use */

/* crate use */

/* module declaration */
pub mod cli;
pub mod error;
pub mod newrecruit;
pub mod profile;
pub mod roll;
pub mod roll_ia;
pub mod roll_tree;
pub mod utils;

/* project use */

#[hotpath::measure]
pub fn alpha_vs_beta(
    alpha: &profile::Army,
    beta: &profile::Army,
) -> Vec<(String, String, String, (u8, u8))> {
    let mut results = Vec::new();

    for (aname, aunit) in alpha.units.iter() {
        for bname in beta.units.keys() {
            log::info!("alpha unit {} against beta unit {}", aname, bname);
            for (_n, weapon) in aunit.weapons() {
                for (model, _n) in aunit.models.iter() {
                    let tree =
                        roll_tree::RollTree::from_weapon_body(weapon.clone(), model.body().clone());

                    log::info!(
                        "outcome of {:?} to {:?} => {:?}",
                        weapon,
                        model.body(),
                        tree.outcome()
                    );

                    if aname == "Hernkyn Pioneers"
                        && weapon.name == "Plasma knife"
                        && bname == "Kâhl"
                    {
                        let dot = petgraph::dot::Dot::new(&tree.graph);
                        println!("\n{}\n", dot);
                    }

                    results.push((
                        aname.to_string(),
                        weapon.name.to_string(),
                        bname.to_string(),
                        tree.outcome(),
                    ));
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    #[test]
    fn roll_tree_alpha_beta() -> error::Result<()> {
        let nr_alpha: newrecruit::NewRecruit = serde_json::from_reader(
            std::fs::File::open("tests/data/alpha.json").map(std::io::BufReader::new)?,
        )?;
        let army_alpha = profile::Army::try_from(&nr_alpha)?;

        let nr_beta: newrecruit::NewRecruit = serde_json::from_reader(
            std::fs::File::open("tests/data/beta.json").map(std::io::BufReader::new)?,
        )?;
        let army_beta = profile::Army::try_from(&nr_beta)?;

        assert_eq!(
            alpha_vs_beta(&army_alpha, &army_beta),
            vec![
                (
                    "Blade Champion".to_string(),
                    "➤ Vaultswords - Behemor".to_string(),
                    "Bigboss".to_string(),
                    (72, 204)
                ),
                (
                    "Blade Champion".to_string(),
                    "➤ Vaultswords - Behemor".to_string(),
                    "Boyz".to_string(),
                    (72, 204)
                ),
                (
                    "Custodian Guard".to_string(),
                    "Guardian Spear".to_string(),
                    "Bigboss".to_string(),
                    (60, 170)
                ),
                (
                    "Custodian Guard".to_string(),
                    "Guardian Spear".to_string(),
                    "Boyz".to_string(),
                    (60, 170)
                )
            ]
        );

        Ok(())
    }
}
