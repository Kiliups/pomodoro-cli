use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pomodoro", subcommand_required = false,)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(short = 'f', long, default_value_t = 25)]
        focus: u32,

        #[arg(short = 'b', long, default_value_t = 5)]
        break_time: u32,

        #[arg(short = 'c', long, default_value_t = 4)]
        cycles: u32,

        #[arg(short = 'l', long, default_value_t = 15)]
        long_break: u32,
    },
    Stop,
    Reset,
    Skip,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {
            focus,
            break_time,
            cycles,
            long_break,
        } => {
            println!(
                "Starting Pomodoro: {} min focus / {} min break, {} cycles ({} min long break)",
                focus, break_time, cycles, long_break
            );
        }
        Commands::Stop => println!("Stopping Pomodoro (not yet implemented)."),
        Commands::Reset => println!("Resetting Pomodoro stats (not yet implemented)."),
        Commands::Skip => println!("not implemented yet"),
    }
}
