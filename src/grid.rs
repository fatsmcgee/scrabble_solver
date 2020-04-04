use super::util::Direction;
use serde::Serialize;

#[derive(Clone, Copy, Serialize, PartialEq)]
pub struct Coord {
    pub row: i32,
    pub col: i32,
}

impl Coord {
    pub fn new(row: i32, col: i32) -> Coord {
        Coord { row, col }
    }

    pub fn next(&self, d: Direction) -> Coord {
        match d {
            Direction::Right => Coord { row: self.row, col: self.col + 1 },
            Direction::Down => Coord { row: self.row + 1, col: self.col },
        }
    }

    pub fn prev(&self, d: Direction) -> Coord {
        match d {
            Direction::Right => Coord { row: self.row, col: self.col - 1 },
            Direction::Down => Coord { row: self.row - 1, col: self.col },
        }
    }
}

#[derive(Clone)]
pub struct Grid<T> {
    nrows: usize,
    ncols: usize,
    storage: Vec<T>,
}

impl<T> Grid<T> {
    pub fn is_coord_in_bounds(&self, coord: Coord) -> bool {
        if coord.row < 0 || coord.col < 0 {
            false
        } else if (coord.row as usize) >= self.nrows || (coord.col as usize) >= self.ncols {
            false
        } else {
            true
        }
    }

    pub fn new(nrows: usize, ncols: usize, default: T) -> Grid<T> where T: Clone {
        let storage = vec![default; nrows * ncols];
        Grid { nrows, ncols, storage }
    }

    pub fn from(rows: Vec<Vec<T>>) -> Grid<T> where T: Clone + std::fmt::Debug {
        let mut storage = Vec::new();
        let nrows = rows.len();
        let mut ncols = None;
        for row in rows.iter() {
            ncols = match (ncols, row.len()) {
                (None, i) => Some(i),
                (Some(i), j) => if i == j { Some(i) } else {
                    panic!("One row has {} columns and another has {}",
                           i, j)
                }
            };
            storage.extend(row.clone());
        }

        if let Some(ncols) = ncols {
            Grid { nrows, ncols, storage }
        } else {
            panic!("No rows")
        }
    }

    fn offset(&self, coord: Coord) -> usize {
        (coord.row as usize) * self.ncols + (coord.col as usize)
    }

    pub fn get(&self, coord: Coord) -> Option<T> where T: Clone {
        let offset = self.offset(coord);
        self.storage.get(offset).cloned()
    }

    pub fn get_unchecked(&self, coord: Coord) -> T where T: Clone {
        let offset = self.offset(coord);
        self.storage[offset].clone()
    }

    pub fn set_unchecked(&mut self, coord: Coord, val: T) {
        let offset = self.offset(coord);
        self.storage[offset] = val;
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }
}