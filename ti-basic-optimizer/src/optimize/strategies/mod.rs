//! # Strategies
//! Sometimes a decision needs to be made between several competing ways of doing the same thing.
//!
//! `Strategy` provides a systematic way to compare these alternatives so that adding a new strategy
//! is easy. See [`numeric_literal`] for an example of how `Strategy` can be used to implement a
//! peephole optimization for numeric literals.
mod numeric_literal;

use crate::optimize::Priority;
use crate::parse::Reconstruct;
use crate::Config;
use std::cmp::Ordering;
use titokens::Token;

pub trait Strategy<T>: Reconstruct {
    fn exists(&self) -> bool;

    /// The exact number of bytes that this `Strategy` would use.
    fn size_cost(&self) -> Option<usize>;
    /// Estimation of the average clock cycles that this `Strategy` would use.
    fn speed_cost(&self) -> Option<u32>;
}

impl<T> Strategy<T> for Box<dyn Strategy<T>> {
    fn exists(&self) -> bool {
        (**self).exists()
    }

    fn size_cost(&self) -> Option<usize> {
        (**self).size_cost()
    }

    fn speed_cost(&self) -> Option<u32> {
        (**self).speed_cost()
    }
}

impl<T> Reconstruct for Box<dyn Strategy<T>> {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        (**self).reconstruct(config)
    }
}

/// Compare two `Strategies`. This function makes the resource-allocation decision for balancing
/// speed and size under [neutral](Priority::Neutral) optimization
fn partial_cmp<T>(
    a: &dyn Strategy<T>,
    b: &dyn Strategy<T>,
    priority: Priority,
) -> Option<Ordering> {
    match priority {
        Priority::Neutral => {
            let my_cost = (a.size_cost()? as u64).saturating_mul(a.speed_cost()? as u64);
            let other_cost = (b.size_cost()? as u64).saturating_mul(b.speed_cost()? as u64);

            Some(my_cost.cmp(&other_cost))
        }
        Priority::Speed => a.speed_cost().partial_cmp(&b.speed_cost()),
        Priority::Size => a.size_cost().partial_cmp(&b.size_cost()),
    }
}

impl<T> Reconstruct for Vec<Box<dyn Strategy<T>>> {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        self.iter()
            .filter(|&x| x.exists())
            .min_by(|&a, &b| {
                partial_cmp(a, b, config.priority)
                    .expect("Strategy which `exists` returned `None` for a `_cost`.")
            })
            .map(|x| x.reconstruct(config))
            .expect("No strategies were available!")
    }
}
