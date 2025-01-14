use std::fmt;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use Direction::{East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West};

use crate::board::Disk::{Dark, Light};
use crate::errors::Error;
use crate::errors::Error::{InvalidArgument, ParseError};

pub const BOARD_SIZE: usize = 8;

pub const DARK_CHAR: char = 'D';
pub const LIGHT_CHAR: char = 'L';
pub const EMPTY_CHAR: char = 'E';

const POSITION_WEIGHTS: [[i32; BOARD_SIZE]; BOARD_SIZE] = [
    [100, -10,  30,  20,  20,  30, -10, 100],
    [-10, -10,   1,   2,   2,   1, -10, -10],
    [ 30,   1,  10,   6,   6,  10,   1,  30],
    [ 20,   2,   6,   0,   0,   6,   2,  20],
    [ 20,   2,   6,   0,   0,   6,   2,  20],
    [ 30,   1,  10,   6,   6,  10,   1,  30],
    [-10, -10,   1,   2,   2,   1, -10, -10],
    [100, -10,  30,  20,  20,  30, -10, 100]
];

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    
    /// Returns the iterator for all possible directions
    pub fn all() -> impl Iterator<Item=Direction> {
       vec![North, NorthEast, East, SouthEast, South, SouthWest, West, NorthWest].into_iter()
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Disk {
    #[default]
    Dark,
    Light,
}

impl Display for Disk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match *self {
            Dark => DARK_CHAR,
            Light => LIGHT_CHAR,
        })
    }
}

impl Disk {
    
    /// Parses the given character into a disk
    pub fn parse(ch: char) -> Result<Self, Error> {
        match ch {
            DARK_CHAR => Ok(Dark),
            LIGHT_CHAR => Ok(Light),
            _ => Err(ParseError(format!("Invalid character to parse into a disk: {}", ch))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Position {
    row: usize,
    col: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {row: 0, col: 0}
    }
}

impl Position {
    
    /// Parses the given string into a position
    pub fn parse(s: String) -> Result<Self, Error> {
        if let [row, col] = s.split(",")
            .map(|s| s.parse::<usize>())
            .filter(|result| result.is_ok())
            .map(|result| result.unwrap())
            .collect_vec()[..] {
            Ok(Self {
                row,
                col,
            })
        } else {
            Err(ParseError(format!("Invalid string to parse into a position: {}", s)))
        }
    }
    
    /// Creates a new Position
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
        }
    }
    
    /// Checks if this position is in bound
    fn is_inbound(&self) -> bool {
        self.row < BOARD_SIZE && self.col < BOARD_SIZE
    }
    
    /// Returns the direction towards the target
    /// Pre-conditions:
    /// * self != target
    #[cfg(test)]
    pub fn direction(&self, target: &Position) -> Direction {
        assert_ne!(self, target);
        
        let row_diff = target.row as i32 - self.row as i32;
        let hor_diff = target.col as i32 - self.col as i32;
        
        if row_diff < 0 {
            if hor_diff < 0 {
                NorthWest
            } else if hor_diff == 0 {
                North
            } else {
                NorthEast
            }
        } else if row_diff == 0 {
            if hor_diff < 0 {
                West
            } else {
                East
            }
        } else {
            if hor_diff < 0 {
                SouthWest
            } else if hor_diff == 0 {
                South
            } else {
                SouthEast
            } 
        }
    }
    
    /// Returns the weight of this position
    pub fn weight(&self) -> i32 {
        POSITION_WEIGHTS[self.row][self.col]
    }

    /// Returns all possible positions of the board
    pub fn all() -> impl Iterator<Item=Position> {
        let mut positions = vec![];
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                positions.push(Position::new(i, j));
            }
        }

        positions.into_iter()
    }
}

impl Display for Position {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.row, self.col)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    grid: [[Option<Disk>; BOARD_SIZE]; BOARD_SIZE],
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(BOARD_SIZE * BOARD_SIZE + BOARD_SIZE);
        
        for row in self.grid.iter() {
            for cell in row.iter() {
                buf.push(match cell {
                    None => EMPTY_CHAR,
                    Some(disk) => disk.to_string().chars().nth(0).unwrap(),
                })
            }
            
            buf.push('\n');
        }
        
        write!(f, "{}", buf.trim())
    }
}

impl Board {
    
    /// Creates a new board
    pub fn new() -> Self {
        assert_eq!(BOARD_SIZE % 2, 0, "Board size must be even");
        
        let mut board = Board {
            grid: [[None; BOARD_SIZE]; BOARD_SIZE]
        };
        
        let mid_pos = Position::new(BOARD_SIZE / 2 - 1, BOARD_SIZE / 2 - 1);
        
        board.grid[mid_pos.row][mid_pos.col] = Some(Dark);
        board.grid[mid_pos.row + 1][mid_pos.col] = Some(Light);
        board.grid[mid_pos.row][mid_pos.col + 1] = Some(Light);
        board.grid[mid_pos.row + 1][mid_pos.col + 1] = Some(Dark);

        board
    }
    
