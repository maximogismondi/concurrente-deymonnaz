use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

pub fn retain_top_elements<K, V>(elements: &mut HashMap<K, V>, top_count: usize)
where
    K: Eq + Hash + Ord,
    V: Ord,
{
    let mut heap = BinaryHeap::new();

    for (key, value) in elements.drain() {
        // push reversed value to the heap
        heap.push(((Reverse(value)), key));

        if heap.len() > top_count {
            heap.pop();
        }
    }

    elements.extend(heap.into_iter().map(|(Reverse(value), key)| (key, value)));
}
