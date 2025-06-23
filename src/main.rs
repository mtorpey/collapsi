use std::ops::Add;
use std::cmp::Eq;
use std::collections::HashSet;
use itertools::Itertools;

const SIZE: usize = 4;



fn main() {
    let mut board = Board::new();
    board.print();

    let mut counter = 0;
    explore_game_tree(&mut board, &mut counter);

    println!("{} games tried", counter);
}

fn explore_game_tree(board: &mut Board, counter: &mut u64) {
    let moves = board.legal_moves();//.into_iter().sorted().collect();
    if moves.is_empty() {
        *counter += 1;
        if *counter % 100000 == 0 {
            println!("{}", counter);
        }
    }
    for m in moves {
        //println!("{:?}", m);
        board.make_move(m);
        explore_game_tree(board, counter);
        board.undo_move();
    }
    //println!("{:?}", moves);
}



struct Board {
    cards: [[u8; SIZE]; SIZE],
    pawns: [Point; 2],
    turn: usize,
    moves: Vec<(u8, Point)>, // move n spaces from point p
}

impl Board {
    fn new() -> Board {
        Board {
            cards: [[1, 2, 2, 3],
                    [4, 1, 2, 0], // should be 0 last
                    [3, 1, 2, 3],
                    [0, 3, 1, 4]],
            pawns: [Point(1, 3), Point(3, 0)],
            turn: 0,
            moves: vec![],
        }
    }

    fn card(&self, point: Point) -> u8 {
        let Point(x, y) = point;
        self.cards[x][y]
    }

    fn reachable(&self, point: Point, dist: u8, visited: &mut Vec<Point>) -> HashSet<Point> {
        if visited.contains(&point) || self.card(point) == 0 {
            return HashSet::new();
        } else if dist == 0 {
            return HashSet::from([point]);
        }
        let mut out = HashSet::new();
        for neighbor in point.neighbors() {
            visited.push(point);
            out.extend(self.reachable(neighbor, dist - 1, visited));
            visited.pop();
        }
        out
    }

    fn legal_moves(&self) -> HashSet<Point> {
        let origin: Point = self.pawns[self.turn];
        let dist = self.card(origin);

        // Start of game: can move anywhere
        if dist == 0 {
            // all points except starting spaces
            return HashSet::from_iter(
                (0..SIZE)
                    .cartesian_product(0..SIZE)
                    .map(|(x, y)| Point(x, y))
                    .filter(|p| self.card(*p) != 0 && !self.pawns.contains(p))
            );
        }

        // Otherwise, depth-first search
        let mut moves = self.reachable(origin, dist, &mut vec![]);

        // Cannot move onto the opponent's piece
        self.pawns.map(|p| moves.remove(&p));

        moves
    }

    fn make_move(&mut self, point: Point) {
        // Write to history
        let from = self.pawns[self.turn];
        self.moves.push((self.card(from), from));

        // Make the move
        self.set_card(from, 0);
        self.pawns[self.turn] = point;
        self.turn = 1 - self.turn;
    }

    fn undo_move(&mut self) {
        // Get history
        let (dist, from) = self.moves.pop().expect("We should never undo a fresh board");

        // Undo move
        self.turn = 1 - self.turn;
        self.pawns[self.turn] = from;
        self.set_card(from, dist);
    }

    fn set_card(&mut self, point: Point, dist: u8) {
        let Point(x, y) = point;
        self.cards[x][y] = dist;
    }

    fn print(&self) {
        for row in 0..SIZE {
            for col in 0..SIZE {
                let marker =
                    if self.pawns[0] == Point(row, col) { "r"
                    } else if self.pawns[1] == Point(row, col) { "b"
                    } else { " "
                    };
                print!("{}{}{}", marker, self.cards[row][col], marker);
            }
            println!();
        }
    }
}



#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, PartialEq, Eq)]
struct Point(usize, usize);

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let Self(x, y) = self;
        let Self(dx, dy) = other;
        Self((x + dx) % SIZE, (y + dy) % SIZE)
    }
}

impl Point {
    fn neighbors(&self) -> [Point; 4] {
        const DIRECTIONS: [Point; 4] = [Point(1, 0), Point(SIZE-1, 0), Point(0, 1), Point(0, SIZE-1)];
        DIRECTIONS.map(|d| *self + d)
    }
}
