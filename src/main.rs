mod world;
mod physics;
mod entity;
mod render;
mod input;
mod game;

use clap::Parser;
use game::Game;
use render::terminal::TerminalRenderer;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(name = "verbatim", about = "ASCII physics RPG - Noita meets Caves of Qud")]
struct Cli {
    #[arg(long, default_value = "terminal")]
    render_mode: String,

    #[arg(long, default_value_t = 0)]
    headless_ticks: u32,
}

fn main() {
    let cli = Cli::parse();

    if cli.headless_ticks > 0 {
        run_headless(cli.headless_ticks);
        return;
    }

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

fn run_headless(ticks: u32) {
    let mut game = Game::new();
    game.init_world();

    let (px, py) = game.player.center(&game.entities);
    let cam_x = px as i32 - 40;
    let cam_y = py as i32 - 12;

    let mut log = String::new();
    log.push_str(&format!("=== Verbatim Headless Run: {} ticks ===\n\n", ticks));
    log.push_str(&format!("World: {}x{}\n", game.grid.width, game.grid.height));
    log.push_str(&format!("Player start: ({:.1}, {:.1})\n", px, py));
    log.push_str(&format!("Camera: ({}, {})\n\n", cam_x, cam_y));

    log.push_str("=== Tick 0 (initial state) ===\n");
    log.push_str(&dump_view(&game.grid, &game.entities, cam_x, cam_y, 80, 25));
    log.push_str(&format!("Player: {:?}\n", player_info(&game)));
    log.push_str(&format!("Entities: {}\n\n", game.entities.all().len()));

    let dump_interval = (ticks / 10).max(1);

    for t in 1..=ticks {
        game.fixed_update();

        if t % dump_interval == 0 || t == ticks {
            let (px, py) = game.player.center(&game.entities);
            let cam_x = px as i32 - 40;
            let cam_y = py as i32 - 12;

            log.push_str(&format!("=== Tick {} ===\n", t));
            log.push_str(&dump_view(&game.grid, &game.entities, cam_x, cam_y, 80, 25));
            log.push_str(&format!("Player: {:?}\n", player_info(&game)));
            log.push_str(&format!("Entities: {}\n", game.entities.all().len()));

            let alive: Vec<_> = game.entities.all().iter()
                .filter(|e| e.alive)
                .map(|e| format!("{}(hp={:.0}, pos={:?})", entity_kind_name(e.kind), e.health, e.center()))
                .collect();
            log.push_str(&format!("Alive entities: {}\n\n", alive.join(", ")));
        }
    }

    let mut f = std::fs::File::create("headless_dump.txt").expect("Cannot create dump file");
    f.write_all(log.as_bytes()).expect("Cannot write dump");
    eprintln!("Headless run complete: {} ticks, dump written to headless_dump.txt", ticks);
}

fn dump_view(grid: &world::grid::Grid, entities: &entity::EntityManager, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    let mut buf = String::with_capacity(vw * vh + vh + 100);
    buf.push_str(&format!("  Camera ({}, {}):\n", cam_x, cam_y));

    let mut entity_map = std::collections::HashMap::new();
    for e in entities.all() {
        for b in &e.bodies {
            if !b.alive { continue; }
            let sx = b.x as i32 - cam_x;
            let sy = b.y as i32 - cam_y;
            if sx >= 0 && sx < vw as i32 && sy >= 0 && sy < vh as i32 {
                let ch = match e.kind {
                    entity::EntityKind::Player if e.alive => '@',
                    entity::EntityKind::Goblin if e.alive => 'g',
                    _ => '%',
                };
                entity_map.insert((sx, sy), ch);
            }
        }
    }

    for dy in 0..vh {
        let y = cam_y + dy as i32;
        if dy == 0 {
            buf.push_str("  ");
            for dx in 0..vw {
                let x = cam_x + dx as i32;
                if x % 10 == 0 {
                    buf.push_str(&format!("{}", (x / 10) % 10));
                } else {
                    buf.push(' ');
                }
            }
            buf.push('\n');
        }
        buf.push_str(&format!("{:2}", y % 100));
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            if let Some(&ch) = entity_map.get(&(dx as i32, dy as i32)) {
                buf.push(ch);
            } else if !grid.in_bounds(x, y) {
                buf.push('?');
            } else {
                let cell = grid.get(x, y);
                buf.push(cell.material.display_char());
            }
        }
        buf.push('\n');
    }
    buf
}

fn player_info(game: &Game) -> String {
    if let Some(e) = game.player.entity(&game.entities) {
        let (cx, cy) = e.center();
        let body_count = e.bodies.iter().filter(|b| b.alive).count();
        let on_fire = e.on_fire;
        let kind = entity_kind_name(e.kind);
        format!("{} hp={:.1}/{:.1} pos=({:.1},{:.1}) bodies={}/{} on_fire={}", kind, e.health, e.max_health, cx, cy, body_count, e.bodies.len(), on_fire)
    } else {
        "None".to_string()
    }
}

fn entity_kind_name(kind: entity::EntityKind) -> &'static str {
    match kind {
        entity::EntityKind::Player => "Player",
        entity::EntityKind::Goblin => "Goblin",
        entity::EntityKind::Corpse => "Corpse",
    }
}
