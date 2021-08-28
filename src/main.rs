use std::env;
use std::fs;
use std::cmp::Ordering;
use std::convert::TryInto;
use text_colorizer::*;

use crate::ranker::{rank_hand, compare_high_card_values};

mod ranker;

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();

  if args.len() < 1 {
    eprintln!("{}", "Missing filename".red());
    std::process::exit(1);
  }

  let filename = &args[0];

  println!("Reading file: {}", filename.green());

  let data = match fs::read_to_string(filename) {
    Ok(buffer) => buffer,
    Err(e) => {
      eprintln!("{} failed to read from file {}: {:?}", "Error:".red(), filename.red(), e);
      std::process::exit(1);
    }
  };

  let result = score_poker_hands(data, num_cpus::get());
  println!("Player 1: {} hands", result.0);
  println!("Player 2: {} hands", result.1);
}

fn score_poker_hands(data: String, max_threads: usize) -> (usize, usize) {
  let lines: Vec<&str> = data.lines().collect();
  let total_lines = lines.len();
  let chunk_size = total_lines / max_threads + if total_lines % max_threads > 0 { 1 } else { 0 };

  crossbeam::scope(|spawner| {
    lines.chunks(chunk_size).into_iter().fold((0, 0), |wins, chunk| {
      let (lhs, rhs) = spawner.spawn(move |_| total_wins(chunk)).join().unwrap_or((0, 0));
      (wins.0 + lhs, wins.1 + rhs)
    })
  }).unwrap()
}

fn total_wins(lines: &[&str]) -> (usize, usize) {
  let mut wins: (usize, usize) = (0, 0);
  for line in lines {
    let cards: Vec<&str> = line.split_whitespace().collect();

    let mut hands_iter = cards.chunks(5).into_iter();

    let lhs = match hands_iter.next() {
      Some(values) => {
        if values.len() != 5 {
          eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
          std::process::exit(1);
        }
        values.try_into().unwrap()
      },
      None => {
        eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
        std::process::exit(1);
      }
    };

    let rhs = match hands_iter.next() {
      Some(values) => {
        if values.len() != 5 {
          eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
          std::process::exit(1);
        }
        values.try_into().unwrap()
      },
      None => {
        eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
        std::process::exit(1);
      }
    };

    let (lhs_score, rhs_score) = score_hands(lhs, rhs);
    wins.0 += lhs_score as usize;
    wins.1 += rhs_score as usize;
  }
  wins
}

fn score_hands(lhs: [&str; 5], rhs: [&str; 5]) -> (u8, u8) {
  let (lhs_rank, lhs_values) = rank_hand(lhs);
  let (rhs_rank, rhs_values) = rank_hand(rhs);

  // println!("lhs: {:?} {:?} {:?}", lhs, lhs_rank, lhs_values);
  // println!("rhs: {:?} {:?} {:?}", rhs, rhs_rank, rhs_values);

  if lhs_rank > rhs_rank {
    return (1, 0);
  }
  if lhs_rank < rhs_rank {
    return (0, 1);
  }

  match compare_high_card_values(lhs_values, rhs_values) {
    Ordering::Greater => (1, 0),
    Ordering::Less => (0, 1),
    Ordering::Equal => (0, 0),
  }
}
