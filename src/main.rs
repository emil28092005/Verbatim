use clap::Parser;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};
use verbatim::ai;
use verbatim::game::Game;
use verbatim::render::lighting;
use verbatim::render::terminal::TerminalRenderer;
use verbatim::render::window_input::WindowInput;
use verbatim::world::cell::MaterialId;
use verbatim::world::chunked_grid::ChunkedGrid;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

#[derive(Parser, Debug)]
#[command(
    name = "verbatim",
    about = "ASCII physics RPG - Noita meets Caves of Qud"
)]
struct Cli {
    #[arg(long, default_value = "ascii")]
    mode: String,

    #[arg(long, default_value_t = 0)]
    headless_ticks: u32,

    #[arg(long, default_value = "scenarios")]
    scenario_dir: String,

    #[arg(long)]
    scenario: Option<String>,

    #[arg(long)]
    replay_file: Option<String>,

    #[arg(long, default_value_t = 600)]
    benchmark_ticks: u32,

    #[arg(long, default_value = "graphics")]
    benchmark_renderer: String,

    #[arg(long, default_value = "benchmark_results.json")]
    benchmark_output: String,

    #[arg(long, default_value = "surface")]
    benchmark_biome: String,

    #[arg(long, default_value_t = 5)]
    tape_interval: u32,

    #[arg(long, default_value = "tape.txt")]
    tape_output: String,

    #[arg(long)]
    tape_json: Option<String>,
}

trait GpuRenderer {
    fn new(window: Arc<Window>) -> Result<Self, String>
    where
        Self: Sized;
    fn render(
        &mut self,
        grid: &ChunkedGrid,
        entities: &verbatim::entity::EntityManager,
        items: &verbatim::entity::item::ItemManager,
        ui: &verbatim::ui::UiLayer,
        cam_x: i32,
        cam_y: i32,
        lighting: Option<&lighting::LightGrid>,
    );
    fn grid_w(&self) -> usize;
    fn grid_h(&self) -> usize;
    fn cell_pixel_size(&self) -> u32;
    fn upload_particles(&mut self, particles: &verbatim::physics::particle::ParticleManager);
}

impl GpuRenderer for verbatim::render::vulkan::VulkanRenderer {
    fn new(window: Arc<Window>) -> Result<Self, String> {
        verbatim::render::vulkan::VulkanRenderer::new(window)
    }
    fn render(
        &mut self,
        grid: &ChunkedGrid,
        entities: &verbatim::entity::EntityManager,
        items: &verbatim::entity::item::ItemManager,
        ui: &verbatim::ui::UiLayer,
        cam_x: i32,
        cam_y: i32,
        lighting: Option<&lighting::LightGrid>,
    ) {
        verbatim::render::vulkan::VulkanRenderer::render(
            self, grid, entities, items, ui, cam_x, cam_y, lighting,
        )
    }
    fn grid_w(&self) -> usize {
        verbatim::render::vulkan::VulkanRenderer::grid_w(self)
    }
    fn grid_h(&self) -> usize {
        verbatim::render::vulkan::VulkanRenderer::grid_h(self)
    }
    fn cell_pixel_size(&self) -> u32 {
        verbatim::render::vulkan::VulkanRenderer::cell_pixel_size(self)
    }
    fn upload_particles(&mut self, particles: &verbatim::physics::particle::ParticleManager) {
        verbatim::render::vulkan::VulkanRenderer::upload_particles(self, particles);
    }
}

