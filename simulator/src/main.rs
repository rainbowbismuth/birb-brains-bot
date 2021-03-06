#[macro_use]
extern crate lazy_static;

use std::io;

use clap::Clap;

pub mod data;
pub mod dto;
pub mod runner;
pub mod sim;

/// This program is a rough re-implementation of Final Fantasy Tactics & the game's AI,
/// for the purposes of predicting matches on the twitch channel, FFTBattleground.
#[derive(Clap)]
#[clap(version = "0.1", author = "Emily A. Bellows")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Test the engine by running against nearly every match from the stream.
    #[clap(name = "test")]
    Test(Test),

    /// Run a specific match
    #[clap(name = "run")]
    Run(Run),

    /// Read match up & patch data from my python code on stdin, writing out the match up &
    /// patch data into a binary format this program expects.
    #[clap(name = "feed")]
    Feed(Feed),
}

#[derive(Clap)]
struct Test {
    /// The number of simulated matches per match up
    #[clap(short = "n")]
    num_runs: i32,

    /// Print the match with the highest log loss at the end
    #[clap(short = "w")]
    print_worst: bool,

    /// Save the predictions to a file
    #[clap(long = "save")]
    save: bool,

    /// Run most recent M matches
    #[clap(short = "m")]
    most_recent: Option<u64>,

    /// Filter out all matches with any monsters
    #[clap(long = "filter-no-monsters")]
    filter_no_monsters: bool,

    /// Filter for a piece of equipment
    #[clap(long = "filter-equip")]
    filter_equip: Vec<String>,

    /// Filter for an implemented ability
    #[clap(long = "filter-ability")]
    filter_ability: Vec<String>,

    /// Filter for any skill
    #[clap(long = "filter-skill")]
    filter_skill: Vec<String>,

    /// Filter any map
    #[clap(long = "filter-map")]
    filter_map: Vec<String>,
}

#[derive(Clap)]
struct Run {
    /// The number of simulated matches to run
    #[clap(short = "n")]
    num_runs: i32,

    /// The match ID
    match_id: u64,
}

#[derive(Clap)]
struct Feed {}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    match opts.sub_cmd {
        SubCommand::Test(test) => runner::run_all_matches(
            test.num_runs,
            test.print_worst,
            test.save,
            test.filter_equip,
            test.filter_ability,
            test.filter_skill,
            test.filter_no_monsters,
            test.filter_map,
            test.most_recent,
        ),
        SubCommand::Run(run) => runner::run_specific_match(run.match_id, run.num_runs),
        SubCommand::Feed(_feed) => data::convert_data_from_feed(),
    }
}
