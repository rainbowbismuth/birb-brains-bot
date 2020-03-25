use std::io;

use indicatif::ProgressIterator;

mod dto;
mod sim;
mod data;

fn main() -> io::Result<()> {
    // data::convert_data_from_feed()?;
    let patches = data::read_all_patches()?;
    let matches = data::read_all_match_ups()?;

    println!("{} patches", patches.len());
    println!("{} matches", matches.len());

    return Ok(());
}
