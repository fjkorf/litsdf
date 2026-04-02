use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "litsdf-cli", about = "Command-line tool for manipulating litsdf SDF scene files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scene-level operations
    Scene {
        #[command(subcommand)]
        cmd: commands::scene::SceneCmd,
    },
    /// Bone operations
    Bone {
        #[command(subcommand)]
        cmd: commands::bone::BoneCmd,
    },
    /// Shape operations
    Shape {
        #[command(subcommand)]
        cmd: commands::shape::ShapeCmd,
    },
    /// Modifier operations
    Modifier {
        #[command(subcommand)]
        cmd: commands::modifier::ModifierCmd,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Scene { cmd } => commands::scene::run(cmd),
        Commands::Bone { cmd } => commands::bone::run(cmd),
        Commands::Shape { cmd } => commands::shape::run(cmd),
        Commands::Modifier { cmd } => commands::modifier::run(cmd),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
