#[macro_use]
extern crate lazy_static;

use std::io;

use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};

use sim::{Combatant, CombatantId, Simulation, Team};

use crate::sim::{describe_entry, CombatantInfo};

mod data;
mod dto;
mod sim;

fn run_many_sims(combatants: &[Combatant; 8]) -> (f64, u64) {
    let mut thread_rng = thread_rng();
    let num_runs = 20;
    let mut left_wins = 0;
    let mut time_outs = 0;
    for _ in 0..num_runs {
        let rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let mut sim = Simulation::new(combatants.clone(), 10, rng, false);
        sim.run();
        if sim.left_wins.unwrap() {
            left_wins += 1;
        }
        if sim.time_out_win.unwrap() {
            time_outs += 1;
        }
    }
    let left_wins_percent = left_wins as f64 / num_runs as f64;
    (left_wins_percent, time_outs)
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    assert!(min <= max);
    let mut x = n;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

fn main() -> io::Result<()> {
    // data::convert_data_from_feed()
    run_sims()
}

fn run_sims() -> io::Result<()> {
    let patches = data::read_all_patches()?;
    // let matches = data::read_all_match_ups()?;

    println!("{} patches\n", patches.len());

    let mut correct = 0;
    let total = 12391;
    let mut time_outs = 0;
    let mut log_loss: f64 = 0.0;

    let mut thread_rng = thread_rng();
    let random_replay = thread_rng.gen_range(0, total);
    let mut replay_data = vec![];

    let mut buffer = vec![];
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} {per_sec} {eta}",
            )
            .progress_chars("##-"),
    );
    for match_num in 0..total {
        bar.inc(1);
        let (patch_num, match_up) = data::read_match(match_num as usize, &mut buffer)?;
        let patch = patches
            .iter()
            .find(|p| p.time as usize == patch_num)
            .unwrap();

        let combatant_infos = [
            CombatantInfo::new(
                CombatantId::new(0),
                Team::Left,
                &match_up.left.combatants[0],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(1),
                Team::Left,
                &match_up.left.combatants[1],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(2),
                Team::Left,
                &match_up.left.combatants[2],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(3),
                Team::Left,
                &match_up.left.combatants[3],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(4),
                Team::Right,
                &match_up.right.combatants[0],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(5),
                Team::Right,
                &match_up.right.combatants[1],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(6),
                Team::Right,
                &match_up.right.combatants[2],
                patch,
            ),
            CombatantInfo::new(
                CombatantId::new(7),
                Team::Right,
                &match_up.right.combatants[3],
                patch,
            ),
        ];

        let combatants = [
            Combatant::new(&combatant_infos[0]),
            Combatant::new(&combatant_infos[1]),
            Combatant::new(&combatant_infos[2]),
            Combatant::new(&combatant_infos[3]),
            Combatant::new(&combatant_infos[4]),
            Combatant::new(&combatant_infos[5]),
            Combatant::new(&combatant_infos[6]),
            Combatant::new(&combatant_infos[7]),
        ];

        let (left_wins_percent, new_time_outs) = run_many_sims(&combatants);
        time_outs += new_time_outs;

        if match_up.left_wins.unwrap() && left_wins_percent > 0.5 {
            correct += 1;
        } else if !match_up.left_wins.unwrap() && left_wins_percent <= 0.5 {
            correct += 1;
        }

        let clamped = clamp(left_wins_percent, 1e-15, 1.0 - 1e-15);
        if match_up.left_wins.unwrap() {
            log_loss += -clamped.ln();
        } else {
            log_loss += -clamped.ln_1p();
        }

        if random_replay == match_num {
            let rng = SmallRng::from_rng(&mut thread_rng).unwrap();
            let mut sim = Simulation::new(combatants.clone(), 10, rng, true);
            sim.run();
            for entry in sim.log.entries() {
                replay_data.push(format!("{}", describe_entry(&entry)));
            }
        }
    }
    bar.finish();

    println!("\nmatch {}:", random_replay);
    for line in replay_data {
        println!("{}", line);
    }

    let correct_percent = correct as f32 / total as f32;
    println!("\ntotal: {}", total);
    println!(
        "correct: {:.1}%, time_outs: {}",
        correct_percent * 100.0,
        time_outs
    );
    println!("improvement: {:.1}%", (correct_percent - 0.5) * 200.0);
    println!("log loss: {:.6}", log_loss / total as f64);

    return Ok(());
}
