use std::io;
use std::io::Write;
use std::thread;

use serde_json;

use dto::match_up::MatchUp;
use dto::patch::Patch;

mod dto;
mod sim;

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    loop {
        if std::io::stdin().read_line(&mut buffer)? == 0 {
            return Ok(());
        }
        let patch: Patch = serde_json::from_str(&buffer).unwrap();
        buffer.clear();
        if std::io::stdin().read_line(&mut buffer)? == 0 {
            return Ok(());
        }
        let match_up: MatchUp = serde_json::from_str(&buffer).unwrap();
        buffer.clear();

        let result = dto::output::Output { error: None };
        println!("{}", serde_json::to_string(&result)?);
    }
}
