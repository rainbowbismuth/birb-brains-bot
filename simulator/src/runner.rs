use std::io;

use crate::data;
use crate::dto::rust::{MatchUp, Patch};
use crate::sim::{
    describe_entry, unit_card, Arena, Combatant, CombatantId, CombatantInfo, Gender, Pathfinder,
    Simulation, Team,
};
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;
use rand::{thread_rng, SeedableRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

pub fn run_many_sims<'a>(
    num_runs: i32,
    combatants: &'a [Combatant<'a>; 8],
    arena: &'a Arena,
) -> (f64, u64) {
    let mut thread_rng = thread_rng();
    let mut left_wins = 0;
    let mut time_outs = 0;
    for _ in 0..num_runs {
        let rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let pathfinder = RefCell::new(Pathfinder::new(&arena));
        let mut sim = Simulation::new(combatants.clone(), &arena, &pathfinder, rng, false);
        sim.run();
        if sim.left_wins.unwrap() {
            left_wins += 1;
        }
        if sim.time_out_win.unwrap() {
            time_outs += 1;
        }
    }
    let left_wins_percent = left_wins as f64 / num_runs as f64;
    (clamp(left_wins_percent, 0.01, 0.99), time_outs)
}

pub fn clamp(n: f64, min: f64, max: f64) -> f64 {
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

pub fn match_to_combatant_infos<'a>(
    patch: &'a Patch,
    match_up: &'a MatchUp,
) -> [CombatantInfo<'a>; 8] {
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

pub fn match_to_combatants<'a>(combatant_infos: &'a [CombatantInfo<'a>]) -> [Combatant<'a>; 8] {
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
    let arena = Arena::from_dto(match_up.arena.clone());
    let pathfinder = RefCell::new(Pathfinder::new(&arena));
    let (left_wins_percent, new_time_outs) = run_many_sims(num_runs, &combatants, &arena);
    let rng = SmallRng::from_entropy();
    let mut sim = Simulation::new(combatants.clone(), &arena, &pathfinder, rng, true);
    sim.run();

    for combatant in &combatants {
        println!("{}", unit_card(combatant));
    }
    println!("Playing on {}", &match_up.arena_name);
    for entry in sim.log.entries() {
        println!("{}", describe_entry(&entry, &arena));
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

pub fn has_ability(combatants: &[CombatantInfo], name: &str) -> bool {
    combatants
        .iter()
        .any(|info| info.abilities.iter().any(|ability| ability.name == name))
}

pub fn has_skill(combatants: &[CombatantInfo], name: &str) -> bool {
    combatants
        .iter()
        .any(|info| info.all_skills.iter().any(|skill| *skill == name))
}

pub fn has_monster(combatants: &[CombatantInfo]) -> bool {
    combatants.iter().any(|info| info.gender == Gender::Monster)
}

pub fn run_all_matches(
    num_runs: i32,
    print_worst: bool,
    save: bool,
    filter_equip: Vec<String>,
    filter_ability: Vec<String>,
    filter_skill: Vec<String>,
    filter_no_monsters: bool,
    filter_map: Vec<String>,
    most_recent: Option<u64>,
) -> io::Result<()> {
    let mut results: HashMap<String, HashMap<String, f64>> = HashMap::new();

    let patches = data::read_all_patches()?;

    println!("{} patches\n", patches.len());

    let mut match_up_paths = data::find_all_match_ups()?;

    if let Some(most_recent) = most_recent {
        match_up_paths.sort();
        match_up_paths.reverse();
        match_up_paths.truncate(most_recent as usize);
    }

    let mut worst_involves = HashMap::new();

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

    'filter: for match_up_path in match_up_paths.iter() {
        bar1.inc(1);
        let (patch_num, match_up) = data::read_match_at_path(&match_up_path, &mut buffer)?;
        let patch = patches
            .iter()
            .find(|p| p.time as usize == patch_num)
            .unwrap();
        let combatant_infos = match_to_combatant_infos(&patch, &match_up);

        for equip in &filter_equip {
            if !has_equip(&combatant_infos, equip) {
                continue 'filter;
            }
        }
        for ability in &filter_ability {
            if !has_ability(&combatant_infos, ability) {
                continue 'filter;
            }
        }
        for skill in &filter_skill {
            if !has_skill(&combatant_infos, skill) {
                continue 'filter;
            }
        }
        if filter_no_monsters && has_monster(&combatant_infos) {
            continue 'filter;
        }
        for map in &filter_map {
            if !match_up.arena_name.contains(map) {
                continue 'filter;
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
        let arena = Arena::from_dto(match_up.arena.clone());
        let (left_wins_percent, new_time_outs) = run_many_sims(num_runs, &combatants, &arena);
        time_outs += new_time_outs;

        // if new_time_outs > (num_runs as u64 / 2) {
        //     println!("time out heavy match: {}", replay_path.to_string_lossy());
        // }

        let tournament_map = results
            .entry(match_up.tournament_id.to_string())
            .or_insert(HashMap::new());
        let key = format!("{},{}", match_up.left.color, match_up.right.color);
        tournament_map.insert(key, left_wins_percent);

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
            for combatant in &match_up.left.combatants {
                *worst_involves.entry(combatant.class.clone()).or_insert(0) += 1;
                *worst_involves
                    .entry(combatant.action_skill.clone())
                    .or_insert(0) += 1;
            }
            for combatant in &match_up.right.combatants {
                *worst_involves.entry(combatant.class.clone()).or_insert(0) += 1;
                *worst_involves
                    .entry(combatant.action_skill.clone())
                    .or_insert(0) += 1;
            }

            worst_loss = current_log_loss;
            replay_path = (*match_up_path).clone();
            let rng = SmallRng::from_entropy();
            let arena = Arena::from_dto(match_up.arena.clone());
            let pathfinder = RefCell::new(Pathfinder::new(&arena));
            let mut sim = Simulation::new(combatants.clone(), &arena, &pathfinder, rng, true);
            sim.run();
            replay_data.clear();
            replay_data.push(format!("log loss: {}\n", current_log_loss));

            for combatant in &combatants {
                replay_data.push(unit_card(combatant));
            }
            replay_data.push(format!("Playing on {}", &match_up.arena_name));
            for entry in sim.log.entries() {
                replay_data.push(format!("{}", describe_entry(&entry, &arena)));
            }
        }
    }
    bar.finish();

    println!("\nmatch {}:", replay_path.to_string_lossy());
    for line in replay_data {
        println!("{}", line);
    }

    println!("\nworst matches involve:");
    let mut worst_involves_pairs: Vec<_> = worst_involves.iter().collect();
    worst_involves_pairs.sort_by_key(|p| -p.1);
    for entry in worst_involves_pairs {
        println!("{:>20}: {:>5}", entry.0, entry.1);
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

    if save {
        let bin = serde_json::to_vec_pretty(&results).unwrap();
        let mut file = std::fs::File::create("data/sim.json")?;
        file.write_all(&bin)?;
    }

    return Ok(());
}
