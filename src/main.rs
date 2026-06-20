use clap::Parser;
use verbatim::game::Game;
use verbatim::render::terminal::TerminalRenderer;
use verbatim::render::window_input::WindowInput;
use verbatim::world::cell::MaterialId;
use verbatim::ai;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(name = "verbatim", about = "ASCII physics RPG - Noita meets Caves of Qud")]
struct Cli {
    #[arg(long, default_value = "window")]
    mode: String,

    #[arg(long, default_value_t = 0)]
    headless_ticks: u32,

    #[arg(long, default_value = "scenarios")]
    scenario_dir: String,

    #[arg(long)]
    scenario: Option<String>,

    #[arg(long)]
    replay_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode.as_str() {
        "window" => {
            run_vulkan_mode();
        }
        "terminal" => {
            std::panic::set_hook(Box::new(|info| {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::style::ResetColor,
                    crossterm::cursor::Show,
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::event::DisableMouseCapture,
                );
                eprintln!("PANIC: {}", info);
            }));
            let mut renderer = TerminalRenderer::new();
            let mut game = Game::new();
            game.run(&mut renderer);
        }
        "pipe" => {
            ai::run_pipe_protocol();
        }
        "test" => {
            run_test_mode(&cli);
        }
        "replay" => {
            run_replay_mode(&cli);
        }
        "headless" => {
            if cli.headless_ticks > 0 {
                run_headless(cli.headless_ticks);
            } else {
                eprintln!("Use --headless-ticks N with --mode headless");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown mode: {}. Use terminal, pipe, test, replay, or headless.", cli.mode);
            std::process::exit(1);
        }
    }
}

