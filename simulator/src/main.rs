#[macro_use]
extern crate lazy_static;

use std::io;

mod data;
mod dto;
mod runner;
mod sim;

use clap::Clap;

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

    /// Print a random match at the end
    #[clap(short = "r")]
    print_random: bool,
}

#[derive(Clap)]
struct Feed {}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    match opts.sub_cmd {
        SubCommand::Test(test) => runner::run_all_matches(test.num_runs, test.print_random),
        SubCommand::Feed(_feed) => data::convert_data_from_feed(),
    }
}
