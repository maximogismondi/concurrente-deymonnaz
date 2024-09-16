use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

/// A capped min heap map.
/// It will keep the top `capacity` elements.
struct CappedMinHeapMap<K, V> {
    heap: BinaryHeap<(Reverse<V>, K)>,
    capacity: usize,
}

impl<K, V> CappedMinHeapMap<K, V>
where
    K: Ord,
    V: Ord,
{
    /// Creates a new `CappedMinHeapMap` instance.
    fn new(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            capacity,
        }
    }

    /// Pushes a key-value pair into the heap.
    /// If the heap is full, it will remove the smallest element.
    fn push(&mut self, key: K, value: V) {
        self.heap.push((Reverse(value), key));

        if self.heap.len() > self.capacity {
            self.heap.pop();
        }
    }

    /// Consumes the heap and returns an iterator with the key-value pairs.
    fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.heap
            .into_iter()
            .map(|(Reverse(value), key)| (key, value))
    }

    /// Merges another `CappedMinHeapMap` into this one.
    fn merge(&mut self, other: Self) {
        for (key, value) in other.into_iter() {
            self.push(key, value);
        }
    }
}

/// Retains the top `top_count` elements in the given map.
/// The map will be modified in place.
/// If the map has less elements than `top_count`, all elements will be kept.
/// If the map is empty, it will remain empty.
/// The elements are retained based on their `Ord` implementation.
pub fn retain_top_elements<K, V>(elements: &mut HashMap<K, V>, top_count: usize)
where
    K: Hash + Ord + Send,
    V: Ord + Send,
{
    let top_elements = elements
        .drain()
        .par_bridge()
        .fold(
            || CappedMinHeapMap::new(top_count),
            |mut acc_heap, (key, value)| {
                acc_heap.push(key, value);
                acc_heap
            },
        )
        .reduce(
            || CappedMinHeapMap::new(top_count),
            |mut acc_heap, local_heap| {
                acc_heap.merge(local_heap);
                acc_heap
            },
        );

    elements.extend(top_elements.into_iter());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retain_top_elements() {
        let mut elements = vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]
            .into_iter()
            .collect();

        retain_top_elements(&mut elements, 3);

        assert_eq!(elements.len(), 3);
        assert_eq!(elements.get(&3), Some(&3));
        assert_eq!(elements.get(&4), Some(&4));
        assert_eq!(elements.get(&5), Some(&5));
    }

    #[test]
    fn test_retain_empty_map() {
        let mut elements: HashMap<usize, usize> = HashMap::new();

        retain_top_elements(&mut elements, 3);

        assert!(elements.is_empty());
    }

    #[test]
    fn test_retain_less_elements_than_capacity() {
        let mut elements = vec![(1, 1), (2, 2)].into_iter().collect();

        retain_top_elements(&mut elements, 3);

        assert_eq!(elements.len(), 2);
        assert_eq!(elements.get(&1), Some(&1));
        assert_eq!(elements.get(&2), Some(&2));
    }
}