    /// Parses the given data to a board
    pub fn parse(data: String) -> Result<Self, Error> {
        let mut board = Board::new();
        for (i, line) in data.lines().enumerate() {
            for (j, ch) in line.chars().enumerate() {
                let disk = if ch == EMPTY_CHAR {
                    None
                } else {
                    Some(Disk::parse(ch)?)
                };

                board.grid[i][j] = disk;
            }
        }
        Ok(board)
    }
    
    /// Returns the disk at the given position
    pub fn disk(&self, pos: &Position) -> Option<Disk> {
        self.grid[pos.row][pos.col]
    }
    
    /// Places the disk at the given position
    /// Pre-conditions:
    /// * Given position isn't occupied by a disk
    pub fn place(&mut self, disk: Disk, pos: &Position) -> Result<(), Error> {
        if self.disk(pos).is_some() {
            return Err(InvalidArgument(
                format!("Given position is not empty to place a disk: {}", pos)));
        }
        
        self.grid[pos.row][pos.col] = Some(disk);
        Ok(())
    }
    
    /// Returns all positions of the given disk
    pub fn positions(&self, disk: Disk) -> impl Iterator<Item=Position> {
        self.grid.into_iter()
            .flatten()
            .enumerate()
            .filter(move |(_, d)| d.is_some() && d.unwrap() == disk)
            .map(|(i, _)| Position::new(i / BOARD_SIZE, i % BOARD_SIZE))
    }
    
    /// Flips the disk at the given position
    /// Pre-conditions:
    /// * pos.is_inbound()
    /// * The given position must be occupied by a disk
    pub fn flip(&mut self, pos: &Position) -> Result<(), Error> {
        assert!(pos.is_inbound());

        match self.disk(pos) {
            None => Err(InvalidArgument(format!("Board is empty at {}", pos))),
            Some(disk) => { 
                self.grid[pos.row][pos.col] = Some(if disk == Dark { Light } else { Dark });
                Ok(())
            }
        }
    }
    
    /// Returns the neighbours of the given position
    /// Pre-conditions:
    /// * pos.is_inbound()
    #[cfg(test)]
    pub fn neighbours(&self, pos: &Position) -> impl Iterator<Item=Position> {
        assert!(pos.is_inbound());
        
        let mut neighbours = Vec::with_capacity(9);
        
        let range: [i32; 3] = [-1, 0, 1];
        for i in range {
            for j in range {
                neighbours.push(Position::new((i + pos.row as i32) as usize,
                                              (j + pos.col as i32) as usize));
            }
        }
        
        let pos = pos.clone();
        neighbours.into_iter()
            .filter(move |neighbour| *neighbour != pos)
            .filter(|p| p.is_inbound())
    }
    
    /// Returns the neighbour from the given position at the given direction
    /// Pre-conditions:
    /// * pos.is_inbound()
    pub fn neighbour(&self, pos: &Position, dir: Direction) -> Option<Position> {
        assert!(pos.is_inbound());
        
        let offset = match dir {
            North => (-1, 0),
            NorthEast => (-1, 1),
            East => (0, 1),
            SouthEast => (1, 1),
            South => (1, 0),
            SouthWest => (1, -1),
            West => (0, -1),
            NorthWest => (-1, -1),
        };
        
        let neighbour = Position::new((pos.row as i32 + offset.0) as usize, 
                      (pos.col as i32 + offset.1) as usize);
        
        if neighbour.is_inbound() {Some(neighbour)} else {None}
    }

    #[cfg(test)]
    /// Clears this board
    pub fn clear(&mut self) {
        self.grid = [[None; BOARD_SIZE]; BOARD_SIZE];
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, BOARD_SIZE, Direction, Disk, Position};
    use crate::board::Direction::{East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West};
    use crate::board::Disk::{Dark, Light};

    #[test]
    fn new() {
        let board = Board::new();

        // This test is only correct when BOARD_SIZE == 8
        assert_eq!(BOARD_SIZE, 8);

        assert_eq!(board.grid[3][3], Some(Dark));
        assert_eq!(board.grid[4][4], Some(Dark));
        assert_eq!(board.grid[3][4], Some(Light));
        assert_eq!(board.grid[4][3], Some(Light));
        
        assert_eq!(board.to_string(),
        "\
        EEEEEEEE\n\
        EEEEEEEE\n\
        EEEEEEEE\n\
        EEEDLEEE\n\
        EEELDEEE\n\
        EEEEEEEE\n\
        EEEEEEEE\n\
        EEEEEEEE"
        )
    }
    
    #[test]
    fn disk() { 
        let mut board = Board::new();
        
        let pos = Position::new(0, 0);
        assert!(board.disk(&pos).is_none());
        
        board.grid[pos.row][pos.col] = Some(Dark);
        assert_eq!(board.disk(&pos), Some(Dark));
    }
    
    #[test]
    fn flip() {
        let mut board = Board::new();

        let pos = Position::new(0, 0);
        assert!(board.flip(&pos).is_err());

        board.grid[0][0] = Some(Dark);
        assert!(board.flip(&pos).is_ok());
        assert_eq!(board.disk(&pos), Some(Light));

        assert!(board.flip(&pos).is_ok());
        assert_eq!(board.disk(&pos), Some(Dark));
    }
    
