//! newrecruit module

/* std use */

/* crate use */

/* project use */

#[derive(
    std::clone::Clone,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct NewRecruit {
    pub roster: Roster,
}

#[derive(
    std::clone::Clone,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Roster {
    pub id: Option<String>,
    pub name: String,
    pub battle_scribe_version: f64,
    pub generated_by: String,
    pub game_system_id: String,
    pub game_system_name: String,
    pub game_system_revision: u64,
    pub xmlns: String,
    pub costs: Vec<Cost>,
    pub cost_limits: Vec<Cost>,
    pub forces: Vec<Force>,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    pub name: String,
    pub type_id: String,
    pub value: u64,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Force {
    pub id: String,
    pub name: String,
    pub entry_id: String,
    pub catalogue_id: String,
    pub catalogue_revision: u64,
    pub catalogue_name: String,
    pub selections: Vec<Selection>,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub rules: Option<Vec<Rule>>,
    pub profiles: Option<Vec<Profile>>,
    pub selections: Option<Vec<Selection>>,
    pub costs: Option<Vec<Cost>>,
    pub categories: Option<Vec<Category>>,
    pub id: String,
    pub name: String,
    pub entry_id: String,
    pub entry_group_id: Option<String>,
    pub number: u8,
    pub type_id: Option<String>,
    pub type_name: Option<String>,
    pub from: String,
    pub group: Option<String>,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub description: String,
    pub id: String,
    pub name: String,
    pub hidden: bool,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub characteristics: Vec<Characteristic>,
    pub id: String,
    pub name: String,
    pub hidden: bool,
    pub type_id: String,
    pub type_name: String,
    pub from: String,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub entry_id: String,
    pub name: String,
    pub primary: bool,
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::default::Default,
    std::fmt::Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Characteristic {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub name: String,
    pub type_id: String,
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
}
