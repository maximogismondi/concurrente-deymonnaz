use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

pub fn retain_top_elements<K, V>(elements: &mut HashMap<K, V>, top_count: usize)
where
    K: Hash + Ord + Send,
    V: Ord + Send,
{
    let top_elements = elements
        .drain()
        .par_bridge()
        .fold(BinaryHeap::new, |mut acc_heap, (key, value)| {
            acc_heap.push((Reverse(value), key));

            if acc_heap.len() > top_count {
                acc_heap.pop();
            }
            acc_heap
        })
        .reduce(BinaryHeap::new, |mut acc_heap, local_heap| {
            for (value, key) in local_heap.into_iter() {
                acc_heap.push((value, key));

                if acc_heap.len() > top_count {
                    acc_heap.pop();
                }
            }
            acc_heap
        });

    elements.extend(
        top_elements
            .into_iter()
            .map(|(Reverse(value), key)| (key, value)),
    );
}
