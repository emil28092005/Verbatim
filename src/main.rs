mod world;
mod physics;
mod entity;
mod render;
mod input;
mod game;

use clap::Parser;
use game::Game;
use render::terminal::TerminalRenderer;

#[derive(Parser, Debug)]
#[command(name = "verbatim", about = "ASCII physics RPG - Noita meets Caves of Qud")]
struct Cli {
    #[arg(long, default_value = "terminal")]
    render_mode: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.render_mode.as_str() {
        "terminal" => {
            let mut renderer = TerminalRenderer::new();
            let mut game = Game::new();
            game.run(&mut renderer);
        }
        "vulkan" => {
            eprintln!("Vulkan renderer not yet implemented. Use --render-mode terminal.");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Unknown render mode: {}. Use 'terminal' or 'vulkan'.", cli.render_mode);
            std::process::exit(1);
        }
    }
}
