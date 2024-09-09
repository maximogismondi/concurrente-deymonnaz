use std::{collections::BinaryHeap, collections::HashMap, hash::Hash};

pub fn truncate_top_elements<K, V>(elements: &mut HashMap<K, V>, top_count: usize)
where
    K: Eq + Hash + Ord,
    V: Ord,
{
    let mut heap = BinaryHeap::new();

    for (key, value) in elements.drain() {
        heap.push((key, value));
        if heap.len() > top_count {
            heap.pop();
        }
    }

    elements.clear();
    elements.extend(heap);
}
