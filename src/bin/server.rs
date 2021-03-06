#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]

#[macro_use] extern crate rocket;
extern crate regex;


use ScrabbleSolver::{DictionaryTrie, ScrabbleBoard, Coord, Direction, LetterBag, print_top_solutions, ScrabbleSolution};
use rocket::State;
use rocket::http::Method;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use regex::Regex;
use serde::Serialize;
use std::fmt::Display;
use std::error::Error;
use std::ops::Try;


//http://localhost:8000/is_word?word=dog
#[get("/is_word?<word>")]
fn is_word(dict_trie: State<DictionaryTrie>, word:String) -> String {
    format!("{}", dict_trie.inner().is_word_string(&word))
}


///
/// board_spec: String specifying placedLetters placed on a scrabble board
/// with multiple comma separated <row>,<col>,<letter> pieces separated by semi-colon ';' e.g
/// 3,4,a;4,5,z;

fn boardspec_to_board(board_spec:&str) -> Result<ScrabbleBoard,String> {

    if(board_spec == "") {
        return Result::Ok(ScrabbleBoard::empty_scrabble_board());
    }

    let mut board = ScrabbleBoard::empty_scrabble_board();
    let row_col_letter_re = Regex::new(r"^(\d+),(\d+),([a-zA-Z])$").unwrap();

    for spec_part in board_spec.split(";") {
        if let Some(captures) = row_col_letter_re.captures(spec_part) {

            let row = captures.get(1)
                .and_then(|c| c.as_str().parse::<i32>().ok())
                .unwrap();

            let col = captures.get(2)
                .and_then(|c| c.as_str().parse::<i32>().ok())
                .unwrap();

            let letter = captures.get(3).unwrap().as_str().as_bytes()[0];

            let coord = Coord::new(row,col);
            board.set_letter_unchecked(coord, letter);
        } else {
            return Result::Err(format!("{} is not in row,col,letter format", spec_part));
        }
    }


    Result::Ok(board)
}

#[derive(Serialize)]
struct SolutionsResponse {
    error:Option<String>,
    solutions:Vec<ScrabbleSolution>
}

//http://localhost:8000/solutions?board_letters=s&board_spec=7,5,d;7,6,o;7,7,g
#[get("/solutions?<board_letters>&<board_spec>")]
fn solutions(dict_trie: State<DictionaryTrie>,
             board_letters:String,
             board_spec:String) -> Json<SolutionsResponse> {
    let letter_bag = LetterBag::from_string(&board_letters);
    let board = boardspec_to_board(&board_spec);
    match board {
        Ok(board) => {
            let mut solutions = board.find_all_valid_words(&letter_bag,
                                                           dict_trie.inner());
            solutions.sort_by_key(|s| -(s.score as i32));

            Json(SolutionsResponse {error:None, solutions:solutions})
        },
        Err(err_msg) => {
            Json(SolutionsResponse {error:Some(err_msg), solutions:Vec::new()})
        }
    }
}

fn main() {
    rocket::ignite()
        .manage(DictionaryTrie::from_scrabble_ospd())
        .mount("/", routes![is_word, solutions])
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/frontend")))
        .launch();
}