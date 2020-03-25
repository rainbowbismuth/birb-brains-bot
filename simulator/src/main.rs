use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::thread;

use bincode;
use serde_json;

use dto::match_up::MatchUp;
use dto::patch::Patch;

mod dto;
mod sim;

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let mut match_counter = 0;
    let mut seen_patches = HashSet::new();

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

        let int_time = patch.time as isize;
        if !seen_patches.insert(int_time) {
            let patch_bin = bincode::serialize(&patch).unwrap();

            let mut file = std::fs::File::create(format!("data/sim/{}.patch", int_time))?;
            file.write_all(&patch_bin)?;
        }

        let info = bincode::serialize(&(int_time, match_up)).unwrap();
        let mut file = std::fs::File::create(format!("data/sim/{:06}.match", match_counter))?;
        match_counter += 1;
        file.write_all(&info)?;

        let result = dto::output::Output { error: None };
        println!("{}", serde_json::to_string(&result)?);
    }
}