fn run_test_mode(cli: &Cli) {
    if let Some(path) = &cli.scenario {
        match ai::load_scenario(path) {
            Ok(scenario) => {
                let result = ai::run_scenario(&scenario);
                let report = ai::format_results(&[result.clone()]);
                println!("{}", report);
                if !result.passed {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error loading scenario: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let results = ai::run_all_scenarios(&cli.scenario_dir);
        if results.is_empty() {
            eprintln!("No scenarios found in {}", cli.scenario_dir);
            std::process::exit(1);
        }
        let report = ai::format_results(&results);
        println!("{}", report);
        let any_failed = results.iter().any(|r| !r.passed);
        if any_failed {
            std::process::exit(1);
        }
    }
}

fn run_replay_mode(cli: &Cli) {
    let path = match &cli.replay_file {
        Some(p) => p,
        None => {
            eprintln!("Use --replay-file PATH with --mode replay");
            std::process::exit(1);
        }
    };

    match ai::ReplayPlayer::load(path) {
        Ok(player) => {
            let session = player.play();
            let state = session.get_state();
            println!("=== Replay: {} events ===", player.recording().events.len());
            println!("Final tick: {}", state.tick);
            if let Some(ref p) = state.player {
                println!("Player: {} hp={:.1}/{:.1} pos=({:.1},{:.1}) alive={}",
                    p.kind, p.health, p.max_health, p.pos[0], p.pos[1], p.alive);
            }
            println!("Entities: {}", state.entities.len());
            println!("\nFinal view:\n{}", state.view);
        }
        Err(e) => {
            eprintln!("Error loading replay: {}", e);
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
                .map(|e| format!("{}(hp={:.0}, pos={:?})", ai::entity_kind_name(e.kind), e.health, e.center()))
                .collect();
            log.push_str(&format!("Alive entities: {}\n\n", alive.join(", ")));
        }
    }

    let mut f = std::fs::File::create("headless_dump.txt").expect("Cannot create dump file");
    f.write_all(log.as_bytes()).expect("Cannot write dump");
    eprintln!("Headless run complete: {} ticks, dump written to headless_dump.txt", ticks);
}

fn dump_view(grid: &verbatim::world::grid::Grid, entities: &verbatim::entity::EntityManager, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    ai::render_view(grid, entities, cam_x, cam_y, vw, vh)
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:2}{}", (cam_y + i as i32) % 100, line))
        .collect::<Vec<_>>()
        .join("\n") + "\n"
}

fn player_info(game: &Game) -> String {
    if let Some(e) = game.player.entity(&game.entities) {
        let (cx, cy) = e.center();
        let body_count = e.bodies.iter().filter(|b| b.alive).count();
        let on_fire = e.on_fire;
        let kind = ai::entity_kind_name(e.kind);
        format!("{} hp={:.1}/{:.1} pos=({:.1},{:.1}) bodies={}/{} on_fire={}", kind, e.health, e.max_health, cx, cy, body_count, e.bodies.len(), on_fire)
    } else {
        "None".to_string()
    }
}


fn run_vulkan_mode() {
    use verbatim::render::vulkan::VulkanRenderer;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{EventLoop, ControlFlow};
    use winit::window::Window;
    use std::time::{Duration, Instant};
    use std::sync::Arc;

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop.create_window(
        Window::default_attributes()
            .with_title("Verbatim (Vulkan)")
            .with_inner_size(winit::dpi::LogicalSize::new(160 * 8, 50 * 16))
    ).expect("Failed to create window");
    let window = Arc::new(window);

    let mut renderer = match VulkanRenderer::new(Arc::clone(&window)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Vulkan init failed: {e}");
            eprintln!("Falling back to terminal mode...");
            let mut renderer = TerminalRenderer::new();
            let mut game = Game::new();
            game.run(&mut renderer);
            return;
        }
    };

    let mut game = Game::new();
    game.init_world();

    let mut input = WindowInput::new();
    let vw = renderer.grid_w();
    let vh = renderer.grid_h();

    let fixed_dt = Duration::from_millis(16);
    let mut last_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut running = true;

    event_loop.run(|event, ctrl| {
        ctrl.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        running = false;
                        ctrl.exit();
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        input.on_key_event(key_event.physical_key, key_event.state);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                if !running {
                    ctrl.exit();
                    return;
                }

                let now = Instant::now();
                let frame_time = now.duration_since(last_time);
                last_time = now;
                accumulator += frame_time;

                let mut steps = 0;
                while accumulator >= fixed_dt && steps < 5 {
                    game.fixed_update();
                    accumulator -= fixed_dt;
                    steps += 1;
                }

                input.update();

                if input.quit {
                    running = false;
                    ctrl.exit();
                    return;
                }

                if input.jump {
                    let on_ground = game.check_on_ground();
                    game.player.jump(&mut game.entities, on_ground);
                }

                if input.left {
                    game.player.move_left(&mut game.entities);
                } else if input.right {
                    game.player.move_right(&mut game.entities);
                } else {
                    game.player.stop_horizontal(&mut game.entities);
                }

                if input.cam_left { game.cam_x -= 3; }
                if input.cam_right { game.cam_x += 3; }
                if input.cam_up { game.cam_y -= 3; }
                if input.cam_down { game.cam_y += 3; }

                if let Some(brush_id) = input.paint {
                    let mat = match brush_id {
                        1 => MaterialId::Sand,
                        2 => MaterialId::Water,
                        3 => MaterialId::Stone,
                        4 => MaterialId::Lava,
                        5 => MaterialId::Wood,
                        6 => MaterialId::Acid,
                        7 => MaterialId::Grass,
                        8 => MaterialId::Dirt,
                        9 => MaterialId::Fire,
                        0 => MaterialId::Flesh,
                        99 => MaterialId::Empty,
                        _ => MaterialId::Empty,
                    };
                    let cx = game.cam_x + (vw as i32 / 2);
                    let cy = game.cam_y + (vh as i32 / 2);
                    let r = 2;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy <= r * r + 1 {
                                if mat == MaterialId::Empty {
                                    game.grid.set(cx + dx, cy + dy, verbatim::world::cell::Cell::empty());
                                } else {
                                    game.grid.set_material(cx + dx, cy + dy, mat);
                                }
                            }
                        }
                    }
                }

                let (px, py) = game.player.center(&game.entities);
                game.cam_x = px as i32 - (vw as i32 / 2);
                game.cam_y = py as i32 - (vh as i32 / 2);

                renderer.render(&game.grid, &game.entities, game.cam_x, game.cam_y);
            }
            _ => {}
        }
    }).expect("event loop error");
}
