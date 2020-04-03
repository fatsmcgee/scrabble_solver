mod dictionary;
mod grid;
mod letter_bag;
mod util;

use std::collections::{HashMap, VecDeque};
use std::slice::from_ref;
use lazy_static::lazy_static;
use serde::Serialize;


pub use util::Direction;
use util::{Letter,Word};
pub use letter_bag::LetterBag;
pub use dictionary::DictionaryTrie;
use dictionary::DictionaryTrieNodePtr;
pub use grid::Coord;
use grid::Grid;
use std::fmt::{Display, Formatter, Error};

const CAPITAL_A_TO_Z:&str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const WILDCARD_LETTER:Letter = b'*';

lazy_static! {
    //Official Scrabble letter values
    static ref LETTER_VALUES: [u32;26] = {
        let mut letter_values = [0;26];

        let letter_values_spec = include_str!("../resources/scrabble_letter_values.txt");
        for line in letter_values_spec.trim().split('\n') {
            let mut parts = line.split_ascii_whitespace();
            let letters = parts.next().unwrap();
            let value = parts.next().unwrap();
            let value = value.parse::<u32>().unwrap();

            for l in letters.chars() {
                let idx = ((l as u8) - b'a') as usize;
                letter_values[idx] = value;
            }

        }
        letter_values
    };
}



#[derive(Clone, Debug)]
enum Modifier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}


impl Modifier {
    fn from_char_spec(c: char) -> Option<Modifier> {
        match c {
            'd' => Some(Modifier::DoubleLetter),
            't' => Some(Modifier::TripleLetter),
            'D' => Some(Modifier::DoubleWord),
            'T' => Some(Modifier::TripleWord),
            ' ' => None,
            _ => panic!("Unexpected character for modifier")
        }
    }
}


#[derive(Clone)]
pub struct ScrabbleBoard {
    modifiers: Grid<Option<Modifier>>,
    letters: Grid<Option<Letter>>,
}

#[derive(Clone, Serialize)]
pub struct ScrabbleSolution {
    pub word: Word,
    pub score: u32,
    pub direction: Direction,
    pub start_coord: Coord,
}

impl Display for ScrabbleSolution {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f,
               "{}: {},{}({}): {} points",
                self.word,
                self.start_coord.row,
                self.start_coord.col,
                if let Direction::Right = self.direction {"R"} else {"D"},
                self.score);
        Ok(())
    }
}

struct ScrabbleSolutionBuilder<'a> {
    word_so_far: Word,
    trie_ptr: DictionaryTrieNodePtr<'a>,
    letters_available: LetterBag,
    anchored: bool,
    letters_placed: u32,
    letter_score: u32,
    word_multiplier: u32,
    addon_score: u32,
}

enum WordAroundValidation {
    InvalidWord,
    NothingAround,
    ValidWordScore(u32),
}

impl<'a> ScrabbleSolutionBuilder<'a> {
    fn new(letters_available: LetterBag, dict_trie: &'a DictionaryTrie) -> ScrabbleSolutionBuilder {
        ScrabbleSolutionBuilder {
            word_so_far: String::from(""),
            trie_ptr: dict_trie.root(),
            letters_available: letters_available.clone(),
            anchored: false,
            letters_placed: 0,
            letter_score: 0,
            word_multiplier: 1,
            addon_score: 0,
        }
    }

    fn final_score(&self) -> u32 {
        self.letter_score * self.word_multiplier
            + self.addon_score
            + if self.letters_placed >= 7 { 50 } else { 0 }
    }

    fn build(&self, dir: Direction, end_coord: Coord) -> ScrabbleSolution {
        let strlen = self.word_so_far.len() as i32;
        let start_coord = match dir {
            Direction::Right => Coord { col: end_coord.col - strlen, ..end_coord },
            Direction::Down => Coord { row: end_coord.row - strlen, ..end_coord }
        };
        ScrabbleSolution {
            word: self.word_so_far.clone(),
            score: self.final_score(),
            direction: dir,
            start_coord,
        }
    }

    fn is_valid_solution(&self) -> bool {
        self.anchored && (self.letters_placed > 0) && self.trie_ptr.is_word()
    }

    fn get_trie_child(&self, l: Letter) -> Option<DictionaryTrieNodePtr> {
        self.trie_ptr.get_child(l)
    }

