// TODO: web interface with WASM

use rayon::prelude::*;
use simple_tqdm::ParTqdm;
//use simple_tqdm::Tqdm;
use std::env;

use collapsi::Board;
use collapsi::CollapsiVersion;

const USAGE: &str = "Usage: collapsi command board
where command is one of:
  solve     (compute a perfect-play move)
  simulate  (run the full game with perfect play, showing all moves)
  full      (explore the full game tree and count the leaves)
and board is either the word 'all' or 'all_old' or a string of the form:
  1223/4121r/3123/1b314/0
where:
  - the first four groups of characters represent the four rows
  - a number indicates the possible movement from that space
  - 0 indicates a face-down card (or a joker in the old rules)
  - r or b indicates that the previous space contains a red/blue pawn
  - the final number is the number of turns taken so far
  - 'all' will instead run the operation for all boards and report a summary
  - 'all_old' behaves like 'all' but using old rules (Collapsi v1.1)";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("{}", USAGE);
        return;
    }
    let command: &str = &args[1];
    let board: &str = &args[2];

    if board == "all" || board == "all_old" {
        let version = if board == "all_old" {
            CollapsiVersion::V1_1
        } else {
            CollapsiVersion::V1_3
        };
        match command {
            "solve" => run_solve_all(version),
            "full" => run_full_all(version),
            "simulate" => println!("simulate cannot be run over all boards"),
            _ => println!("invalid command"),
        }
    } else {
        let mut board: Board = match Board::new(&board) {
            Ok(board) => board,
            Err(message) => {
                println!("Invalid board: {}", message);
                return;
            }
        };
        println!("{}", board);
        match command {
            "solve" => run_solve(&mut board),
            "full" => run_full(&mut board),
            "simulate" => run_simulate(&mut board),
            _ => println!("invalid command"),
        }
    }
}

// All solutions are using length-perfect play.
// That is, they call best_move_by_cards_remaining instead of winning_move.

fn run_solve_all(version: CollapsiVersion) {
    let scores = Board::all_boards(version)
        .par_iter_mut()
        .tqdm()
        .map(|(board, weight)| (board.best_move_by_cards_remaining(), board, weight))
        .map(|((m, score), board, weight)| {
            if score.unsigned_abs() > 8 {
                println!("{}", board);
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
}

fn run_full_all(version: CollapsiVersion) {
    let mut boards = Board::all_boards(version);
    let tree_sizes = boards
        .par_iter_mut()
        .tqdm()
        .map(|(board, weight)| board.number_of_possible_games() * *weight)
        .sum::<u64>();
    println!("{} game sequences considered in total", tree_sizes);
}

fn run_solve(board: &mut Board) {
    match board.best_move_by_cards_remaining() {
        (Some(m), score) => println!("R plays {:?} and gets a score of {}", m, score),
        _ => eprintln!("Something went wrong"),
    };
}
fn run_full(board: &mut Board) {
    println!(
        "{} positions in game tree",
        board.number_of_possible_games()
    );
}
fn run_simulate(board: &mut Board) {
    board.simulate_game();
}
