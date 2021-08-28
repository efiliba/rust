use std::collections::HashMap;
use std::convert::TryInto;
use std::cmp::Ordering;

const CARD_VALUE_ORDERING: &str = "AKQJT98765432";

// rank hands from 10 for royal flush to 1 for high card, and return the orderd card values needed to break ties
pub fn rank_hand(hand: [&str; 5]) -> (usize, Vec<char>) {
  match rank_multiple(hand) {                         // check for pairs and other duplicated values
    Some(result) => result,                           // found duplicated card vales
    None => {
      let ordered_values = order_values(hand);        // extract values from each card and order them

      let staight = is_straight(&ordered_values);     // only the values (not suits) required to determine if straight
      let flush = is_flush(&hand);                    // flush needs the original hand

      if staight && flush {
        if ordered_values[0] == CARD_VALUE_ORDERING.chars().nth(0).unwrap() { // check if high card is an ace
          return (10, Vec::<char>::new());            // royal flush - no card values required for comparisons
        }
        return (9, vec![ordered_values[0]]);          // staight flush - only highest value needed for comparisons
      }

      if staight {
        return (5, vec![ordered_values[0]]);
      }

      if flush {
        return (6, ordered_values.to_vec());          // flush may need all card values to break ties
      }

      (1, ordered_values.to_vec())                    // High card only
    }
  }
}

pub fn compare_high_card_values(lhs: Vec<char>, rhs: Vec<char>) -> Ordering {
  match lhs.iter().zip(&rhs).find(|&(left, right)| left != right) {
    Some((left, right)) => {
      let left_index = CARD_VALUE_ORDERING.find(*left).unwrap();
      let right_index = CARD_VALUE_ORDERING.find(*right).unwrap();

      if left_index < right_index {
        return Ordering::Greater;
      }
      Ordering::Less
    },
    None => Ordering::Equal
  }
}

