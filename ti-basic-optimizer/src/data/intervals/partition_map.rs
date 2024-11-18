//! Partition Map
//!
//! Map from non-overlapping half-open `[begin, end)` subsets of a range to arbitrary
//! data. The subsets must cover the entire set.

/// Map from non-overlapping half-open `[begin, end)` subsets of a range to arbitrary
/// data. The subsets must cover the entire set.
#[derive(Debug)]
pub struct PartitionMap<K: Ord + Sized, V: Sized> {
    pub breaks: Vec<K>,
    pub values: Vec<V>,
}

impl<K, V> PartitionMap<K, V>
where
    K: Ord + Sized,
    V: Sized,
{
    /// Construct a partition map from a *sorted* list of range beginnings and their corresponding values.
    ///
    /// Panics if `begins` and `values` are different sizes or if `begins` is not sorted.
    pub fn new<I: Into<Vec<K>>, J: Into<Vec<V>>>(begins: I, values: J) -> Self {
        let breaks: Vec<K> = begins.into();
        let values: Vec<V> = values.into();

        assert_eq!(breaks.len(), values.len());
        assert!(breaks.is_sorted());

        PartitionMap { breaks, values }
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        if self.breaks.is_empty() || key < &self.breaks[0] {
            return None;
        }

        let index = self.breaks.partition_point(|x| x <= key) - 1;
        Some(&self.values[index])
    }

    /// Returns `true` if the provided keys are in the same range.
    pub fn in_same_range(&self, key_a: &K, key_b: &K) -> bool {
        self.breaks.partition_point(|x| x <= key_a) == self.breaks.partition_point(|x| x <= key_b)
    }
}

#[cfg(test)]
mod tests {
    use super::PartitionMap;

    #[test]
    fn find() {
        let map = PartitionMap::new([1, 2, 4, 8], ['A', 'B', 'C', 'D']);

        assert_eq!(map.find(&0), None);
        assert_eq!(map.find(&1), Some(&'A'));
        assert_eq!(map.find(&2), Some(&'B'));
        assert_eq!(map.find(&3), Some(&'B'));
        assert_eq!(map.find(&4), Some(&'C'));
        assert_eq!(map.find(&7), Some(&'C'));
        assert_eq!(map.find(&8), Some(&'D'));
        assert_eq!(map.find(&99), Some(&'D'));
    }
}
