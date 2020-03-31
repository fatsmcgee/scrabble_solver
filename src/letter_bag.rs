use super::util::Letter;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Keys;
use std::iter::FromIterator;

pub type LetterBag = VecLetterBag;

#[derive(Clone)]
pub struct VecLetterBag {
    bag: Vec<(Letter,u32)>
}

impl VecLetterBag {

    fn find_entry_mut(&mut self, key:Letter) -> Option<&mut (Letter,u32)> {
        self.bag.iter_mut().find(|l| l.0==key)
    }

    fn decrement(&mut self, key: Letter) {
        let entry = self.find_entry_mut(key);
        match entry {
            None => {},
            Some((_, value)) => {
                if *value > 0 {
                    *value -= 1;
                }
            }
        }
    }

    pub fn decremented(&self, key: Letter) -> VecLetterBag {
        let mut bag = self.clone();
        bag.decrement(key);
        bag
    }

    pub fn from_string(s:&str) -> VecLetterBag {
        let mut counts = HashMap::new();
        for b in s.bytes() {
            let entry = counts.entry(b).or_insert(0);
            *entry += 1;
        }
        let mut bag = Vec::from_iter(counts);
        bag.shrink_to_fit();
        VecLetterBag{bag}

    }

    pub fn keys(&self) -> impl Iterator<Item=&u8> {
        self.bag.iter()
            .filter(|t| t.1>0)
            .map(|t| &t.0)
    }
}

#[derive(Clone)]
pub struct HashLetterBag {
    bag: HashMap<Letter, u32>
}

impl HashLetterBag {
    fn decrement(&mut self, key: Letter) {
        match self.bag.get(&key) {
            None => {}
            Some(&i) => {
                if i > 1 {
                    self.bag.insert(key, i - 1);
                } else {
                    self.bag.remove(&key);
                }
            }
        }
    }

    pub fn decremented(&self, key: Letter) -> HashLetterBag {
        let mut bag = self.clone();
        bag.decrement(key);
        bag
    }

    pub fn new() -> HashLetterBag {
        HashLetterBag{bag:HashMap::new()}
    }

    pub fn from_string(s:&str) -> HashLetterBag {
        let mut lb = HashLetterBag::new();
        for c in s.bytes() {
            let count = lb.bag.entry(c).or_insert(0);
            *count += 1;
        }
        lb
    }

    pub fn keys(&self) -> impl Iterator<Item=&u8> {
        self.bag.keys()
    }
}