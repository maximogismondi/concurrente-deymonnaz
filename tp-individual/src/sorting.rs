use rayon::prelude::*;
use std::{cmp::Ordering, collections::HashMap};

pub fn find_top_elements<K, V, F>(
    elements: HashMap<K, V>,
    top_count: usize,
    comparator: F,
) -> HashMap<K, V>
where
    F: Fn(&V, &V) -> Ordering + Sync,
    K: Send + Eq + std::hash::Hash,
    V: Send,
{
    let mut result_elements = elements.into_iter().collect::<Vec<_>>();

    result_elements.par_sort_by(|a, b| comparator(&a.1, &b.1));

    result_elements.truncate(top_count);

    result_elements.into_iter().collect()
}
