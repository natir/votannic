//! A tools to generate all possible outcomes in Warhammer 40k system

#![warn(missing_docs)]

/* std use */

/* crate use */
use anyhow::Context as _;
use clap::Parser as _;

/* project use */
use votannic::cli;
use votannic::error;
use votannic::newrecruit;
use votannic::profile;

#[hotpath::main]
fn main() -> error::Result<()> {
    // Parse argument
    let arguments = cli::Arguments::parse();

    // Setup logger
    stderrlog::new()
        .module(module_path!())
        .quiet(arguments.quiet())
        .verbosity(arguments.verbosity())
        .timestamp(arguments.timestamp())
        .init()
        .context("stderrlog already create a logger")?;

    let nr_alpha: newrecruit::NewRecruit = serde_json::from_reader(
        std::fs::File::open(arguments.alpha()).map(std::io::BufReader::new)?,
    )?;
    let army_alpha = profile::Army::try_from(&nr_alpha)?;

    let nr_beta: newrecruit::NewRecruit = serde_json::from_reader(
        std::fs::File::open(arguments.beta()).map(std::io::BufReader::new)?,
    )?;
    let army_beta = profile::Army::try_from(&nr_beta)?;

    let results = votannic::alpha_vs_beta(&army_alpha, &army_beta);

    for (aname, weapon_name, bname, result) in results {
        println!("{}: {} -> {} => {:?}", aname, weapon_name, bname, result);
    }

    let results = votannic::alpha_vs_beta(&army_beta, &army_alpha);

    for (aname, weapon_name, bname, result) in results {
        println!("{}: {} -> {} => {:?}", aname, weapon_name, bname, result);
    }

    Ok(())
}
