use avl::AvlTreeSet;
use std::{
    collections::{hash_map::Keys, HashMap},
    ops::{Index, IndexMut},
};

use serde::ser::{Serialize, SerializeMap};
#[derive(Clone)]
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

    pub fn iter(&self) -> std::vec::IntoIter<V>
    where
        V: Clone,
    {
        let mut s = self
            .v
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(usize, V)>>();
        s.sort_by_key(|(k, _)| *k);

        s.into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<V>>()
            .into_iter()
    }

    pub fn ids(&self) -> Keys<usize, V> {
        self.v.keys()
    }
}

impl<V> Index<usize> for IdMap<V> {
    type Output = V;
    fn index(&self, index: usize) -> &Self::Output {
        &self.v[&index]
    }
}

impl<V> IndexMut<usize> for IdMap<V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.v.get_mut(&index).expect("Id not valid")
    }
}

impl<V> Serialize for IdMap<V>
where
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.v.len()))?;
        let mut s = self.v.iter().collect::<Vec<(&usize, &V)>>();
        s.sort_by_key(|(x, _)|**x);
        for (id, v) in s {
            map.serialize_entry(id, v)?;
        }
        map.end()
    }
}

impl<V> std::fmt::Debug for IdMap<V> where V: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in self.ids() {
            write!(f, "{}: {:?},", i, self[*i])?;
        }
        write!(f, "]")
    }
}

impl<V> std::fmt::Display for IdMap<V> where V: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in self.ids() {
            write!(f, "{}: {},", i, self[*i])?;
        }
        write!(f, "]")
    }
}