    #[test]
    fn neighbours() {
        let board = Board::new();

        let get_result = |pos: &Position| -> Vec<String> {
            board.neighbours(&pos)
                .map(|pos| pos.to_string())
                .collect()
        };
        
        let pos = Position::new(0, 0);
        assert_eq!(get_result(&pos), vec!["0,1", "1,0", "1,1"]);
        
        let pos = Position::new(0, BOARD_SIZE - 1);
        assert_eq!(get_result(&pos), vec!["0,6", "1,6", "1,7"]);
        
        let pos = Position::new(BOARD_SIZE - 1, 0);
        assert_eq!(get_result(&pos), vec!["6,0", "6,1", "7,1"]);

        let pos = Position::new(BOARD_SIZE - 1, BOARD_SIZE - 1);
        assert_eq!(get_result(&pos), vec!["6,6", "6,7", "7,6"]);

        
        let pos = Position::new(0, 3);
        assert_eq!(get_result(&pos), vec!["0,2", "0,4", "1,2", "1,3", "1,4"]);

        let pos = Position::new(3, BOARD_SIZE - 1);
        assert_eq!(get_result(&pos), vec!["2,6", "2,7", "3,6", "4,6", "4,7"]);

        let pos = Position::new(BOARD_SIZE - 1, 3);
        assert_eq!(get_result(&pos), vec!["6,2", "6,3", "6,4", "7,2", "7,4"]);

        let pos = Position::new(3, 0);
        assert_eq!(get_result(&pos), vec!["2,0", "2,1", "3,1", "4,0", "4,1"]);

        
        let pos = Position::new(3, 3);
        assert_eq!(get_result(&pos), vec!["2,2", "2,3", "2,4", "3,2", "3,4", "4,2", "4,3", "4,4"]);
    }
    
    #[test]
    fn neighbour() {
        let board = Board::new();
        
        let get_result = |pos: &Position, dir: Direction| -> Option<String> {
            board.neighbour(pos, dir)
                .map(|neighbour| neighbour.to_string())
        };
        
        let pos = Position::new(3, 3);
        assert_eq!(get_result(&pos, North), Some("2,3".to_string()));
        assert_eq!(get_result(&pos, NorthEast), Some("2,4".to_string()));
        assert_eq!(get_result(&pos, East), Some("3,4".to_string()));
        assert_eq!(get_result(&pos, SouthEast), Some("4,4".to_string()));
        assert_eq!(get_result(&pos, South), Some("4,3".to_string()));
        assert_eq!(get_result(&pos, SouthWest), Some("4,2".to_string()));
        assert_eq!(get_result(&pos, West), Some("3,2".to_string()));
        assert_eq!(get_result(&pos, NorthWest), Some("2,2".to_string()));

        let pos = Position::new(0, 0);
        assert_eq!(get_result(&pos, North), None);
        assert_eq!(get_result(&pos, NorthEast), None);
        assert_eq!(get_result(&pos, SouthWest), None);
        assert_eq!(get_result(&pos, West), None);
        assert_eq!(get_result(&pos, NorthWest), None);
        
        let pos = Position::new(BOARD_SIZE - 1, BOARD_SIZE - 1);
        assert_eq!(get_result(&pos, NorthEast), None);
        assert_eq!(get_result(&pos, East), None);
        assert_eq!(get_result(&pos, SouthEast), None);
        assert_eq!(get_result(&pos, South), None);
        assert_eq!(get_result(&pos, SouthWest), None);
    }
    
    #[test]
    fn positions() {
        let mut board = Board::new();
        board.clear();

        board.grid[0][0] = Some(Dark);
        board.grid[1][1] = Some(Dark);
        board.grid[2][2] = Some(Light);

        let get_result = |player: Disk| -> Vec<String> {
            board.positions(player)
                .map(|pos| pos.to_string())
                .collect::<Vec<String>>()
        };
        
        assert_eq!(get_result(Dark), vec!["0,0", "1,1"]);
    }
    
    #[test]
    fn direction() {
        let center = Position::new(BOARD_SIZE / 2, BOARD_SIZE / 2);
        
        let target = Position::new(center.row - 2, center.col - 2);
        assert_eq!(center.direction(&target), NorthWest);
        
        let target = Position::new(center.row, center.col - 2);
        assert_eq!(center.direction(&target), West);

        let target = Position::new(center.row + 2, center.col - 2);
        assert_eq!(center.direction(&target), SouthWest);

        let target = Position::new(center.row + 2, center.col);
        assert_eq!(center.direction(&target), South);

        let target = Position::new(center.row + 2, center.col + 2);
        assert_eq!(center.direction(&target), SouthEast);

        let target = Position::new(center.row, center.col + 2);
        assert_eq!(center.direction(&target), East);

        let target = Position::new(center.row - 2, center.col + 2);
        assert_eq!(center.direction(&target), NorthEast);

        let target = Position::new(center.row - 2, center.col);
        assert_eq!(center.direction(&target), North);
    }
}
