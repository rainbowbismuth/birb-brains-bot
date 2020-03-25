use serde_json;

use dto::match_up::MatchUp;
use dto::patch::Patch;

mod dto;
mod sim;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    let patch: Patch = serde_json::from_str(&buffer).unwrap();
    buffer.clear();
    std::io::stdin().read_line(&mut buffer).unwrap();
    let match_up: MatchUp = serde_json::from_str(&buffer).unwrap();
    println!("Success");
}
