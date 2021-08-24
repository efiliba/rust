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

  let result = crossbeam::scope(|spawner| {
    lines.chunks(2).into_iter().fold((0, 0), |wins, chunk| {
      let (lhs, rhs) = spawner.spawn(move |_| count_wins(chunk)).join().unwrap_or((0, 0));
      (wins.0 + lhs, wins.1 + rhs)
    })
  }).unwrap();

  println!("{:?}", result);
}

fn count_wins(lines: &[&str]) -> (i32, i32) {
  println!("{:?}", lines);

  (5, 4)
}