impl GpuRenderer for verbatim::render::graphics::GraphicsRenderer {
    fn new(window: Arc<Window>) -> Result<Self, String> {
        verbatim::render::graphics::GraphicsRenderer::new(window)
    }
    fn render(
        &mut self,
        grid: &ChunkedGrid,
        entities: &verbatim::entity::EntityManager,
        items: &verbatim::entity::item::ItemManager,
        ui: &verbatim::ui::UiLayer,
        cam_x: i32,
        cam_y: i32,
        lighting: Option<&lighting::LightGrid>,
    ) {
        verbatim::render::graphics::GraphicsRenderer::render(
            self, grid, entities, items, ui, cam_x, cam_y, lighting,
        )
    }
    fn grid_w(&self) -> usize {
        verbatim::render::graphics::GraphicsRenderer::grid_w(self)
    }
    fn grid_h(&self) -> usize {
        verbatim::render::graphics::GraphicsRenderer::grid_h(self)
    }
    fn cell_pixel_size(&self) -> u32 {
        verbatim::render::graphics::GraphicsRenderer::cell_pixel_size(self)
    }
    fn upload_particles(&mut self, particles: &verbatim::physics::particle::ParticleManager) {
        verbatim::render::graphics::GraphicsRenderer::upload_particles(self, particles);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.mode.as_str() {
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
            let mut game = Game::new_random();
            game.run(&mut renderer);
        }
        "ascii" => {
            run_gpu_mode::<verbatim::render::vulkan::VulkanRenderer>("Verbatim — ASCII");
        }
        "graphics" => {
            run_gpu_mode::<verbatim::render::graphics::GraphicsRenderer>("Verbatim — Graphics");
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
        "capture" => {
            if cli.headless_ticks > 0 {
                run_capture(cli.headless_ticks);
            } else {
                eprintln!("Use --headless-ticks N with --mode capture");
                std::process::exit(1);
            }
        }
        "benchmark" => {
            run_benchmark_mode(&cli);
        }
        "tape" => {
            if cli.headless_ticks > 0 {
                ai::run_tape_mode(
                    cli.headless_ticks,
                    cli.tape_interval,
                    &cli.tape_output,
                    cli.tape_json.as_deref(),
                );
            } else {
                eprintln!("Use --headless-ticks N with --mode tape");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!(
                "Unknown mode: {}. Use terminal, ascii, graphics, pipe, test, replay, headless, or capture.",
                cli.mode
            );
            std::process::exit(1);
        }
    }
}

fn run_gpu_mode<R: GpuRenderer>(title: &str) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(1600, 900)),
        )
        .expect("Failed to create window");
    let window = Arc::new(window);

    let mut renderer = match R::new(Arc::clone(&window)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Vulkan init failed: {e}");
            eprintln!("Falling back to terminal mode...");
            let mut renderer = TerminalRenderer::new();
            let mut game = Game::new_random();
            game.run(&mut renderer);
            return;
        }
    };

    let mut game = Game::new_random();
    game.init_world();

    let mut input = WindowInput::new();

    let fixed_dt = Duration::from_millis(16);
    let mut last_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut running = true;
    let target_frame_time = Duration::from_nanos(1_000_000_000 / 60);
    let mut frame_count = 0u32;
    let mut frame_time_acc = Duration::ZERO;
    let mut last_fps_print = Instant::now();

    event_loop
        .run(|event, ctrl| {
            ctrl.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        running = false;
                        ctrl.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } => {
                        input.on_key_event(key_event.physical_key, key_event.state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        input.on_mouse_move(position.x, position.y);
                        let cell_px = renderer.cell_pixel_size() as f64;
                        let vw = renderer.grid_w();
                        let vh = renderer.grid_h();
                        game.mouse_ui_x =
                            (position.x / cell_px * verbatim::ui::UI_SCALE as f64) as i32;
                        game.mouse_ui_y =
                            (position.y / cell_px * verbatim::ui::UI_SCALE as f64) as i32;
                        game.show_tooltip = true;
                        game.show_crosshair = true;
                        if vw > 0 && vh > 0 && cell_px > 0.0 {
                            let wx = game.cam_x + (position.x / cell_px) as i32;
                            let wy = game.cam_y + (position.y / cell_px) as i32;
                            game.mouse_world_pos = (wx, wy);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        input.on_mouse_button(button, state);
                    }
                    WindowEvent::Focused(false) => {
                        input.clear_keys();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if !running {
                        ctrl.exit();
                        return;
                    }

                    let vw = renderer.grid_w();
                    let vh = renderer.grid_h();

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

                    if input.toggle_audio {
                        game.audio.toggle();
                    }

                    if input.jump {
                        let on_ground = game.check_on_ground();
                        if on_ground {
                            game.audio.play("jump");
                        }
                        game.player.jump(&mut game.entities, on_ground);
                    }

                    if input.left {
                        game.player.move_left(&mut game.entities);
                    } else if input.right {
                        game.player.move_right(&mut game.entities);
                    } else {
                        game.player.stop_horizontal(&mut game.entities);
                    }

                    if (input.shoot_mouse && !game.inventory_open)
                        || input.shoot_left
                        || input.shoot_right
                        || input.shoot_up
                        || input.shoot_down
                    {
                        let cell_px = renderer.cell_pixel_size() as f64;
                        let (px, py) = game.player.center(&game.entities);
                        let player_screen_x =
                            ((px as i32 - game.cam_x) as f64) * cell_px + cell_px / 2.0;
                        let player_screen_y =
                            ((py as i32 - game.cam_y) as f64) * cell_px + cell_px / 2.0;

                        let (dx, dy) = if input.shoot_mouse {
                            let mx = input.mouse_x - player_screen_x;
                            let my = input.mouse_y - player_screen_y;
                            let len = (mx * mx + my * my).sqrt();
                            if len < 1.0 {
                                (1.0, 0.0)
                            } else {
                                (mx / len, my / len)
                            }
                        } else if input.shoot_left {
                            (-1.0, 0.0)
                        } else if input.shoot_right {
                            (1.0, 0.0)
                        } else if input.shoot_up {
                            (0.0, -1.0)
                        } else {
                            (0.0, 1.0)
                        };

                        game.player_shoot(dx as f32, dy as f32);
                    }
                    if input.toggle_fireball {
                        game.fireball_mode = !game.fireball_mode;
                    }
                    if input.descend {
                        game.descend();
                    }
                    if input.use_item {
                        game.use_item(0);
                    }
                    if input.drop_item {
                        game.drop_item(0);
                    }

                    if input.inventory_toggle {
                        game.inventory_open = !game.inventory_open;
                    }

                    let ui_scale = verbatim::ui::UI_SCALE as f64;
                    let cell_px = 8.0_f64;
                    let ui_cell_px = cell_px / ui_scale;
                    game.inventory_mouse_x = (input.mouse_x / ui_cell_px) as i32;
                    game.inventory_mouse_y = (input.mouse_y / ui_cell_px) as i32;

                    if game.inventory_open {
                        if let Some((mx, my)) = &input.inventory_click {
                            let mouse_ui_x = (*mx / ui_cell_px) as i32;
                            let mouse_ui_y = (*my / ui_cell_px) as i32;
                            let vw = renderer.grid_w();
                            let vh = renderer.grid_h();
                            let ui_w = (vw as i32) * verbatim::ui::UI_SCALE;
                            let ui_h = (vh as i32) * verbatim::ui::UI_SCALE;

                            let cols = 4i32;
                            let rows = 2i32;
                            let slot_w = 12i32;
                            let slot_h = 10i32;
                            let gap = 3i32;
                            let panel_w = cols * slot_w + (cols + 1) * gap + 4;
                            let panel_h = rows * slot_h + (rows + 1) * gap + 28;
                            let px_start = (ui_w - panel_w) / 2;
                            let py_start = (ui_h - panel_h) / 2;

                            for row in 0..rows {
                                for col in 0..cols {
                                    let idx = (row * cols + col) as usize;
                                    let sx = px_start + gap + col * (slot_w + gap) + 2;
                                    let sy = py_start + 14 + row * (slot_h + gap) + gap;
                                    if mouse_ui_x >= sx
                                        && mouse_ui_x < sx + slot_w
                                        && mouse_ui_y >= sy
                                        && mouse_ui_y < sy + slot_h
                                    {
                                        if idx < game.player.inventory.len() {
                                            game.use_item(idx);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some((mx, my)) = &input.inventory_right_click {
                            let mouse_ui_x = (*mx / ui_cell_px) as i32;
                            let mouse_ui_y = (*my / ui_cell_px) as i32;
                            let vw = renderer.grid_w();
                            let vh = renderer.grid_h();
                            let ui_w = (vw as i32) * verbatim::ui::UI_SCALE;
                            let ui_h = (vh as i32) * verbatim::ui::UI_SCALE;

                            let cols = 4i32;
                            let rows = 2i32;
                            let slot_w = 12i32;
                            let slot_h = 10i32;
                            let gap = 3i32;
                            let panel_w = cols * slot_w + (cols + 1) * gap + 4;
                            let panel_h = rows * slot_h + (rows + 1) * gap + 28;
                            let px_start = (ui_w - panel_w) / 2;
                            let py_start = (ui_h - panel_h) / 2;

                            for row in 0..rows {
                                for col in 0..cols {
                                    let idx = (row * cols + col) as usize;
                                    let sx = px_start + gap + col * (slot_w + gap) + 2;
                                    let sy = py_start + 14 + row * (slot_h + gap) + gap;
                                    if mouse_ui_x >= sx
                                        && mouse_ui_x < sx + slot_w
                                        && mouse_ui_y >= sy
                                        && mouse_ui_y < sy + slot_h
                                    {
                                        if idx < game.player.inventory.len() {
                                            game.drop_item(idx);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    {
                        let (px, _py) = game.player.center(&game.entities);
                        let player_screen_x = ((px as i32 - game.cam_x) as f64) * 8.0 + 4.0;
                        let should_face_right = input.mouse_x >= player_screen_x;
                        if should_face_right != game.player.facing_right {
                            if let Some(e) = game.player.entity_mut(&mut game.entities) {
                                e.flip_facing();
                            }
                            game.player.facing_right = should_face_right;
                        }
                    }

                    if input.cam_left {
                        game.cam_offset_x -= 3;
                    }
                    if input.cam_right {
                        game.cam_offset_x += 3;
                    }
                    if input.cam_up {
                        game.cam_offset_y -= 3;
                    }
                    if input.cam_down {
                        game.cam_offset_y += 3;
                    }

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
                                        game.grid.set(
                                            cx + dx,
                                            cy + dy,
                                            verbatim::world::cell::Cell::empty(),
                                        );
                                    } else {
                                        game.grid.set_material(cx + dx, cy + dy, mat);
                                    }
                                }
                            }
                        }
                    }

                    let (px, py) = game.player.center(&game.entities);
                    game.cam_x = px as i32 - (vw as i32 / 2) + game.cam_offset_x;
                    game.cam_y = py as i32 - (vh as i32 / 2) + game.cam_offset_y;

                    game.build_ui(vw, vh);

                    renderer.upload_particles(&game.particles);
                    renderer.render(
                        &game.grid,
                        &game.entities,
                        &game.items,
                        &game.ui,
                        game.cam_x,
                        game.cam_y,
                        None,
                    );

                    let elapsed = Instant::now().duration_since(last_time);
                    if elapsed < target_frame_time {
                        std::thread::sleep(target_frame_time - elapsed);
                    }

                    let frame_time = Instant::now().duration_since(last_time);
                    frame_time_acc += frame_time;
                    frame_count += 1;
                    if last_fps_print.elapsed() >= Duration::from_secs(1) {
                        let avg_ms = frame_time_acc.as_secs_f32() * 1000.0 / frame_count as f32;
                        game.fps = 1000.0 / avg_ms;
                        frame_count = 0;
                        frame_time_acc = Duration::ZERO;
                        last_fps_print = Instant::now();
                    }
                }
                _ => {}
            }
        })
        .expect("event loop error");
}

fn run_benchmark_mode(cli: &Cli) {
    let ticks = cli.benchmark_ticks;
    let renderer_type = cli.benchmark_renderer.as_str();
    let output_path = cli.benchmark_output.as_str();
    let biome = cli.benchmark_biome.as_str();

    eprintln!(
        "Benchmark: {} ticks, renderer={}, biome={}",
        ticks, renderer_type, biome
    );

    match renderer_type {
        "ascii" => run_benchmark_inner::<verbatim::render::vulkan::VulkanRenderer>(
            ticks,
            output_path,
            "ascii",
            biome,
        ),
        "graphics" => run_benchmark_inner::<verbatim::render::graphics::GraphicsRenderer>(
            ticks,
            output_path,
            "graphics",
            biome,
        ),
        _ => {
            eprintln!(
                "Unknown benchmark renderer: {}. Use ascii or graphics.",
                renderer_type
            );
            std::process::exit(1);
        }
    }
}

fn run_benchmark_inner<R: GpuRenderer>(
    ticks: u32,
    output_path: &str,
    mode_name: &str,
    biome: &str,
) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Verbatim — Benchmark")
                .with_inner_size(winit::dpi::LogicalSize::new(1600, 900)),
        )
        .expect("Failed to create window");
    let window = Arc::new(window);

    let mut renderer = match R::new(Arc::clone(&window)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Vulkan init failed: {e}");
            std::process::exit(1);
        }
    };

    let mut game = Game::new_random();
    game.init_world();

    let chunk_size = game.grid.chunk_size as i32;
    let (px, _py) = game.player.center(&game.entities);
    match biome {
        "caves" => {
            let cave_y = 2 * chunk_size + chunk_size / 2;
            game.player
                .set_position(&mut game.entities, px, cave_y as f32);
        }
        "dungeon" => {
            let dungeon_y = 6 * chunk_size + chunk_size / 2;
            game.player
                .set_position(&mut game.entities, px, dungeon_y as f32);
        }
        _ => {}
    }

    let mut tick_count = 0u32;
    let mut ca_times_us: Vec<u64> = Vec::with_capacity(ticks as usize);
    let mut render_times_us: Vec<u64> = Vec::with_capacity(ticks as usize);
    let mut frame_times_us: Vec<u64> = Vec::with_capacity(ticks as usize);
    let benchmark_start = Instant::now();

    event_loop
        .run(|event, ctrl| {
            ctrl.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    ctrl.exit();
                }
                Event::AboutToWait => {
                    if tick_count >= ticks {
                        let total_elapsed = benchmark_start.elapsed();

                        let ca_avg_us = ca_times_us.iter().sum::<u64>() / ca_times_us.len() as u64;
                        let render_avg_us =
                            render_times_us.iter().sum::<u64>() / render_times_us.len() as u64;
                        let frame_avg_us =
                            frame_times_us.iter().sum::<u64>() / frame_times_us.len() as u64;

                        let ca_p99_us = percentile(&ca_times_us, 99);
                        let render_p99_us = percentile(&render_times_us, 99);
                        let frame_p99_us = percentile(&frame_times_us, 99);

                        let ca_min_us = *ca_times_us.iter().min().unwrap_or(&0);
                        let render_min_us = *render_times_us.iter().min().unwrap_or(&0);
                        let frame_min_us = *frame_times_us.iter().min().unwrap_or(&0);

                        let total_ms = total_elapsed.as_secs_f64() * 1000.0;
                        let avg_fps = ticks as f64 / (total_ms / 1000.0);
                        let avg_frame_ms = frame_avg_us as f64 / 1000.0;
                        let p99_frame_ms = frame_p99_us as f64 / 1000.0;
                        let min_frame_ms = frame_min_us as f64 / 1000.0;

                        let json = format!(
                            r#"{{
  "mode": "{}",
  "ticks": {},
  "total_time_ms": {:.1},
  "avg_fps": {:.1},
  "avg_frame_time_ms": {:.2},
  "p99_frame_time_ms": {:.2},
  "min_frame_time_ms": {:.2},
  "subsystems": {{
    "ca_step_avg_us": {},
    "ca_step_p99_us": {},
    "ca_step_min_us": {},
    "render_avg_us": {},
    "render_p99_us": {},
    "render_min_us": {}
  }}
}}"#,
                            mode_name,
                            ticks,
                            total_ms,
                            avg_fps,
                            avg_frame_ms,
                            p99_frame_ms,
                            min_frame_ms,
                            ca_avg_us,
                            ca_p99_us,
                            ca_min_us,
                            render_avg_us,
                            render_p99_us,
                            render_min_us,
                        );

                        let mut f = std::fs::File::create(output_path)
                            .expect("Cannot create benchmark output");
                        f.write_all(json.as_bytes())
                            .expect("Cannot write benchmark output");

                        eprintln!("=== Benchmark Results ===");
                        eprintln!("Mode:       {}", mode_name);
                        eprintln!("Ticks:      {}", ticks);
                        eprintln!("Total time: {:.1} ms", total_ms);
                        eprintln!("Avg FPS:    {:.1}", avg_fps);
                        eprintln!("Avg frame:  {:.2} ms", avg_frame_ms);
                        eprintln!("P99 frame:  {:.2} ms", p99_frame_ms);
                        eprintln!("Min frame:  {:.2} ms", min_frame_ms);
                        eprintln!(
                            "CA step:    avg={}us p99={}us min={}us",
                            ca_avg_us, ca_p99_us, ca_min_us
                        );
                        eprintln!(
                            "Render:     avg={}us p99={}us min={}us",
                            render_avg_us, render_p99_us, render_min_us
                        );
                        eprintln!("Results written to {}", output_path);

                        ctrl.exit();
                        return;
                    }

                    let vw = renderer.grid_w();
                    let vh = renderer.grid_h();

                    let frame_start = Instant::now();

                    let ca_start = Instant::now();
                    game.fixed_update();
                    let ca_elapsed = ca_start.elapsed();

                    let (px, py) = game.player.center(&game.entities);
                    game.cam_x = px as i32 - (vw as i32 / 2);
                    game.cam_y = py as i32 - (vh as i32 / 2);
                    game.build_ui(vw, vh);

                    let render_start = Instant::now();
                    renderer.render(
                        &game.grid,
                        &game.entities,
                        &game.items,
                        &game.ui,
                        game.cam_x,
                        game.cam_y,
                        None,
                    );
                    let render_elapsed = render_start.elapsed();

                    let frame_elapsed = frame_start.elapsed();

                    ca_times_us.push(ca_elapsed.as_micros() as u64);
                    render_times_us.push(render_elapsed.as_micros() as u64);
                    frame_times_us.push(frame_elapsed.as_micros() as u64);

                    tick_count += 1;
                }
                _ => {}
            }
        })
        .expect("event loop error");
}

