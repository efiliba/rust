use std::env;
use std::fs;
use text_colorizer::*;

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();

  if args.len() < 1 {
    eprintln!("{}", "Missing filename".red());
    std::process::exit(1);
  }

  let filename = &args[0];

  println!("Reading file: {}", filename.green());

  let reader = match fs::read_to_string(filename) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{} failed to read from file {}: {:?}", "Error:".red(), filename.red(), e);
      std::process::exit(1);
    }
  };

  let lines: Vec<&str> = reader.lines().collect();

  let total_lines = lines.len();
  let max_threads = num_cpus::get();
  let chunk_size = total_lines / max_threads + if total_lines % max_threads > 0 { 1 } else { 0 };
  let chunk_size = 2;

  let result = crossbeam::scope(|spawner| {
    lines.chunks(chunk_size).into_iter().fold((0, 0), |wins, chunk| {
      let (lhs, rhs) = spawner.spawn(move |_| count_wins(chunk)).join().unwrap_or((0, 0));
      (wins.0 + lhs, wins.1 + rhs)
    })
  }).unwrap();

  println!("{:?}", result);
}

fn count_wins(lines: &[&str]) -> (usize, usize) {
  let mut wins: (usize, usize) = (0, 0);
  for line in lines {
    let cards: Vec<&str> = line.split_whitespace().collect();

    let mut hands_iter = cards.chunks(5).into_iter();

    let lhs = match hands_iter.next() {
      Some(v) => {
        if v.len() != 5 {
          eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
          std::process::exit(1);
        }
        [v[0], v[1], v[2], v[3], v[4]]
      },
      None => {
        eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
        std::process::exit(1);
      }
    };

    let rhs = match hands_iter.next() {
      Some(v) => {
        if v.len() != 5 {
          eprintln!("{} invalid hand found: {:?}", "Error:".red(), cards);
          std::process::exit(1);
        }
        [v[0], v[1], v[2], v[3], v[4]]
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
  let rank = rank_hand(lhs);

  println!("rank: {:?}", rank);

  println!("lhs: {:?}", lhs);
  println!("rhs: {:?}", rhs);

  (1, 1)
}

fn rank_hand(hand: [&str; 5]) -> u8 {
  println!("hand: {:?}", hand);

  if is_royal_flush(hand) {
    return 10;
  }

  if is_straight_flush(hand) {
    return 9;
  }

  if is_four_of_a_kind(hand) {
    return 8;
  }

  if is_full_house(hand) {
    return 7;
  }

  if is_flush(hand) {
    return 6;
  }

  if is_straight(hand) {
    return 5;
  }

  if is_three_of_a_kind(hand) {
    return 4;
  }

  if is_two_pairs(hand) {
    return 3;
  }

  if is_a_pair(hand) {
    return 2;
  }

  1 // High card
}

fn is_royal_flush(hand: [&str; 5]) -> bool {
  is_straight_flush(hand) && high_card(hand) == 'A'
}

fn is_straight_flush(hand: [&str; 5]) -> bool {
  is_flush(hand) && is_straight(hand)
}

fn is_four_of_a_kind(hand: [&str; 5]) -> bool {
  false
}

fn is_full_house(hand: [&str; 5]) -> bool {
  false
}

fn is_flush(hand: [&str; 5]) -> bool {
  false
}

fn is_straight(hand: [&str; 5]) -> bool {
  false
}

fn is_three_of_a_kind(hand: [&str; 5]) -> bool {
  false
}

fn is_two_pairs(hand: [&str; 5]) -> bool {
  false
}

fn is_a_pair(hand: [&str; 5]) -> bool {
  false
}

fn high_card(hand: [&str; 5]) -> char {
  order_values(hand)[0]
}

fn order_values(hand: [&str; 5]) -> [char; 5] {
  ['A', 'K', 'Q', 'J', 'T']
}

#[test]
fn test_order_values() {
  let hand = ["2S", "KD", "TH", "9H", "8H"];
  assert_eq!(order_values(&hand), ['A', 'K', 'Q', 'J', 'T']);
}
