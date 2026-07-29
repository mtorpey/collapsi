// TODO:
// - Solve particular position as input
// - Web interface with WASM

use rayon::prelude::*;
use simple_tqdm::ParTqdm;
//use simple_tqdm::Tqdm;
use std::env;

use collapsi::Board;
use collapsi::CollapsiVersion;

const USAGE: &str = "Specify one of the following options: --simulate --solve --full --solve-all --full-all";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut board = Board::new();

    // All solutions are using length-perfect play.
    // That is, they call best_move_by_cards_remaining instead of winning_move.

    if args.len() <= 1 {
        println!("{}", USAGE);
    } else if &args[1] == "--full" {
        println!("{} positions in game tree", board.number_of_possible_games());
    } else if &args[1] == "--simulate" {
        board.simulate_game();
    } else if &args[1] == "--solve" {
        match board.best_move_by_cards_remaining() {
            (Some(m), score) => println!("R plays {:?} and gets a score of {}", m, score),
            _ => eprintln!("Something went wrong"),
        };
    } else if &args[1] == "--solve-all" {
        let scores = Board::all_boards(CollapsiVersion::V1_3)
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
    } else if &args[1] == "--full-all" {
        let mut boards = Board::all_boards(CollapsiVersion::V1_3);
        let tree_sizes = boards
            .par_iter_mut()
            .tqdm()
            .map(|(board, weight)| board.number_of_possible_games() * *weight)
            .sum::<u64>();
        println!("{} game sequences considered in total", tree_sizes);
    } else {
        println!("{}", USAGE);
    }
}
