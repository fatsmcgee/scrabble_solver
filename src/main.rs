extern crate time;

use std::io;
use time::Instant;

use ScrabbleSolver::{DictionaryTrie,
                     ScrabbleBoard,
                     Coord,
                     Direction,
                     LetterBag,
                     print_top_solutions};


fn main() {

    let dict = DictionaryTrie::from_scrabble_ospd();

    let mut boards = Vec::new();
    boards.push(ScrabbleBoard::empty_scrabble_board());

    loop {
        let mut command = String::new();
        io::stdin().read_line(&mut command);
        let mut parts = command.split_ascii_whitespace();
        match parts.next().unwrap_or("") {
            "help" => {
                println!("print #prints board");
                println!("undo #undoes last letter placement");
                println!("top boardLetters [n] #find top words right/down for row/col with boardLetters, optionally limiting to top n");
                println!("place (r,d) row col boardLetters #place boardLetters on board");

            },
            "print" => {
                boards.last().unwrap().print_board();
            },
            "undo" => {
                if boards.len() > 1 {
                    boards.pop();
                }
            },
            "place" => {
                let dir = parts.next().and_then( |d| match d {
                    "r" | "R" => Some(Direction::Right),
                    "d" | "D" => Some(Direction::Down),
                    _ => None
                });
                let row = parts.next().and_then(|r| r.parse::<i32>().ok());
                let col = parts.next().and_then(|c| c.parse::<i32>().ok());
                let letters = parts.next();


                let coord =
                    row.and_then(|r| col.map(|c| Coord::new(r,c)));

                let board = boards.last().unwrap();

                match (dir, coord, letters) {
                    (Some(dir), Some(coord), Some(letters)) => {
                        let mut new_board = board.clone();
                        new_board.add_word(coord, dir,letters);
                        boards.push(new_board);
                    },
                    _ => { println!("Invalid place command"); }
                }

            },
            "top" => {
                let letters = parts.next();
                let n = parts.next().and_then(|p| p.parse::<usize>().ok());
                match (letters, n) {
                    (Some(letters), n) => {
                        let letters =
                            LetterBag::from_string(letters);
                        let board = boards.last().unwrap();
                        let solutions = board.find_all_valid_words(&letters,&dict);
                        print_top_solutions(&solutions,n);
                    },
                    _ => { println!("Invalid place command"); }
                }
            },
            s => {
                println!("Unknown command: {}", s);
            }
        }
    }
}