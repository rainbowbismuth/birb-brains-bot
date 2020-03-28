use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::{Read, Write};

use bincode;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json;

use crate::dto::python;
use crate::dto::rust;

pub fn convert_data_from_feed() -> io::Result<()> {
    let mut buffer = String::new();
    let mut match_counter = 0;
    let mut seen_patches = HashSet::new();

    loop {
        if std::io::stdin().read_line(&mut buffer)? == 0 {
            return Ok(());
        }
        let patch: python::Patch = serde_json::from_str(&buffer).unwrap();
        buffer.clear();
        if std::io::stdin().read_line(&mut buffer)? == 0 {
            return Ok(());
        }
        let match_up: python::MatchUp = serde_json::from_str(&buffer).unwrap();
        buffer.clear();

        let int_time = patch.time as u64;
        if !seen_patches.insert(int_time) {
            let rust_patch = rust::Patch::from_python(patch);

            let rust_bin = bincode::serialize(&rust_patch).unwrap();
            let mut file = std::fs::File::create(format!("data/sim/{}.patch", int_time))?;
            file.write_all(&rust_bin)?;
        }

        let rust_match_up = rust::MatchUp::from_python(match_up);
        let rust_bin = bincode::serialize(&(int_time, rust_match_up)).unwrap();
        let mut file = std::fs::File::create(format!("data/sim/{:06}.match", match_counter))?;
        file.write_all(&rust_bin)?;
        match_counter += 1;
    }
}

fn read_files_matching<T: DeserializeOwned>(extension: &str) -> io::Result<Vec<T>> {
    let mut out = vec![];
    let extension: OsString = OsString::from(extension);
    let mut buffer = vec![];
    for entry in fs::read_dir("data/sim/")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension() != Some(&extension) {
            continue;
        }
        buffer.clear();
        let bin = std::fs::File::open(path)?.read_to_end(&mut buffer)?;
        let val = bincode::deserialize(&buffer).unwrap();
        out.push(val);
    }
    return Ok(out);
}

pub fn read_all_patches() -> io::Result<Vec<rust::Patch>> {
    return read_files_matching("patch");
}

pub fn read_all_match_ups() -> io::Result<Vec<(usize, rust::MatchUp)>> {
    return read_files_matching("match");
}

pub fn read_match(id: usize, buffer: &mut Vec<u8>) -> io::Result<(usize, rust::MatchUp)> {
    buffer.clear();
    fs::File::open(format!("data/sim/{:06}.match", id))?.read_to_end(buffer)?;
    Ok(bincode::deserialize(&buffer).unwrap())
}