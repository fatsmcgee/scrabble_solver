use serde::Serialize;

pub type Letter = u8;
pub type Word = String;

#[derive(Clone, Copy, Serialize, PartialEq)]
pub enum Direction {
    Right,
    Down,
}

impl Direction {
    pub fn rotate(&self) -> Direction {
        match self {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Right
        }
    }
}