fn rank_multiple(hand: [&str; 5]) -> Option<(usize, Vec<char>)> {
  // build a frequency distribution object of the card values e.g. { 'A': 3, 'K': 2 }
  let value_frequencies = hand.iter().fold(HashMap::<char, usize>::new(), |mut frequencies, card| {
    *frequencies.entry(card.chars().nth(0).unwrap()).or_default() += 1;
    frequencies
  });

  match value_frequencies.iter().max_by_key(|(_, value)| *value) {  // get value with highest frequency
    Some((value, 4)) => Some((8, vec![*value])),      // four of a kind - ranking 8 (3rd best)
    Some((value, 3)) => {                             // three of a kind or full house
      match find_value_with_n_occurrences(&value_frequencies, 2) {  // check if the hand also contains a pair
        Some(_) => Some((7, vec![*value])),           // full house - ranking 7
        None => Some((4, vec![*value])),              // three of a kind only
      }
    },
    Some((value, 2)) => {                             // a pair or 2 pairs
      Some(extract_all_pairs(&value_frequencies, value))
    },
    _ => None                                         // no multiples found
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

fn extract_all_pairs(value_frequencies: &HashMap<char, usize>, value: &char) -> (usize, Vec<char>) {
  // get ordered card values with a frequency of 1
  let mut ordered_single_values: Vec<char> = value_frequencies.iter()
    .filter_map(|(key, value)| match value {
      1 => Some(*key),
      _ => None
    })
    .collect();
  sort_by_card_values(&mut ordered_single_values);

  if ordered_single_values.len() == 1 {               // only 1 single value exists => 2 pairs
    let other_pair = value_frequencies.iter()
      .find(|&(key, occurrences)| *occurrences == 2 && key != value)
      .map(|(key, _)| *key).expect("second pair not found");

    let ordered_pair_values = match sort_card_values(value, &other_pair) {
      Ordering::Greater => vec![other_pair, *value],
      _ => vec![*value, other_pair]
    };

    return (3, ordered_pair_values.into_iter().chain(ordered_single_values).collect());
  }
  
  (2, vec![*value].into_iter().chain(ordered_single_values).collect())  // only 1 pair
}

fn is_flush(hand: &[&str; 5]) -> bool {
  // second char (suit) of each consecutive pair equal
  hand.windows(2).all(|w| w[0].chars().nth(1) == w[1].chars().nth(1))
}

// Assumes that the hand does not have duplicate values
// NOTE: Does not count A, 2, 3, 4, 5 as a straight
fn is_straight(ordered_hand: &[char; 5]) -> bool {
  let index_of_highest = CARD_VALUE_ORDERING.find(ordered_hand[0]).unwrap();
  let index_of_lowest = CARD_VALUE_ORDERING.find(ordered_hand[4]).unwrap();

  // index of highest card is 4 away from lowest
  index_of_lowest - index_of_highest == 4
}

fn order_values(hand: [&str; 5]) -> [char; 5] {
  let mut values: Vec<char> = hand.iter().map(|card| card.chars().nth(0).unwrap()).collect();

  sort_by_card_values(&mut values);

  values.try_into().unwrap_or_else(|v: Vec<char>| panic!("Hand contains {} cards, expected 5.", v.len()))
}

fn sort_by_card_values(values: &mut Vec<char>) {
  values.sort_by(sort_card_values);
}

fn sort_card_values(a: &char, b: &char) -> Ordering {
  let a_index = CARD_VALUE_ORDERING.find(*a).unwrap();
  let b_index = CARD_VALUE_ORDERING.find(*b).unwrap();
  a_index.cmp(&b_index)
}

#[test]
fn should_rank_hands_and_return_ordered_values_required_for_comparison() {
  let high_card = ["2S", "KD", "TH", "9H", "AD"];       // High card        1
  let pair = ["2S", "KD", "TH", "2H", "AD"];            // Pair             2
  let two_pairs = ["2S", "KD", "9H", "9H", "KD"];       // Two pairs        3
  let three_of_a_kind = ["TS", "KD", "TH", "TH", "AD"]; // Three of a kind  4
  let straight = ["2S", "6D", "5H", "3H", "4D"];        // Straight         5
  let flush = ["2H", "KH", "TH", "9H", "AH"];           // Flush            6
  let full_house = ["3S", "KD", "3H", "KH", "3D"];      // Full house       7
  let four_of_a_kind = ["TS", "KD", "TH", "TC", "TD"];  // Four of a kind   8
  let straight_flush = ["2S", "6S", "3S", "4S", "5S"];  // Straight flush   9
  let royal_flush = ["QD", "KD", "TD", "JD", "AD"];     // Royal flush     10

  assert_eq!(rank_hand(high_card), (1, vec!['A', 'K', 'T', '9', '2']));
  assert_eq!(rank_hand(pair), (2, vec!['2', 'A', 'K', 'T']));
  assert_eq!(rank_hand(two_pairs), (3, vec!['K', '9', '2']));
  assert_eq!(rank_hand(three_of_a_kind), (4, vec!['T']));
  assert_eq!(rank_hand(straight), (5, vec!['6']));
  assert_eq!(rank_hand(flush), (6, vec!['A', 'K', 'T', '9', '2']));
  assert_eq!(rank_hand(full_house), (7, vec!['3']));
  assert_eq!(rank_hand(four_of_a_kind), (8, vec!['T']));
  assert_eq!(rank_hand(straight_flush), (9, vec!['6']));
  assert_eq!(rank_hand(royal_flush), (10, vec![]));
}

#[test]
fn should_compare_high_card_single_values_as_equal() {
  assert_eq!(compare_high_card_values(vec!['A'], vec!['A']), Ordering::Equal);
}

#[test]
fn should_compare_high_card_multi_values_as_equal() {
  assert_eq!(compare_high_card_values(vec!['2', '7', 'K'], vec!['2', '7', 'K']), Ordering::Equal);
}

#[test]
fn should_return_lhs_greater_when_comparing_high_card_single_values() {
  assert_eq!(compare_high_card_values(vec!['A'], vec!['7']), Ordering::Greater);
}

#[test]
fn should_return_lhs_greater_when_comparing_high_card_multi_values() {
  assert_eq!(compare_high_card_values(vec!['7', 'K', 'A'], vec!['7', 'Q', 'A']), Ordering::Greater);
}

#[test]
fn should_not_contain_a_flush() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(is_flush(&hand), false);
}

#[test]
fn should_be_at_least_a_flush() {
  let hand = ["2H", "KH", "TH", "9H", "8H"];
  assert!(is_flush(&hand));
}

#[test]
fn should_not_contain_a_straight() {
  let ordered_hand = order_values(["2S", "KD", "TH", "9H", "AD"]);
  assert_eq!(is_straight(&ordered_hand), false);
}

#[test]
fn should_not_contain_low_ace_as_a_straight() {
  let ordered_hand = order_values(["2S", "4D", "5H", "3H", "AD"]);
  assert_eq!(is_straight(&ordered_hand), false);
}

#[test]
fn should_be_at_least_a_straight() {
  let ordered_hand = order_values(["QH", "KH", "TH", "AH", "JH"]);
  assert_eq!(is_straight(&ordered_hand), true);
}

#[test]
fn should_return_is_straight_and_flush_and_high_card_an_ace_for_royal_flush() {
  let hand = ["QH", "KH", "TH", "AH", "JH"];
  let ordered_hand = order_values(hand);
  assert!(is_straight(&ordered_hand));
  assert!(is_flush(&hand));
  assert_eq!(ordered_hand[0], CARD_VALUE_ORDERING.chars().nth(0).unwrap()); // high card an Ace
}

#[test]
fn should_order_values() {
  let hand = ["2S", "KD", "TH", "9H", "AD"];
  assert_eq!(order_values(hand), ['A', 'K', 'T', '9', '2']);
}
