// TODO:
// - Solve particular position as input
// - Web interface with WASM

use rayon::prelude::*;
use simple_tqdm::ParTqdm;
//use simple_tqdm::Tqdm;
use std::env;

use collapsi::Board;

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
            .map(|(board, weight)| board.number_of_possible_games() * *weight)
            .sum::<u64>();
        println!("{} positions considered in total", tree_sizes);
    } else if &args[1] == "--full" {
        println!("{} positions in game tree", board.number_of_possible_games());
    } else if &args[1] == "--simulate" {
        board.simulate_game();
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
