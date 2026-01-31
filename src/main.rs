use crount::args::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    crount::solve(args);
}
