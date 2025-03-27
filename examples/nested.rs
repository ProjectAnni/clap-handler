use clap::{Args, Parser, Subcommand};
use clap_handler::{Handler, handler};

#[derive(Parser, Handler, Debug, Clone)]
#[clap(name = "Your program", author)]
pub struct TopArgs {
    #[clap(short, long)]
    first: bool,

    #[clap(subcommand)]
    subcommand: Cmds,
}

#[derive(Subcommand, Handler, Debug, Clone)]
pub enum Cmds {
    First(FirstSubcommand),
}

#[derive(Parser, Debug, Clone)]
pub struct FirstSubcommand {
    #[clap(flatten)]
    flat: Flatten,

    #[clap(long)]
    arg: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct Flatten {
    #[clap(long)]
    pub arg2: Option<String>,
}

#[handler(FirstSubcommand)]
fn handle_first(me: FirstSubcommand, args: TopArgs) -> anyhow::Result<()> {
    println!("{}", args.first);
    Ok(())
}

fn main() {
    let args = TopArgs::parse();
    args.run().unwrap();
}