    fn dict_trie(&self) -> &DictionaryTrie {
        self.trie_ptr.dict_trie()
    }
}

fn scrabble_letter_score(l: Letter) -> u32 {
    if (b'A'..=b'Z').contains(&l) {
        //capitals, the blank equivalents of normal letters have zero points
        0
    } else if (b'a'..=b'z').contains(&l) {
        let idx = (l - b'a') as usize;
        LETTER_VALUES[idx]
    } else {
        panic!("Letter '{}' has no value", l)
    }
}

fn scrabble_letters_score_utf8(w: &Vec<u8>) -> u32 {
    w.iter()
        .fold(0, |score, l|
            score + scrabble_letter_score(*l))
}



pub fn print_top_solutions(solutions: &Vec<ScrabbleSolution>,
                            limit: Option<usize>) {
    let mut sorted_solutions = solutions.clone();
    sorted_solutions.sort_by(
        |a,b| b.score.cmp(&a.score));
    for (i,solution) in sorted_solutions.iter().enumerate() {
        println!("{}: {},{}({}): {} points", solution.word,
                 solution.start_coord.row,
                 solution.start_coord.col,
                 if let Direction::Right = solution.direction {"R"} else {"D"},
                 solution.score);
        if let Some(limit) = limit {
            if i>limit {
                break;
            }
        }
    }
}


impl ScrabbleBoard {
    pub fn empty_scrabble_board() -> ScrabbleBoard {
        const SCRABBLE_DIM: usize = 15;
        let scrab_modifiers_spec = include_str!("../resources/scrabble_modifiers.txt");
        let modifiers: Vec<Vec<Option<Modifier>>> = scrab_modifiers_spec.trim().split('\n')
            .into_iter()
            .map(|line| line.chars().map(Modifier::from_char_spec).collect())
            .collect();
        let modifiers = Grid::from(modifiers);

        let letters = Grid::new(SCRABBLE_DIM, SCRABBLE_DIM, None);

        ScrabbleBoard { modifiers, letters }
    }

    fn nrows(&self) -> usize {
        self.modifiers.nrows()
    }

    fn ncols(&self) -> usize {
        self.modifiers.ncols()
    }

    pub fn add_word(&mut self, coord:Coord, direction:Direction, word:&str) {
        let mut cur_coord = coord;
        for l in word.bytes() {
            self.set_letter_unchecked(cur_coord, l);
            cur_coord = cur_coord.next(direction);
        }
    }

    pub fn set_letter_unchecked(&mut self, coord: Coord, l: Letter) {
        self.letters.set_unchecked(coord, Some(l))
    }

