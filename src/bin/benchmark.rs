extern crate time;
use time::Instant;

use ScrabbleSolver::{DictionaryTrie,
                     ScrabbleBoard,
                     Coord,
                     Direction,
                     letterbag_from_string,
                     print_top_solutions};

fn main() {
    let dict = DictionaryTrie::from_scrabble_ospd();
    let mut board = ScrabbleBoard::empty_scrabble_board();
    board.add_word(Coord::new(7,5), Direction::Right, "lolcatz");
    board.add_word(Coord::new(6,6), Direction::Down, "goalie");


    for i in 1..5 {
        let now = Instant::now();
        let solutions = board.find_all_valid_words(&letterbag_from_string("*fsaebd"), &dict);
        let duration = (Instant::now() - now).as_seconds_f32();
        println!("Took {} seconds", duration);
    }
}