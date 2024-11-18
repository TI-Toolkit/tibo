//! # Interval Tree
//! Interval Tree implementation
//!
//! This just does a linear scan for stabbing queries. This is a great target for optimizing the compiler.

use std::ops::Range;

#[derive(Debug)]
pub struct IntervalTree<T: Ord + Clone> {
    data: Vec<Range<T>>,
}

impl<T: Ord + Clone> IntervalTree<T> {
    pub fn new(mut ranges: Vec<Range<T>>) -> Self {
        Self { data: ranges }
    }

    pub fn stab(&self, point: T) -> Vec<Range<T>> {
        self.data
            .iter()
            .filter(|&range| range.contains(&point))
            .cloned()
            .collect()
    }
}
