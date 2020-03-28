use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Write;

use rmp_serde;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json;

use crate::dto::match_up::MatchUp;
use crate::dto::patch::Patch;

pub fn convert_data_from_feed() -> io::Result<()> {
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

        let int_time = patch.time as usize;
        if !seen_patches.insert(int_time) {
            let patch_bin = rmp_serde::to_vec(&patch).unwrap();

            let mut file = std::fs::File::create(format!("data/sim/{}.patch", int_time))?;
            file.write_all(&patch_bin)?;
        }

        let info = rmp_serde::to_vec(&(int_time, match_up)).unwrap();
        let mut file = std::fs::File::create(format!("data/sim/{:06}.match", match_counter))?;
        match_counter += 1;
        file.write_all(&info)?;
    }
}

fn read_files_matching<T: DeserializeOwned>(extension: &str) -> io::Result<Vec<T>> {
    let mut out = vec![];
    let extension: OsString = OsString::from(extension);
    for entry in fs::read_dir("data/sim/")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension() != Some(&extension) {
            continue;
        }
        let file = std::fs::File::open(path)?;
        let patch = rmp_serde::from_read(file).unwrap();
        out.push(patch);
    }
    return Ok(out);
}

pub fn read_all_patches() -> io::Result<Vec<Patch>> {
    return read_files_matching("patch");
}

pub fn read_all_match_ups() -> io::Result<Vec<(usize, MatchUp)>> {
    return read_files_matching("match");
}

pub fn read_match(id: usize) -> io::Result<(usize, MatchUp)> {
    let file = fs::File::open(format!("data/sim/{:06}.match", id))?;
    Ok(rmp_serde::from_read(file).unwrap())
}