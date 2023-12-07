use std::path::PathBuf;

use clap::Parser;
use colored::control::set_override;
use lock_diff::{CargoLock, CargoLockDiff};

#[derive(Parser)]
struct Cli {
    old: PathBuf,
    new: PathBuf,

    #[arg(long, default_value = "false")]
    no_color: bool,

    #[arg(short, default_value = "false")]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let old_lock = CargoLock::load_lock(cli.old);
    let new_lock = CargoLock::load_lock(cli.new);

    set_override(!cli.no_color);

    CargoLockDiff::difference(old_lock, new_lock).pretty_print(cli.verbose);
}
