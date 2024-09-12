use clap::{Args, Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(author = "midoBB", version = env!("CARGO_PKG_VERSION"), about = "A Wayland clipboard manager / search", long_about = None, name = "ClipSearch")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Store(StoreArgs),
    Wipe,
    Version,
}

#[derive(Args)]
struct StoreArgs {
    max_dedupe_search: Option<usize>,
    max_items: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Store(args)) => handle_store(args),
        Some(Commands::Wipe) => handle_wipe(),
        Some(Commands::Version) => handle_version(),
        None => print_usage()
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  command [SUBCOMMAND]");
    println!();
    println!("Subcommands:");
    println!("  store           Store clipboard content");
    println!("  wipe            Wipe clipboard history");
    println!("  version         Print version information");
    println!();
    println!("For more information, use --help with any subcommand.");
}

fn handle_store(args: StoreArgs) {
    let clipboard_state = env::var("CLIPBOARD_STATE").unwrap_or_default();
    match clipboard_state.as_str() {
        "sensitive" | "clear" => {
            // Implement delete_last functionality
        }
        _ => {
            // Implement store functionality
            println!(
                "Store with max_dedupe_search: {:?}, max_items: {:?}",
                args.max_dedupe_search, args.max_items
            );
        }
    }
}

fn handle_wipe() {
    println!("Wipe");
}

fn handle_version() {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
}

