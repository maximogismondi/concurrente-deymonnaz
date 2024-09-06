use rayon::prelude::*;
use std::{cmp::Ordering, collections::HashMap};

pub fn truncate_top_elements<K, V, F>(elements: &mut HashMap<K, V>, top_count: usize, comparator: F)
where
    F: Fn(&V, &V) -> Ordering + Sync + Send,
    K: Send + Eq + std::hash::Hash,
    V: Send,
{
    let mut result_elements: Vec<(K, V)> = elements.drain().collect();

    result_elements.par_sort_by(|a, b| comparator(&a.1, &b.1));

    result_elements.truncate(top_count);

    elements.clear();
    elements.extend(result_elements);
}
