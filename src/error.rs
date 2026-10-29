//! Error definition of votannic

/* std use */

/* crate use */

/* module declaration */

/* project use */

/// Enum to define error
#[derive(std::fmt::Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tyche(#[from] tyche::parse::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    Log(#[from] log::SetLoggerError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    FormatError(#[from] std::fmt::Error),
}

/// Alias of result
pub type Result<T> = anyhow::Result<T>;

macro_rules! unreachable {
    ($file:expr, $line: expr) => {
        unreachable!(
            "Code reach an unreachable situation please report: {}:{}",
            $file, $line
        )
    };
}

