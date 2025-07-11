// TODO:
// - Tidy up
// - Solve particular position as input
// - Web interface with WASM

use itertools::Itertools;
use rayon::prelude::*;
use simple_tqdm::ParTqdm;
//use simple_tqdm::Tqdm;
use std::cmp::Eq;
use std::collections::HashSet;
use std::env;
use std::ops::Add;

const SIZE: usize = 4;
const USAGE: &str = "Specify one of the following options: --full --solve --solve-length --simulate --all --all-length --all-full";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut board = Board::new();

    if args.len() <= 1 {
        println!("{}", USAGE);
    } else if &args[1] == "--all" {
        let red_wins = Board::all_boards()
            .par_iter_mut()
            .tqdm()
            .map(|(board, weight)| (board.winning_move(), weight))
            .map(|(m, weight)| if m.is_some() { *weight } else { 0 })
            .sum::<u64>();
        println!("R wins {} total", red_wins);
    } else if &args[1] == "--all-length" {
        let scores = Board::all_boards()
            .par_iter_mut()
            .tqdm()
            .map(|(board, weight)| (board.best_move_by_cards_remaining(), board, weight))
            .map(|((m, score), board, weight)| {
                if score.unsigned_abs() > 8 {
                    board.print();
                    println!(
                        "R plays {:?} and gets a score of {}",
                        m.expect("First move should never lose"),
                        score
                    );
                }
                (score, *weight)
            })
            .collect::<Vec<(i8, u64)>>()
            .into_iter()
            .fold([0; 16], |mut results, (score, weight)| {
                results[score.unsigned_abs() as usize] += weight;
                results
            });
        println!("Scores: {:?}", scores);
    } else if &args[1] == "--all-full" {
        let mut boards = Board::all_boards();
        let tree_sizes = boards
            .par_iter_mut()
            .tqdm()
            .map(|(board, weight)| *traverse_game_tree(board, &mut 0) * *weight)
            .sum::<u64>();
        println!("{} games considered in total", tree_sizes);
    } else if &args[1] == "--full" {
        let mut counter = 0;
        traverse_game_tree(&mut board, &mut counter);
        println!("{} games considered", counter);
    } else if &args[1] == "--simulate" {
        simulate_game(&mut board);
    } else if &args[1] == "--solve" {
        match board.winning_move() {
            Some(m) => println!("R wins by playing {:?}", m),
            None => println!("B wins, whatever R plays"),
        };
    } else if &args[1] == "--solve-length" {
        match board.best_move_by_cards_remaining() {
            (Some(m), score) => println!("R plays {:?} and gets a score of {}", m, score),
            _ => eprintln!("Something went wrong"),
        };
    } else {
        println!("{}", USAGE);
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

fn traverse_game_tree<'a>(board: &mut Board, counter: &'a mut u64) -> &'a mut u64 {
    let moves = board.legal_moves(); //.into_iter().sorted().collect();
    if moves.is_empty() {
        *counter += 1;
    }
    for m in moves {
        board.make_move(m);
        traverse_game_tree(board, counter);
        board.undo_move();
    }
    counter
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

    fn all_boards() -> Vec<(Board, u64)> {
        let mut boards = vec![];
        for (pawn2, weight) in [(1, 4), (2, 2), (5, 4), (6, 4), (10, 1)] {
            for perm in unique_permutations(vec![], &[0, 4, 4, 4, 2]) {
                let mut cards = vec![0];
                cards.extend_from_slice(&perm[..pawn2 - 1]);
                cards.push(0);
                cards.extend_from_slice(&perm[pawn2 - 1..]);
                //println!("{:?}", cards);
                boards.push((Board {
                    cards: [
                        cards[0..4].try_into().unwrap(),
                        cards[4..8].try_into().unwrap(),
                        cards[8..12].try_into().unwrap(),
                        cards[12..16].try_into().unwrap(),
                    ],
                    pawns: [Point(0, 0), Point(pawn2 / 4, pawn2 % 4)],
                    turn: 0,
                    moves: vec![],
                }, weight));
            }
        }
        println!("Considering {} boards representing {} deals", boards.len(), boards.iter().map(|(_b, w)| w).sum::<u64>());
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

    fn best_move_by_cards_remaining(&mut self) -> (Option<Point>, i8) {
        self.best_move_by_cards_remaining_bounded(-16, 16)
    }

    fn best_move_by_cards_remaining_bounded(
        &mut self,
        mut at_least: i8,
        mut at_most: i8,
    ) -> (Option<Point>, i8) {
        let moves = self.legal_moves();
        if moves.is_empty() {
            let cards_remaining = 16 - self.moves.len() as i8;
            if cards_remaining % 2 == 1 {
                // P0 wins
                return (None, cards_remaining);
            } else {
                // P1 wins
                return (None, -cards_remaining);
            }
        } else {
            let mut best_score = if self.turn == 0 { -16 } else { 16 }; // worst case
            let mut best_move = Point(0, 0);
            for m in moves {
                self.make_move(m); // note: this flips self.turn
                let (_, score) = self.best_move_by_cards_remaining_bounded(at_least, at_most);
                if self.turn == 1 {
                    // This was P0's turn
                    if score > best_score {
                        best_score = score;
                        best_move = m;
                        if best_score >= at_most {
                            self.undo_move();
                            break;
                        }
                        if best_score > at_least {
                            at_least = best_score;
                        }
                    }
                } else {
                    // This was P1's turn
                    if score < best_score {
                        best_score = score;
                        best_move = m;
                        if best_score <= at_least {
                            self.undo_move();
                            break;
                        }
                        if best_score < at_most {
                            at_most = best_score;
                        }
                    }
                }
                self.undo_move();
            }
            (Some(best_move), best_score)
        }
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

fn unique_permutations(start: Vec<u8>, remaining: &[u8; 5]) -> Vec<Vec<u8>> {
    //println!("{:?}", start);
    let mut out = vec![];
    for value in 1..remaining.len() {
        if remaining[value] > 0 {
            let mut rem_new = remaining.clone();
            rem_new[value] -= 1;
            let mut start_new = start.clone();
            start_new.push(value as u8);
            for perm in unique_permutations(start_new, &rem_new) {
                out.push(perm);
            }
        }
    }
    if out.is_empty() {
        out.push(start);
    }
    out
}
