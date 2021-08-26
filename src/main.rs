use std::env;
use std::fs;
use std::collections::HashMap;
use text_colorizer::*;

const VALUE_ORDERING: &str = "AKQJT98765432";

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

  let result = crossbeam::scope(|spawner| {
    lines.chunks(chunk_size).into_iter().fold((0, 0), |wins, chunk| {
      let (lhs, rhs) = spawner.spawn(move |_| count_wins(chunk)).join().unwrap_or((0, 0));
      (wins.0 + lhs, wins.1 + rhs)
    })
  }).unwrap();

  println!("Player 1: {} hands", result.0);
  println!("Player 2: {} hands", result.1);
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
  let (lhs_rank, lhs_high_card) = rank_hand(lhs);
  let (rhs_rank, rhs_high_card) = rank_hand(rhs);

  // println!("lhs: {:?} {:?} {:?}", lhs, lhs_rank, lhs_high_card);
  // println!("rhs: {:?} {:?} {:?}", rhs, rhs_rank, rhs_high_card);

  if lhs_rank > rhs_rank {
    return (1, 0);
  }
  if lhs_rank < rhs_rank {
    return (0, 1);
  }

  // rankings equal with need to check high cards
  // when rankings equal both sides will either contain their high card or both be None
  match lhs_high_card {
    Some(left_high_card) => {
      let right_high_card = rhs_high_card.unwrap();
      if left_high_card > right_high_card {
        return (1, 0);
      }
      if left_high_card < right_high_card {
        return (0, 1);
      }
      (0, 0)                                    // tie - no points
    },
    None => compare_individual_card_values(lhs, rhs)
  }
}

fn compare_individual_card_values(lhs: [&str; 5], rhs: [&str; 5]) -> (u8, u8) {
  let lhs_ordered = order_values(lhs);
  let rhs_ordered = order_values(rhs);

  match lhs_ordered.iter().zip(&rhs_ordered).find(|&(left, right)| left != right) {
    Some((left, right)) => {
      let index_of_left = VALUE_ORDERING.find(*left).unwrap();
      let index_of_right = VALUE_ORDERING.find(*right).unwrap();

      if index_of_left > index_of_right {
        return (1, 0);
      }
      return (0, 1);
    },
    None => (0, 0)
  }
}

fn rank_hand(hand: [&str; 5]) -> (usize, Option<char>) {
  match rank_multiple(hand) {                   // check for pairs and other duplicated values
    Some((rank, value)) => (rank, Some(value)),
    None => {
      if is_royal_flush(hand) {
        return (10, Some(VALUE_ORDERING.chars().nth(0).unwrap()));
      }
      if is_straight_flush(hand) {
        return (9, None);
      }
      if is_flush(hand) {
        return (6, None);
      }
      if is_straight(hand) {
        return (5, None);
      }
      
      (1, None)                                 // High card only
    }
  }
}

fn rank_multiple(hand: [&str; 5]) -> Option<(usize, char)> {
  // build a frequency distribution object of the card values e.g. { 'A': 3, 'K': 2 }
  let value_frequencies = hand.iter().fold(HashMap::<char, usize>::new(), |mut frequencies, card| {
    *frequencies.entry(card.chars().nth(0).unwrap()).or_default() += 1;
    frequencies
  });

  match value_frequencies.iter().max_by_key(|(_, value)| *value) {  // get value with highest frequency
    Some((value, 4)) => Some((8, *value)),      // four of a kind - ranking 8 (3rd best)
    Some((value, 3)) => {                       // three of a kind or full house
      match find_value_with_n_occurrences(&value_frequencies, 2) {  // check if the hand also contains a pair
        Some(_) => Some((7, *value)),           // full house - ranking 7
        None => Some((4, *value)),              // three of a kind only
      }
    },
    Some((value, 2)) => {                       // a pair or 2 pairs
      if value_frequencies.keys().len() == 3 {  // 3 keys in hashmap => 2 pairs
        return Some((3, *value));
      }
      Some((2, *value))                         // only 1 pair
    },
    _ => None                                   // no multiples found
  }
}

fn find_value_with_n_occurrences(map: &HashMap<char, usize>, occurrences: usize) -> Option<char> {
  map.iter().find_map(|(&value, &count)|
    if count == occurrences {
      Some(value)
    } else {
      None
    }
  )
}

fn is_royal_flush(hand: [&str; 5]) -> bool {
  // straight flush and the high card in an Ace (first char in VALUE_ORDERING)
  is_straight_flush(hand) && high_card_value(hand) == VALUE_ORDERING.chars().nth(0).unwrap()
}

fn is_straight_flush(hand: [&str; 5]) -> bool {
  is_flush(hand) && is_straight(hand)
}

fn is_flush(hand: [&str; 5]) -> bool {
  // second char (suit) of each consecutive pair are equal
  hand.windows(2).all(|w| w[0].chars().nth(1) == w[1].chars().nth(1))
}

fn is_straight(hand: [&str; 5]) -> bool {
  let ordered = order_values(hand);
  let index_of_highest = VALUE_ORDERING.find(ordered[0]).unwrap();
  let index_of_lowest = VALUE_ORDERING.find(ordered[4]).unwrap();

  // index of highest card is 4 away from lowest
  index_of_lowest - index_of_highest == 4
}

fn high_card_value(hand: [&str; 5]) -> char {
  order_values(hand)[0]
}

fn order_values(hand: [&str; 5]) -> Vec<char> {
  let mut values = hand.iter().map(|x| x.chars().nth(0).unwrap()).collect::<Vec<char>>();

  values.sort_by(|a, b| {
    let a_index = VALUE_ORDERING.find(*a).unwrap();
    let b_index = VALUE_ORDERING.find(*b).unwrap();
    a_index.cmp(&b_index)
  });

  values
}

#[test]
fn should_not_be_a_royal_flush() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(is_royal_flush(hand), false);
}

#[test]
fn should_be_a_royal_flush() {
  let hand = ["QH", "KH", "TH", "AH", "JH"];
  assert_eq!(is_royal_flush(hand), true);
}

#[test]
fn should_not_be_a_straight_flush() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(is_straight_flush(hand), false);
}

#[test]
fn should_be_a_straight_flush() {
  let hand = ["QH", "KH", "TH", "9H", "JH"];
  assert_eq!(is_straight_flush(hand), true);
}

#[test]
fn should_not_be_a_flush() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(is_flush(hand), false);
}

#[test]
fn should_be_a_flush() {
  let hand = ["2H", "KH", "TH", "9H", "8H"];
  assert_eq!(is_flush(hand), true);
}

#[test]
fn should_not_be_a_straight() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(is_straight(hand), false);
}

#[test]
fn should_be_a_straight() {
  let hand = ["QH", "KH", "TH", "9H", "JH"];
  assert_eq!(is_straight(hand), true);
}

#[test]
fn should_find_high_card_value() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(high_card_value(hand), 'A');
}

#[test]
fn should_order_values() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(order_values(hand), ['A', 'K', 'T', '9', '2']);
}
