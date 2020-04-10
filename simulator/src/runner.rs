use std::io;

use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;
use rand::{thread_rng, SeedableRng};

use crate::data;
use crate::dto::rust::{MatchUp, Patch};
use crate::sim::{
    describe_entry, unit_card, Combatant, CombatantId, CombatantInfo, Simulation, Team,
};
use std::path::PathBuf;

fn run_many_sims<'a>(num_runs: i32, combatants: &'a [Combatant<'a>; 8]) -> (f64, u64) {
    let mut thread_rng = thread_rng();
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

fn match_to_combatant_infos<'a>(patch: &'a Patch, match_up: &'a MatchUp) -> [CombatantInfo<'a>; 8] {
    [
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
    ]
}

fn match_to_combatants<'a>(combatant_infos: &'a [CombatantInfo<'a>]) -> [Combatant<'a>; 8] {
    [
        Combatant::new(&combatant_infos[0]),
        Combatant::new(&combatant_infos[1]),
        Combatant::new(&combatant_infos[2]),
        Combatant::new(&combatant_infos[3]),
        Combatant::new(&combatant_infos[4]),
        Combatant::new(&combatant_infos[5]),
        Combatant::new(&combatant_infos[6]),
        Combatant::new(&combatant_infos[7]),
    ]
}

pub fn run_specific_match(match_id: u64, num_runs: i32) -> io::Result<()> {
    let patches = data::read_all_patches()?;
    let mut buffer = Vec::with_capacity(1024 * 1024 * 2);
    let (patch_num, match_up) = data::read_match(match_id, &mut buffer)?;
    let patch = patches
        .iter()
        .find(|p| p.time as usize == patch_num)
        .unwrap();
    let combatant_infos = match_to_combatant_infos(&patch, &match_up);
    let combatants = match_to_combatants(&combatant_infos);
    let (left_wins_percent, new_time_outs) = run_many_sims(num_runs, &combatants);
    let rng = SmallRng::from_entropy();
    let mut sim = Simulation::new(combatants.clone(), 10, rng, true);
    sim.run();

    for combatant in &combatants {
        println!("{}", unit_card(combatant));
    }
    for entry in sim.log.entries() {
        println!("{}", describe_entry(&entry));
    }
    let clamped = clamp(left_wins_percent, 1e-15, 1.0 - 1e-15);
    let current_log_loss = if match_up.left_wins.unwrap() {
        -clamped.ln()
    } else {
        -clamped.ln_1p()
    };
    println!("log loss: {:.6}", current_log_loss as f64);
    println!("time outs: {}", new_time_outs);
    Ok(())
}

pub fn has_equip(combatants: &[CombatantInfo], name: &str) -> bool {
    combatants.iter().any(|info| {
        info.main_hand.map_or(false, |eq| eq.name == name)
            || info.off_hand.map_or(false, |eq| eq.name == name)
            || info.headgear.map_or(false, |eq| eq.name == name)
            || info.armor.map_or(false, |eq| eq.name == name)
            || info.accessory.map_or(false, |eq| eq.name == name)
    })
}

pub fn run_all_matches(
    num_runs: i32,
    print_worst: bool,
    filter_equip: Option<String>,
) -> io::Result<()> {
    let patches = data::read_all_patches()?;

    println!("{} patches\n", patches.len());

    let match_up_paths = data::find_all_match_ups()?;

    let mut correct = 0;

    let mut time_outs = 0;
    let mut log_loss: f64 = 0.0;

    let mut worst_loss = 0.0;
    let mut replay_path = PathBuf::new();
    let mut replay_data = vec![];

    let mut buffer = vec![];
    let mut match_ups = vec![];

    let bar1 = ProgressBar::new(match_up_paths.len() as u64);
    bar1.set_style(
        ProgressStyle::default_bar()
            .template(
                "Loading... [{elapsed_precise}] {bar:40.purple/blue} {pos:>9}/{len:9} {msg} {per_sec} {eta}",
            )
            .progress_chars("##-"),
    );

    for match_up_path in match_up_paths.iter() {
        bar1.inc(1);
        let (patch_num, match_up) = data::read_match_at_path(&match_up_path, &mut buffer)?;
        let patch = patches
            .iter()
            .find(|p| p.time as usize == patch_num)
            .unwrap();
        let combatant_infos = match_to_combatant_infos(&patch, &match_up);
        if let Some(ref equip) = filter_equip {
            if !has_equip(&combatant_infos, equip) {
                continue;
            }
        }
        match_ups.push((match_up_path, patch, match_up));
    }
    bar1.finish();
    let total = match_ups.len() as u64 * num_runs as u64;

    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Simulating [{elapsed_precise}] {bar:40.cyan/blue} {pos:>9}/{len:9} {msg} {per_sec} {eta}",
            )
            .progress_chars("##-"),
    );

    for (match_up_path, patch, match_up) in match_ups.iter() {
        bar.inc(num_runs as u64);

        let combatant_infos = match_to_combatant_infos(&patch, &match_up);
        let combatants = match_to_combatants(&combatant_infos);
        let (left_wins_percent, new_time_outs) = run_many_sims(num_runs, &combatants);
        time_outs += new_time_outs;

        if match_up.left_wins.unwrap() && left_wins_percent > 0.5 {
            correct += 1;
        } else if !match_up.left_wins.unwrap() && left_wins_percent <= 0.5 {
            correct += 1;
        }

        let clamped = clamp(left_wins_percent, 1e-15, 1.0 - 1e-15);
        let current_log_loss = if match_up.left_wins.unwrap() {
            -clamped.ln()
        } else {
            -clamped.ln_1p()
        };
        log_loss += current_log_loss;

        if print_worst && current_log_loss >= worst_loss {
            worst_loss = current_log_loss;
            replay_path = (*match_up_path).clone();
            let rng = SmallRng::from_entropy();
            let mut sim = Simulation::new(combatants.clone(), 10, rng, true);
            sim.run();
            replay_data.clear();
            replay_data.push(format!("log loss: {}", current_log_loss));
            for combatant in &combatants {
                replay_data.push(unit_card(combatant));
            }
            for entry in sim.log.entries() {
                replay_data.push(format!("{}", describe_entry(&entry)));
            }
        }
    }
    bar.finish();

    println!("\nmatch {}:", replay_path.to_string_lossy());
    for line in replay_data {
        println!("{}", line);
    }

    let total_matches = match_ups.len();
    let correct_percent = correct as f32 / total_matches as f32;
    println!("\ntotal: {}", total_matches);
    println!("correct: {:.1}%", correct_percent * 100.0);
    println!(
        "time_outs: {:.1}%",
        ((time_outs as f32 / num_runs as f32) / total_matches as f32) * 100.0
    );
    println!("improvement: {:.1}%", (correct_percent - 0.5) * 200.0);
    println!("log loss: {:.6}", log_loss / total_matches as f64);

    return Ok(());
}
