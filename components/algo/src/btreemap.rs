use std::cmp::Ordering;

struct Node<K: Ord, V> {
    keys: Vec<K>,
    edges: Vec<Node<K, V>>,
    values: Vec<V>,
}

impl<K: Ord, V> Node<K, V> {
    pub fn search(&self, key: &K) -> SearchResult {
        self.search_linear(key)
    }
    fn search_linear(&self, key: &K) -> SearchResult {
        for (i, k) in self.keys.iter().enumerate() {
            match k.cmp(key) {
                Ordering::Less => {}
                Ordering::Equal => return SearchResult::Found(i),
                Ordering::Greater => return SearchResult::GoDown(i),
            }
        }
        SearchResult::GoDown(self.keys.len())
    }
}
enum SearchResult {
    Found(usize),
    GoDown(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node() {}
}
