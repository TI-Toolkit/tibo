use crate::optimize::Priority;
use titokens::Version;

#[derive(Clone, Debug)]
/// Optimizer Configuration.
pub struct Config {
    /// Minimum Requested Output Version, i.e. the earliest OS version that the output of the
    /// optimizer is expected to run on.
    pub mrov: Version,

    pub priority: Priority,
}

impl From<Version> for Config {
    fn from(value: Version) -> Self {
        Self {
            mrov: value,
            priority: Priority::Neutral,
        }
    }
}