    pub fn print_board(&self) {
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                let i = i as i32;
                let j = j as i32;
                let c = match self.letters.get_unchecked(Coord::new(i, j)) {
                    None => match self.modifiers.get_unchecked(Coord::new(i, j)) {
                        Some(Modifier::DoubleLetter) => '2',
                        Some(Modifier::TripleLetter) => '3',
                        Some(Modifier::DoubleWord) => '②',
                        Some(Modifier::TripleWord) => '③',
                        None => ' '
                    }
                    Some(l) => char::from(l)
                };
                print!("{}", c);
            }
            print!("{}", '\n');
        }
    }

    fn is_middle(&self, coord: Coord) -> bool {
        return coord.row == (self.nrows() as i32) / 2
            && coord.col == (self.ncols() as i32) / 2;
    }

    fn is_coord_in_bounds(&self, coord: Coord) -> bool {
        self.letters.is_coord_in_bounds(coord)
    }

    pub fn find_all_valid_words(&self,
                                letters_available:&LetterBag,
                                dict: &DictionaryTrie) -> Vec<ScrabbleSolution> {
        let mut all_solutions = Vec::new();
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                for dir in &[Direction::Right, Direction::Down] {
                    let solutions =
                        self.find_valid_words_coord(Coord::new(i as i32,j as i32),
                                                    *dir,
                                                    letters_available.clone(),
                                                    dict);
                    all_solutions.extend(solutions);
                }
            }
        }
        all_solutions
    }

    fn find_valid_words_coord(&self,
                              coord: Coord,
                              dir: Direction,
                              letters_available: LetterBag,
                              dict: &DictionaryTrie) -> Vec<ScrabbleSolution> {

        let prev_coord = coord.prev(dir);
        if self.is_coord_in_bounds(prev_coord) &&
            self.letters.get_unchecked(prev_coord).is_some() {
            //Only start on occupied placedLetters if there is nothing coming before them
            return Vec::new();
        }

        let solution_builder
            = ScrabbleSolutionBuilder::new(letters_available, dict);
        self.find_valid_words_coord_helper(coord, dir, solution_builder)
    }

    fn find_valid_words_coord_helper(&self,
                                     coord: Coord,
                                     dir: Direction,
                                     solution_so_far: ScrabbleSolutionBuilder) -> Vec<ScrabbleSolution> {

        let mut solutions = Vec::new();

        if !self.is_coord_in_bounds(coord) {
            //if we went off the board and have formed a valid solution, add it
            if solution_so_far.is_valid_solution() {
                solutions.push(solution_so_far.build(dir, coord));
            }
            return solutions;
        }

        match self.letters.get_unchecked(coord) {
            //No placedLetters on board at this coordinate
            None => {
                //add the word so far as a solution since this is a blank
                if solution_so_far.is_valid_solution() {
                    solutions.push(solution_so_far.build(dir, coord))
                }

                //recurse on solutions involving available placedLetters
                let letters_available =
                    &solution_so_far.letters_available;

                for &bag_letter in letters_available.keys() {
                    let actual_letters = if bag_letter == WILDCARD_LETTER {
                        //If the wildcard letter is used, go through all
                        // placedLetters in the alphabet (capital denotes the wildcard version)
                        CAPITAL_A_TO_Z.as_bytes()
                    } else {
                        from_ref(&bag_letter)
                    };

                    for l in actual_letters {
                        let next_trie_child
                            = solution_so_far.get_trie_child(l.to_ascii_lowercase());
                        let base_letter_score = scrabble_letter_score(*l);
                        let (added_letter_score, extra_word_multiplier) =
                            match self.modifiers.get_unchecked(coord) {
                                Some(Modifier::DoubleLetter) => (2 * base_letter_score, 1),
                                Some(Modifier::TripleLetter) => (3 * base_letter_score, 1),
                                Some(Modifier::DoubleWord) => (base_letter_score, 2),
                                Some(Modifier::TripleWord) => (base_letter_score, 3),
                                _ => (base_letter_score, 1)
                            };

                        //This is a valid prefix in the Trie, so could lead to a word
                        if let Some(next_trie_node)
                        = next_trie_child {


                            //Check that the surrounding letters are valid too
                            let (has_anchor, addon_score) =
                                match self.validate_word_around(solution_so_far.dict_trie(),
                                                                coord,
                                                                dir.rotate(),
                                                                *l) {
                                    WordAroundValidation::InvalidWord => {
                                        continue;
                                    }
                                    WordAroundValidation::NothingAround => {
                                        (false, 0)
                                    }
                                    WordAroundValidation::ValidWordScore(score) => {
                                        (false, score)
                                    }
                                };

                            let mut next_word_so_far = solution_so_far.word_so_far.clone();
                            next_word_so_far.push(char::from(*l));

                            let next_letters_available =
                                letters_available.decremented(bag_letter);

                            let next_solution_builder = ScrabbleSolutionBuilder {
                                word_so_far: next_word_so_far,
                                trie_ptr: next_trie_node,
                                letters_available: next_letters_available,
                                anchored:
                                solution_so_far.anchored || self.is_middle(coord) || has_anchor,
                                letters_placed: solution_so_far.letters_placed + 1,
                                addon_score: solution_so_far.addon_score + addon_score,
                                letter_score: solution_so_far.letter_score + added_letter_score,
                                word_multiplier:
                                    solution_so_far.word_multiplier * extra_word_multiplier,
                            };

                            let next_solutions = self.find_valid_words_coord_helper(
                                coord.next(dir),
                                dir,
                                next_solution_builder,
                            );
                            solutions.extend(next_solutions);
                        }
                    }
                }
            }
            Some(l) => {
                //There is an existing letter here, recurse on it
                let next_trie_child =
                    solution_so_far.get_trie_child(l);

                if let Some(next_trie_node) = next_trie_child {
                    let mut next_word_so_far = solution_so_far.word_so_far.clone();
                    next_word_so_far.push(char::from(l));

                    let next_solution_builder = ScrabbleSolutionBuilder {
                        word_so_far: next_word_so_far,
                        trie_ptr: next_trie_node,
                        letters_available: solution_so_far.letters_available.clone(),
                        anchored: true,
                        letters_placed: solution_so_far.letters_placed,
                        addon_score: solution_so_far.addon_score,
                        letter_score: solution_so_far.letter_score + scrabble_letter_score(l),
                        word_multiplier: solution_so_far.word_multiplier,
                    };

                    let next_solutions = self.find_valid_words_coord_helper(
                        coord.next(dir),
                        dir,
                        next_solution_builder,
                    );
                    solutions.extend(next_solutions);
                }
            }
        }

        solutions
    }

    fn validate_word_around(&self,
                            dict_trie: &DictionaryTrie,
                            coord: Coord,
                            dir: Direction,
                            letter: Letter) -> WordAroundValidation {
        let mut letters_around = VecDeque::new();


        let mut prev_coord = coord.prev(dir);
        while self.is_coord_in_bounds(prev_coord) {
            if let Some(l) = self.letters.get_unchecked(prev_coord) {
                letters_around.push_front(l);
                prev_coord = prev_coord.prev(dir);
            } else {
                break;
            }
        }

        letters_around.push_back(letter);

        let mut next_coord = coord.next(dir);
        while self.is_coord_in_bounds(next_coord) {
            if let Some(l) = self.letters.get_unchecked(next_coord) {
                letters_around.push_back(l);
                next_coord = next_coord.next(dir);
            } else {
                break;
            }
        }

        if letters_around.len() == 1 {
            WordAroundValidation::NothingAround
        } else {
            let word_around = letters_around.iter().cloned().collect();
            if dict_trie.is_word_utf8(&word_around){
                //Valid word
                let base_score = scrabble_letters_score_utf8(&word_around);
                let final_score = match self.modifiers.get_unchecked(coord) {
                    None => base_score,
                    Some(Modifier::DoubleWord) => base_score * 2,
                    Some(Modifier::TripleWord) => base_score * 3,
                    Some(Modifier::DoubleLetter) => base_score + scrabble_letter_score(letter),
                    Some(Modifier::TripleLetter) => base_score + 2 * scrabble_letter_score(letter)
                };
                WordAroundValidation::ValidWordScore(final_score)
            } else {
                WordAroundValidation::InvalidWord
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    //Board tests
    #[test]
    fn empty_scrabble_board_works() {
        let mut b = ScrabbleBoard::empty_scrabble_board();
        b.set_letter_unchecked(Coord::new(0, 0), b'a');
        println!("Hi");
        b.print_board();
    }

    #[test]
    fn find_words_empty_board() {
        let dict = DictionaryTrie::from_scrabble_ospd();
        let board = ScrabbleBoard::empty_scrabble_board();
        let letters =
            LetterBag::from_string("za");


        let right_solutions = board.find_valid_words_coord(Coord::new(7, 6),
                                                           Direction::Right,
                                                           letters.clone(),
                                                           &dict);

        let down_solutions = board.find_valid_words_coord(Coord::new(7, 6),
                                                          Direction::Right,
                                                          letters.clone(),
                                                          &dict);

        assert_eq!(right_solutions.len(), 1);
        assert_eq!(down_solutions.len(), 1);
        assert_eq!(right_solutions[0].word, String::from("za"));
    }

    #[test]
    fn get_anchored_solutions() {
        let dict = DictionaryTrie::from_scrabble_ospd();
        let mut board = ScrabbleBoard::empty_scrabble_board();
        board.set_letter_unchecked(Coord::new(6, 7), b'g');
        board.set_letter_unchecked(Coord::new(7, 7), b'o');
        board.set_letter_unchecked(Coord::new(8, 7), b'd');

        let letters =
            LetterBag::from_string("d");

        let right_solutions = board.find_valid_words_coord(Coord::new(7, 6),
                                                           Direction::Right,
                                                           letters,
                                                           &dict);

        assert_eq!(right_solutions.len(), 1);
        assert_eq!(right_solutions[0].word, "do");

    }

    #[test]
    fn test_scrabble_letters_score() {
        let score1 =
            scrabble_letters_score_utf8(&Vec::from("za".as_bytes()));
        assert_eq!(score1, 11);
    }
}
