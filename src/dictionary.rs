use super::util::{Letter,Word};


const ROOT_NODE_IDX: usize = 0;

fn letter_alpha_idx(c: Letter) -> u8 {
    if (b'a'..=b'z').contains(&c) {
        c - b'a'
    } else if (b'A'..=b'Z').contains(&c) {
        c - b'A'
    } else {
        println!("{}", c);
        panic!("Non alphabetical character {}", char::from(c));
    }
}

fn word_to_alpha_indices(w: &Word) -> Vec<u8> {
    w.bytes()
        .map(letter_alpha_idx)
        .collect()
}

fn utf8_to_alpha_indices(w: &Vec<u8>) -> Vec<u8> {
    w.iter()
        .cloned()
        .map(letter_alpha_idx)
        .collect()
}


pub struct DictionaryTrie {
    entries: Vec<DictionaryTrieNode>
}

pub struct DictionaryTrieNodePtr<'a> {
    dict_trie: &'a DictionaryTrie,
    node: &'a DictionaryTrieNode,
}

impl<'a> DictionaryTrieNodePtr<'a> {
    pub fn is_word(&self) -> bool {
        self.node.is_word
    }

    pub fn get_child(&self, c: Letter) -> Option<DictionaryTrieNodePtr<'a>> {
        self.get_child_idx(letter_alpha_idx(c))
    }

    pub fn get_child_idx(&self, alpha_idx: u8) -> Option<DictionaryTrieNodePtr<'a>> {
        match self.node.get_child_idx(alpha_idx) {
            None => None,
            Some(new_idx) => {
                let new_node = &self.dict_trie.entries[new_idx];
                let new_ptr = DictionaryTrieNodePtr { dict_trie: self.dict_trie, node: new_node };
                Some(new_ptr)
            }
        }
    }

    pub fn dict_trie(&self) -> &'a DictionaryTrie {
        self.dict_trie
    }
}


impl DictionaryTrie {
    pub fn new() -> DictionaryTrie {
        let mut entries = Vec::new();
        entries.push(DictionaryTrieNode::new());
        DictionaryTrie { entries: entries }
    }

    //
    fn from_word_list(word_list:&str) -> DictionaryTrie {
        let mut trie = DictionaryTrie::new();
        word_list.split_ascii_whitespace()
            .into_iter()
            .for_each(|s| trie.add_word(s.to_owned()));
        trie
    }

    pub fn from_scrabble_2019() -> DictionaryTrie {
        let scrabble_dict = include_str!("../resources/scrabble_dictionary_2019.txt");
        Self::from_word_list(scrabble_dict)
    }

    pub fn from_scrabble_ospd() -> DictionaryTrie {
        let scrabble_dict = include_str!("../resources/scrabble_dictionary_ospd.txt");
        Self::from_word_list(scrabble_dict)
    }

    pub fn add_word(&mut self, s: Word) {
        self.add_alpha_indices(word_to_alpha_indices(&s))
    }

    pub fn add_alpha_indices(&mut self, alpha_indices: Vec<u8>) {
        let mut node_idx = ROOT_NODE_IDX;

        for c in alpha_indices.iter() {
            node_idx = if let Some(next_idx) =
            self.entries[node_idx].get_child_idx(*c) {
                //This node has a child node that has been created
                next_idx
            } else {
                //Create a new child node
                self.entries.push(DictionaryTrieNode::new());
                let new_node_idx = self.entries.len() - 1;
                self.entries[node_idx].set_child_idx(*c, new_node_idx);
                new_node_idx
            }
        }

        self.entries[node_idx].is_word = true;
    }

    pub fn root(&self) -> DictionaryTrieNodePtr {
        DictionaryTrieNodePtr { dict_trie: self, node: &self.entries[ROOT_NODE_IDX] }
    }

    pub fn find_node_from_alpha_indices(&self, alpha_indices: Vec<u8>) -> Option<DictionaryTrieNodePtr> {
        alpha_indices.iter()
            .try_fold(self.root(),
                      |node, code| node.get_child_idx(*code))
    }

    pub fn is_word(&self, s: &Word) -> bool {
        self.are_alpha_indices_word(word_to_alpha_indices(s))
    }

    pub fn is_word_utf8(&self, s: &Vec<u8>) -> bool {
        self.are_alpha_indices_word(utf8_to_alpha_indices(s))
    }


    pub fn are_alpha_indices_word(&self, word: Vec<u8>) -> bool {
        self.find_node_from_alpha_indices(word).map_or(false, |n| n.is_word())
    }
}


struct DictionaryTrieNode {
    is_word: bool,
    child_nodes: [usize; 26],
}

impl DictionaryTrieNode {
    pub fn new() -> DictionaryTrieNode {
        DictionaryTrieNode { is_word: false, child_nodes: [ROOT_NODE_IDX; 26] }
    }

    pub fn set_child_idx(&mut self, alpha_idx: u8, child_idx: usize) {
        self.child_nodes[alpha_idx as usize] = child_idx;
    }

    pub fn get_child_idx(&self, alpha_index: u8) -> Option<usize> {
        match self.child_nodes[alpha_index as usize] {
            ROOT_NODE_IDX => None,
            i => Some(i)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //Trie tests
    #[test]
    fn trie_contains() {
        let mut trie = DictionaryTrie::new();

        trie.add_word(String::from("dog"));
        trie.add_word(String::from("dogcat"));

        assert!(trie.is_word(&String::from("dog")));
        assert!(trie.is_word(&String::from("dogcat")));
        assert!(!trie.is_word(&String::from("do")));
        assert!(!trie.is_word(&String::from("a")));
        assert!(!trie.is_word(&String::from("dogcatz")));
    }

    #[test]
    fn trie_traversal() {
        let mut trie = DictionaryTrie::new();

        trie.add_word(String::from("dog"));

        let mut ptr = trie.root();
        assert!(!ptr.is_word());
        ptr = ptr.get_child(b'd').unwrap();
        ptr = ptr.get_child(b'o').unwrap();
        assert!(!ptr.is_word());
        ptr = ptr.get_child(b'g').unwrap();
        assert!(ptr.is_word());
    }

    #[test]
    fn scrabble_trie() {
        let trie = DictionaryTrie::from_scrabble_ospd();
        assert!(trie.is_word(&String::from("brawns")));
    }
}