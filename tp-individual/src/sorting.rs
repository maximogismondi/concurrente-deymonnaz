use rayon::prelude::*;
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
        heap.push(((Reverse(value)), key));

        if heap.len() > top_count {
            heap.pop();
        }
    }

    elements.extend(heap.into_iter().map(|(Reverse(value), key)| (key, value)));
}

pub fn _retain_top_elements<K, V>(elements: &mut HashMap<K, V>, top_count: usize)
where
    K: Send + Eq + Hash,
    V: Send + Ord,
{
    let mut result_elements: Vec<(K, V)> = elements.drain().collect();

    result_elements.par_sort_by(|a, b| a.1.cmp(&b.1).reverse());

    result_elements.truncate(top_count);

    elements.extend(result_elements);
}
