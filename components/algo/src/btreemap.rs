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
    fn node() {
        type key = Vec<u8>;
        type value = Vec<u8>;
        let a: Vec<Vec<u8>> = Vec::new();
        let mut v = vec![1i32, 2, 3, 4, 5];
        let x = v.get(0); // Sure
        let y = v.get(1); // Mhmm
        let z = v.get_mut(2); // Should be fine..?
    }
}