fn percentile(sorted_data: &[u64], p: u64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    let mut data: Vec<u64> = sorted_data.to_vec();
    data.sort_unstable();
    let idx = (data.len() * p as usize) / 100;
    data[idx.min(data.len() - 1)]
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
                println!(
                    "Player: {} hp={:.1}/{:.1} pos=({:.1},{:.1}) alive={}",
                    p.kind, p.health, p.max_health, p.pos[0], p.pos[1], p.alive
                );
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
    log.push_str(&format!(
        "=== Verbatim Headless Run: {} ticks ===\n\n",
        ticks
    ));
    log.push_str(&format!(
        "World: {}x{}\n",
        game.grid.width, game.grid.height
    ));
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

            let alive: Vec<_> = game
                .entities
                .all()
                .iter()
                .filter(|e| e.alive)
                .map(|e| {
                    format!(
                        "{}(hp={:.0}, pos={:?})",
                        ai::entity_kind_name(e.kind),
                        e.health,
                        e.center()
                    )
                })
                .collect();
            log.push_str(&format!("Alive entities: {}\n\n", alive.join(", ")));
        }
    }

    let mut f = std::fs::File::create("headless_dump.txt").expect("Cannot create dump file");
    f.write_all(log.as_bytes()).expect("Cannot write dump");
    eprintln!(
        "Headless run complete: {} ticks, dump written to headless_dump.txt",
        ticks
    );
}

