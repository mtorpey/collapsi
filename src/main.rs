use itertools::Itertools;
use std::cmp::Eq;
use std::collections::HashSet;
use std::env;
use std::ops::Add;

const SIZE: usize = 4;

fn main() {
    let args: Vec<String> = env::args().collect();

    for board in Board::all_boards() {
        println!(".");
    }
    return;

    let mut board = Board::new();

    if args.len() <= 1 {
        println!("Options: --solve --simulate --full");
    } else if &args[1] == "--full" {
        let mut counter = 0;
        traverse_game_tree(&mut board, &mut counter);
        println!("{} games tried", counter);
    } else if &args[1] == "--simulate" {
        simulate_game(&mut board);
    } else if &args[1] == "--solve" {
        match board.winning_move() {
            Some(m) => println!("R wins by playing {:?}", m),
            None => println!("B wins, whatever R plays"),
        };
    } else {
        println!("Options: --solve --simulate --full");
    }
}

fn simulate_game(board: &mut Board) {
    loop {
        board.print();
        println!();
        let player = if board.turn == 0 { "R" } else { "B" };
        match board.winning_move() {
            Some(m) => {
                println!("{} confidently moves to {:?}", player, m);
                board.make_move(m);
            }
            None => {
                let moves = board.legal_moves();
                match moves.into_iter().next() {
                    Some(m) => {
                        println!("{} cannot win, but moves to {:?}", player, m);
                        board.make_move(m);
                    }
                    None => {
                        println!("{} loses", player);
                        break;
                    }
                }
            }
        }
    }
}

fn traverse_game_tree(board: &mut Board, counter: &mut u64) {
    let moves = board.legal_moves(); //.into_iter().sorted().collect();
    if moves.is_empty() {
        *counter += 1;
        if *counter % 100000 == 0 {
            println!("{}", counter);
        }
    }
    for m in moves {
        board.make_move(m);
        traverse_game_tree(board, counter);
        board.undo_move();
    }
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
            cards: [
                [1, 2, 2, 3],
                [4, 1, 2, 0], // should be 0 last
                [3, 1, 2, 3],
                [0, 3, 1, 4],
            ],
            pawns: [Point(1, 3), Point(3, 0)],
            turn: 0,
            moves: vec![],
        }
    }

    fn all_boards() -> Vec<Board> {
        let mut boards = vec![];
        for pawn2 in [1, 2, 3, 5, 6, 7, 10, 11] {
            let mut remaining = [0, 4, 4, 4, 2];
            for perm in unique_permutations(&mut remaining) {
                let mut cards = vec![0];
                cards.extend_from_slice(&perm[.. pawn2 - 1]);
                cards.push(0);
                cards.extend_from_slice(&perm[pawn2 - 1 ..]);
                println!("{:?}", cards);
            }
            //for fours in (1..=15).filter(|i| *i != pawn2).permutations(2) {
                //let cards = [5; 16];
                //cards[0] = 0;
                //cards[pawn2] = 0;
                //cards[fours[0]] = 4;
                //cards[fours[1]] = 4;
                //let board = Board{
                //    cards: cards.chunks(4),
                //    pawns: [Point(0, 0), pawn2],
                //    turn: 0,
                //    moves: vec![],
                //};
                //boards.push(board);
            //}
        }
        boards
    }

    fn card(&self, point: Point) -> u8 {
        let Point(x, y) = point;
        self.cards[x][y]
    }

    fn winning_move(&mut self) -> Option<Point> {
        for m in self.legal_moves() {
            self.make_move(m);
            if self.winning_move().is_none() {
                self.undo_move();
                return Some(m);
            }
            self.undo_move();
        }
        None
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
                    .filter(|p| self.card(*p) != 0 && !self.pawns.contains(p)),
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
        let (dist, from) = self
            .moves
            .pop()
            .expect("We should never undo a fresh board");

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
                let marker = if self.pawns[0] == Point(row, col) {
                    "R"
                } else if self.pawns[1] == Point(row, col) {
                    "B"
                } else {
                    " "
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
        const DIRECTIONS: [Point; 4] = [
            Point(1, 0),
            Point(SIZE - 1, 0),
            Point(0, 1),
            Point(0, SIZE - 1),
        ];
        DIRECTIONS.map(|d| *self + d)
    }
}

fn unique_permutations(remaining: &mut [usize; 5]) -> Vec<&mut Vec<usize>> {
    let mut out = vec![];
    for value in 1..remaining.len() {
        if remaining[value] > 0 {
            remaining[value] -= 1;
            for perm in unique_permutations(remaining) {
                perm.push(value);
                out.push(perm);
            }
            remaining[value] += 1;
        }
    }
    out
}
