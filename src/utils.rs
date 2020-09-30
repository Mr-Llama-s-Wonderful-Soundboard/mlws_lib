use avl::AvlTreeSet;
use std::collections::HashMap;

pub struct IdMap<V> {
    v: HashMap<usize, V>,
    holes: AvlTreeSet<usize>,
    len: usize,
}

impl<V> IdMap<V> {
    pub fn new() -> Self {
        Self {
            v: HashMap::new(),
            holes: AvlTreeSet::new(),
            len: 0,
        }
    }

    pub fn add(&mut self, val: V) -> usize {
        if self.holes.is_empty() {
            self.v.insert(self.len, val);
            let r = self.len;
            self.len += 1;
            r
        } else {
            let r = *(&self.holes).into_iter().next().unwrap();
            self.holes.remove(&r);
            self.v.insert(r, val);
            r
        }
    }

    pub fn remove(&mut self, id: usize) -> Option<V> {
        let r = self.v.remove(&id);
        if r.is_some() {
            self.holes.remove(&id);
        }
        r
    }
}
