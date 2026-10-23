//! Command Line Interface of votannic

/* std use */

/* crate use */

/* project use */

#[derive(std::clone::Clone, std::default::Default, clap::Parser, std::fmt::Debug)]
#[clap(
    name = "votannic",
    bin_name = "votannic",
    version = "0.1.0",
    author = "Pierre Marijon <pierre@marijon.fr>"
)]
pub struct Arguments {
    /// Army alpha
    #[clap(short = 'a', long = "alpha")]
    alpha: std::path::PathBuf,

    /// Army beta
    #[clap(short = 'b', long = "beta")]
    beta: std::path::PathBuf,

    // Generic parameter
    /// Silence all output
    #[clap(short = 'q', long = "quiet")]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[clap(short = 'v', long = "verbosity", action = clap::ArgAction::Count)]
    verbosity: u8,

    /// Timestamp (sec, ms, ns, none)
    #[clap(short = 'T', long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,
}

impl Arguments {
    /// Get alpha
    pub fn alpha(&self) -> &std::path::Path {
        &self.alpha
    }

    /// Get beta
    pub fn beta(&self) -> &std::path::Path {
        &self.beta
    }

    /// Get verbosity level
    pub fn verbosity(&self) -> usize {
        self.verbosity as usize
    }

    /// Get quiet
    pub fn quiet(&self) -> bool {
        self.quiet
    }

    /// Get timestamp granularity
    pub fn timestamp(&self) -> stderrlog::Timestamp {
        self.ts.unwrap_or(stderrlog::Timestamp::Off)
    }
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    #[test]
    fn default() {
        let argument = Arguments::default();

        assert_eq!(argument.alpha(), std::path::PathBuf::default());
        assert_eq!(argument.beta(), std::path::PathBuf::default());
        assert_eq!(argument.verbosity(), 0);
        assert_eq!(argument.quiet(), false);
        std::assert_matches!(argument.timestamp(), stderrlog::Timestamp::Off);
    }
}
