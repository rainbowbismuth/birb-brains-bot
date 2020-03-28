#[macro_use]
extern crate lazy_static;

use std::io;

use indicatif::ProgressIterator;
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;

use sim::{Combatant, CombatantId, Simulation, Team};

use crate::sim::describe_entry;

mod dto;
mod sim;
mod data;

fn main() -> io::Result<()> {
    // data::convert_data_from_feed()?;
    let patches = data::read_all_patches()?;
    // let matches = data::read_all_match_ups()?;

    println!("{} patches\n", patches.len());
    // println!("{} matches", matches.len());

    let mut thread_rng = thread_rng();

    // GOING TO TRY TO RUN A MATCH AAA
    // let (patch_num, match_up) = &matches[0];
    let (patch_num, match_up) = data::read_match(0)?;
    let patch = patches.iter().find(|p| p.time as usize == patch_num).unwrap();

    let combatants = [
        Combatant::new(CombatantId::new(0), Team::Left, &match_up.left.combatants[0], patch),
        Combatant::new(CombatantId::new(1), Team::Left, &match_up.left.combatants[1], patch),
        Combatant::new(CombatantId::new(2), Team::Left, &match_up.left.combatants[2], patch),
        Combatant::new(CombatantId::new(3), Team::Left, &match_up.left.combatants[3], patch),
        Combatant::new(CombatantId::new(4), Team::Right, &match_up.right.combatants[0], patch),
        Combatant::new(CombatantId::new(5), Team::Right, &match_up.right.combatants[1], patch),
        Combatant::new(CombatantId::new(6), Team::Right, &match_up.right.combatants[2], patch),
        Combatant::new(CombatantId::new(7), Team::Right, &match_up.right.combatants[3], patch),
    ];

    let mut sim = Simulation::new(combatants, 10, SmallRng::from_rng(&mut thread_rng).unwrap());
    sim.run();

    // for entry in sim.log.entries() {
    //     println!("{}", describe_entry(&entry));
    // }

    return Ok(());
}
