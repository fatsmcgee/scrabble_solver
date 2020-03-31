use super::util::Letter;
use std::collections::HashMap;
use std::collections::hash_map::Keys;

#[derive(Clone)]
pub struct LetterBag {
    bag: HashMap<Letter, u32>
}


impl LetterBag {
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

    pub fn decremented(&self, key: Letter) -> LetterBag {
        let mut bag = self.clone();
        bag.decrement(key);
        bag
    }

    pub fn new() -> LetterBag {
        LetterBag{bag:HashMap::new()}
    }

    pub fn from_string(s:&str) -> LetterBag {
        let mut lb = LetterBag::new();
        for c in s.bytes() {
            let count = lb.bag.entry(c).or_insert(0);
            *count += 1;
        }
        lb
    }

    pub fn keys(&self) -> Keys<u8,u32> {
        self.bag.keys()
    }
}