fn run_capture(ticks: u32) {
    let mut game = Game::new();
    game.init_world();

    for _ in 0..ticks {
        game.fixed_update();
    }

    let (px, py) = game.player.center(&game.entities);
    let view_w = (1600 / verbatim::render::capture::CELL_SIZE).min(256);
    let view_h = (900 / verbatim::render::capture::CELL_SIZE).min(256);
    let cam_x = px as i32 - (view_w as i32 / 2);
    let cam_y = py as i32 - (view_h as i32 / 2);

    game.build_ui(view_w as usize, view_h as usize);

    let light = lighting::compute_lighting(
        &game.grid,
        cam_x,
        cam_y,
        view_w as usize,
        view_h as usize,
        lighting::ambient_light(),
    );

    let path = "capture.png";
    match verbatim::render::capture::save_capture(
        path,
        &game.grid,
        &game.entities,
        &game.items,
        &game.ui,
        cam_x,
        cam_y,
        view_w,
        view_h,
        Some(&light),
    ) {
        Ok(_) => eprintln!(
            "Capture complete: {} ticks, image written to {}",
            ticks, path
        ),
        Err(e) => {
            eprintln!("Capture failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn dump_view(
    grid: &ChunkedGrid,
    entities: &verbatim::entity::EntityManager,
    cam_x: i32,
    cam_y: i32,
    w: usize,
    h: usize,
) -> String {
    ai::render_view(grid, entities, cam_x, cam_y, w, h)
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:2}{}", (cam_y + i as i32) % 100, line))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn player_info(game: &Game) -> String {
    if let Some(e) = game.player.entity(&game.entities) {
        let (cx, cy) = e.center();
        let body_count = e.bodies.iter().filter(|b| b.alive).count();
        let on_fire = e.on_fire;
        let kind = ai::entity_kind_name(e.kind);
        format!(
            "{} hp={:.1}/{:.1} pos=({:.1},{:.1}) bodies={}/{} on_fire={}",
            kind,
            e.health,
            e.max_health,
            cx,
            cy,
            body_count,
            e.bodies.len(),
            on_fire
        )
    } else {
        "None".to_string()
    }
}
