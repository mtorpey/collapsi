use itertools::Itertools;
use std::cmp::Eq;
use std::collections::HashSet;
use std::ops::Add;

const SIZE: usize = 4;

/// A complete description of the current gamestate
pub struct Board {
    /// The values of the cards, with 0 for a flipped card or joker
    cards: [[u8; SIZE]; SIZE],

    /// The coordinates of the two pawns, red and blue
    pawns: [Point; 2],

    /// The number of plies that have been made so far
    turn: usize,

    /// The history of the game so far, as a list of moves
    ///
    /// A move is a tuple (n, p): moved n spaces starting at point p.
    moves: Vec<(u8, Point)>,
}

impl Board {
    /// An example start position
    pub fn new() -> Board {
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

    /// Execute this game, with perfect play on both sides
    ///
    /// This board is mutated to the final position, and messages are printed
    /// along the way.
    pub fn simulate_game(&mut self) {
        loop {
            self.print();
            println!();
            let player = if self.turn == 0 { "R" } else { "B" };
            match self.winning_move() {
                Some(m) => {
                    println!("{} confidently moves to {:?}", player, m);
                    self.make_move(m);
                }
                None => {
                    let moves = self.legal_moves();
                    match moves.into_iter().next() {
                        Some(m) => {
                            println!("{} cannot win, but moves to {:?}", player, m);
                            self.make_move(m);
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

    /// How many possible games could be played out from this position
    pub fn number_of_possible_games(&mut self) -> u64 {
        *self.count_game_tree_leaves(&mut 0)
    }

    /// Iterate through the whole game tree, adding to the counter for each leaf
    ///
    /// This is the recursive function that powers `number_of_possible_games`.
    fn count_game_tree_leaves<'a>(&mut self, counter: &'a mut u64) -> &'a mut u64 {
        let moves = self.legal_moves(); //.into_iter().sorted().collect();
        if moves.is_empty() {
            *counter += 1;
        }
        for m in moves {
            self.make_move(m);
            self.count_game_tree_leaves(counter);
            self.undo_move();
        }
        counter
    }

    /// All possible boards up to symmetry, with their likelihoods
    ///
    /// This creates a set of representative boards such that any possible
    /// starting board is strategically equivalent to exactly one board in the
    /// set. Any board can be transformed into one of these boards via
    /// reflection, rotation, or toroidal cycling of rows and columns.
    ///
    /// Some boards in the set represent more possible boards than others. Each
    /// board is therefore associated with a relative likelihood value.
    pub fn all_boards() -> Vec<(Board, u64)> {
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

    /// The value of the card on the given point
    fn card(&self, point: Point) -> u8 {
        let Point(x, y) = point;
        self.cards[x][y]
    }

    /// A winning move from this board state, or None if the position is losing
    ///
    /// This mutates the board in-place when searching, but should return it to
    /// the current position before returning.
    pub fn winning_move(&mut self) -> Option<Point> {
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

    /// An optimal move in the sense of game-length-perfect play
    ///
    /// Return value is the point to which the current player should move (None
    /// if no possible moves) together with the expected score.
    pub fn best_move_by_cards_remaining(&mut self) -> (Option<Point>, i8) {
        self.best_move_by_cards_remaining_bounded(-16, 16)
    }

    /// An optimal move in the sense of game-length-perfect play, but guided by
    /// "at least" and "at most" values (alpha and beta) to restrict the search.
    ///
    /// This is the recursive function that powers
    /// `best_move_by_cards_remaining`.
    ///
    /// It is an implementation of minimax with alpha-beta pruning.
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

    /// Points reachable from `point` in `dist` squares, assuming we already
    /// moved through everything in `visited`
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

    /// All the possible points the current player could move to this ply
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
        for p in self.pawns {
            moves.remove(&p);
        }

        moves
    }

    /// Modify the board to make the next move to the specified point
    fn make_move(&mut self, point: Point) {
        // Write to history
        let from = self.pawns[self.turn];
        self.moves.push((self.card(from), from));

        // Make the move
        self.set_card(from, 0);
        self.pawns[self.turn] = point;
        self.turn = 1 - self.turn;
    }

    /// Undo the latest move
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

    /// Change the given card to have the specified value
    fn set_card(&mut self, point: Point, dist: u8) {
        let Point(x, y) = point;
        self.cards[x][y] = dist;
    }

    /// Print the board in a readable way
    pub fn print(&self) {
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

/// A single (x,y) coordinate on the board, remembering its torroidal nature
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub struct Point(usize, usize);

impl Add for Point {
    type Output = Self;

    /// Add an offset to a point, wrapping if appropriate
    fn add(self, other: Self) -> Self {
        let Self(x, y) = self;
        let Self(dx, dy) = other;
        Self((x + dx) % SIZE, (y + dy) % SIZE)
    }
}

impl Point {
    /// Neighbours of a point in all four directions, wrapping if appropriate
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

/// All ways to arrange cards with the values in `remaining` given the existing
/// cards in `start`
///
/// - `start` is a list of the card values in the first n positions on the
///   board, with all the other 16-n positions not filled so far.
/// - `remaining`[i] is the number of cards of value i left to be placed, with i
///   from 0 (Joker) to 4.
///
/// Return value is a list of vectors representing possible boards, where each
/// one should be exactly 16 long and represent a complete board.
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
