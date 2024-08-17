use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Model {
    #[serde(rename = "TI-82")]
    TI82,

    #[serde(rename = "TI-83")]
    TI83,

    #[serde(rename = "TI-83+")]
    TI83P,

    #[serde(rename = "TI-84+")]
    TI84P,
    #[serde(rename = "TI-84+T")]
    TI84PT,
    #[serde(rename = "TI-82A")]
    TI82A,

    #[serde(rename = "TI-84+CSE")]
    TI84PCSE,

    #[serde(rename = "TI-84+CE")]
    TI84PCE,
    #[serde(rename = "TI-84+CE-T")]
    TI84PCET,
    #[serde(rename = "TI-83PCE")]
    TI83PCE,
    #[serde(rename = "TI-83PCEEP")]
    TI83PCEEP,
    #[serde(rename = "TI-84+CEPY")]
    TI84PCEPY,
    #[serde(rename = "TI-84+CE-TPE")]
    TI84PCETPE,

    LATEST,
}

impl Model {
    fn value(&self) -> u8 {
        match self {
            Model::TI82 => 10,

            Model::TI83 => 20,

            Model::TI83P => 30,

            Model::TI84P | Model::TI84PT | Model::TI82A => 40,

            Model::TI84PCSE => 50,

            Model::TI84PCE
            | Model::TI84PCET
            | Model::TI83PCE
            | Model::TI83PCEEP // TI, these are getting a bit out of hand
            | Model::TI84PCEPY
            | Model::TI84PCETPE => 60,

            Model::LATEST => u8::MAX,
        }
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for Model {}

impl PartialOrd for Model {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Model {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

/// Pinpoints a specific point in the token sheet OS version timeline.
///
/// ## Motivation and Details
/// The token sheet forces the calculator operating system versions into a clean
/// linear timeline spanning over 30 years. There are several essential patterns
/// that allow for this:
///
/// 1. Newer calculators in the lineage generally get the first version of their
///    token table directly from the last version of their immediate predecessor
/// 2. Older calculators generally do not receive patches after a newer model is
///    introduced.
/// 3. The changes from version to version (calculator to calculator changes are
///    version to version changes given the previous points) are usually limited
///    to addition, renaming, and omission. We don't see dramatic reorganization
///    of the token sheet.
/// 4. Critically, any violation of these patterns (that we have encountered) is
///    handled nicely by the current system.
///
/// We can then track the history of any token with a series of half-open ranges
/// placed on this timeline. We use `[since, until)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Version {
    #[serde(with = "quick_xml::serde_helpers::text_content")]
    pub model: Model,
    pub os_version: String,
}

impl Version {
    pub fn latest() -> Self {
        Version {
            model: Model::LATEST,
            os_version: "9.99.99".to_string(),
        }
    }
}

fn cmp_os_version(a: &str, b: &str) -> Ordering {
    a.split('.')
        .map(|n| str::parse::<u64>(n).unwrap())
        .cmp(b.split('.').map(|n| str::parse::<u64>(n).unwrap()))
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.model
            .cmp(&other.model)
            .then_with(|| cmp_os_version(&self.os_version, &other.os_version))
    }
}
