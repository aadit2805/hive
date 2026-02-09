mod app;
mod animation;
mod demo;
mod event;
mod input;
mod positioning;
mod render;
mod state;

use std::path::PathBuf;

use clap::Parser;

use app::{App, AppConfig};

/// Hive: Real-time AI Agent Visualization
///
/// Watch AI agents work together like players on a field. Agents are positioned
/// semantically based on their focus areas, with heat maps showing work intensity,
/// trails showing thought paths, and smooth animations that make the swarm feel alive.
#[derive(Parser, Debug)]
#[command(name = "hive")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the events file to watch (JSON lines format)
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Run in demo mode with simulated agents
    #[arg(long)]
    demo: bool,

    /// Disable heat map display
    #[arg(long)]
    no_heatmap: bool,

    /// Disable trail display
    #[arg(long)]
    no_trails: bool,

    /// Disable landmark display
    #[arg(long)]
    no_landmarks: bool,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // Validate arguments
    if !cli.demo && cli.file.is_none() {
        eprintln!("Error: Either --file or --demo must be specified");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  hive --file events.jsonl   Watch a file for agent events");
        eprintln!("  hive --demo                Run demo mode with simulated agents");
        eprintln!();
        eprintln!("Run 'hive --help' for more options");
        std::process::exit(1);
    }

    let config = AppConfig {
        file_path: cli.file,
        demo_mode: cli.demo,
        show_heatmap: !cli.no_heatmap,
        show_trails: !cli.no_trails,
        show_landmarks: !cli.no_landmarks,
    };

    let mut app = App::new(config);

    // Run the app
    if let Err(e) = app.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
