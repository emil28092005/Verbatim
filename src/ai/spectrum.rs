use crate::entity::{EntityKind, EntityManager};
use crate::world::chunked_grid::ChunkedGrid;

pub enum Spectrum {
    Materials,
    Temperature,
    Light,
    Entities,
    Density,
    Velocity,
    Gas,
    Pressure,
}

impl Spectrum {
    pub fn name(&self) -> &'static str {
        match self {
            Spectrum::Materials => "materials",
            Spectrum::Temperature => "temperature",
            Spectrum::Light => "light",
            Spectrum::Entities => "entities",
            Spectrum::Density => "density",
            Spectrum::Velocity => "velocity",
            Spectrum::Gas => "gas",
            Spectrum::Pressure => "pressure",
        }
    }

    pub fn all() -> &'static [Spectrum] {
        &[
            Spectrum::Materials,
            Spectrum::Temperature,
            Spectrum::Light,
            Spectrum::Entities,
            Spectrum::Density,
            Spectrum::Velocity,
            Spectrum::Gas,
            Spectrum::Pressure,
        ]
    }
}

pub fn render_spectrum(
    spectrum: &Spectrum,
    grid: &ChunkedGrid,
    entities: &EntityManager,
    light: Option<&crate::render::lighting::LightGrid>,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    match spectrum {
        Spectrum::Materials => render_materials(grid, entities, cam_x, cam_y, vw, vh),
        Spectrum::Temperature => render_temperature(grid, cam_x, cam_y, vw, vh),
        Spectrum::Light => render_light(grid, entities, light, cam_x, cam_y, vw, vh),
        Spectrum::Entities => render_entities(grid, entities, cam_x, cam_y, vw, vh),
        Spectrum::Density => render_density(grid, cam_x, cam_y, vw, vh),
        Spectrum::Velocity => render_velocity(grid, entities, cam_x, cam_y, vw, vh),
        Spectrum::Gas => render_gas(grid, cam_x, cam_y, vw, vh),
        Spectrum::Pressure => render_pressure(grid, cam_x, cam_y, vw, vh),
    }
}

fn render_materials(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let mut entity_map = std::collections::HashMap::new();
    for e in entities.all() {
        for b in &e.bodies {
            if !b.alive {
                continue;
            }
            let sx = b.x as i32 - cam_x;
            let sy = b.y as i32 - cam_y;
            if sx >= 0 && sx < vw as i32 && sy >= 0 && sy < vh as i32 {
                let ch = match e.kind {
                    EntityKind::Player if e.alive => '@',
                    EntityKind::Goblin if e.alive => 'g',
                    EntityKind::Slime if e.alive => 's',
                    _ => '%',
                };
                entity_map.insert((sx, sy), ch);
            }
        }
    }

    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if let Some(&ch) = entity_map.get(&(dx as i32, dy as i32)) {
                buf.push(ch);
            } else if !grid.in_bounds(x, y) {
                buf.push('?');
            } else {
                buf.push(grid.get(x, y).material.display_char());
            }
        }
        buf.push('\n');
    }
    buf
}

fn render_temperature(grid: &ChunkedGrid, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if !grid.in_bounds(x, y) {
                buf.push(' ');
            } else {
                let cell = grid.get(x, y);
                if cell.is_empty() {
                    buf.push(' ');
                } else {
                    let t = grid.get_temp(x, y);
                    let ch = if t < 0.0 {
                        '.'
                    } else if t < 20.0 {
                        '-'
                    } else if t < 50.0 {
                        '='
                    } else if t < 100.0 {
                        '+'
                    } else if t < 200.0 {
                        'o'
                    } else if t < 400.0 {
                        'x'
                    } else if t < 800.0 {
                        'X'
                    } else {
                        '#'
                    };
                    buf.push(ch);
                }
            }
        }
        buf.push('\n');
    }
    buf
}

fn render_light(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    light: Option<&crate::render::lighting::LightGrid>,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let ambient = crate::render::lighting::ambient_light();
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            let (r, g, b) = if let Some(lg) = light {
                let c = lg.get(dx as i32, dy as i32);
                (c[0], c[1], c[2])
            } else if grid.in_bounds(x, y) {
                let wl = grid.get_light(x, y);
                if wl[0] > 0 || wl[1] > 0 || wl[2] > 0 {
                    (wl[0], wl[1], wl[2])
                } else {
                    (ambient[0], ambient[1], ambient[2])
                }
            } else {
                (ambient[0], ambient[1], ambient[2])
            };
            let brightness = (r as u32 + g as u32 + b as u32) / 3;
            let ch = if !grid.in_bounds(x, y) {
                ' '
            } else if brightness < 30 {
                ' '
            } else if brightness < 60 {
                '.'
            } else if brightness < 90 {
                ':'
            } else if brightness < 120 {
                '+'
            } else if brightness < 160 {
                'o'
            } else if brightness < 200 {
                'O'
            } else {
                '*'
            };
            buf.push(ch);
        }
        buf.push('\n');
    }
    let _ = entities;
    buf
}

fn render_entities(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            let mut found = ' ';
            for e in entities.all() {
                for b in &e.bodies {
                    if !b.alive {
                        continue;
                    }
                    let bx = b.x as i32;
                    let by = b.y as i32;
                    if bx == x && by == y {
                        found = match e.kind {
                            EntityKind::Player if e.alive => '@',
                            EntityKind::Goblin if e.alive => 'g',
                            EntityKind::Slime if e.alive => 's',
                            EntityKind::Player => '@',
                            _ => '%',
                        };
                        break;
                    }
                }
                if found != ' ' {
                    break;
                }
            }
            if found == ' ' && grid.in_bounds(x, y) {
                let cell = grid.get(x, y);
                if !cell.is_empty() {
                    found = '.';
                }
            }
            buf.push(found);
        }
        buf.push('\n');
    }
    buf
}

fn render_density(grid: &ChunkedGrid, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    let reg = crate::world::material::MaterialRegistry::instance();
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if !grid.in_bounds(x, y) {
                buf.push(' ');
            } else {
                let cell = grid.get(x, y);
                if cell.is_empty() {
                    buf.push(' ');
                } else {
                    let mat = reg.get(cell.material);
                    let d = mat.density;
                    let ch = if d < 0.5 {
                        '.'
                    } else if d < 1.0 {
                        ':'
                    } else if d < 2.0 {
                        '+'
                    } else if d < 5.0 {
                        'o'
                    } else if d < 10.0 {
                        'O'
                    } else {
                        '#'
                    };
                    buf.push(ch);
                }
            }
        }
        buf.push('\n');
    }
    buf
}

fn render_velocity(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            let mut ch = ' ';
            for e in entities.all() {
                if !e.alive {
                    continue;
                }
                let vx = e.cvx;
                let vy = e.cvy;
                let speed = (vx * vx + vy * vy).sqrt();
                let (ex, ey) = e.center();
                let sx = ex as i32 - cam_x;
                let sy = ey as i32 - cam_y;
                if sx == dx as i32 && sy == dy as i32 {
                    ch = if speed < 0.1 {
                        '.'
                    } else if speed < 0.5 {
                        ':'
                    } else if speed < 1.0 {
                        '+'
                    } else if speed < 2.0 {
                        'o'
                    } else {
                        'O'
                    };
                    break;
                }
            }
            if ch == ' ' && grid.in_bounds(x, y) {
                let cell = grid.get(x, y);
                if cell.updated_this_tick {
                    ch = ',';
                }
            }
            buf.push(ch);
        }
        buf.push('\n');
    }
    buf
}

pub fn render_all_spectrums(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    light: Option<&crate::render::lighting::LightGrid>,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> Vec<(String, String)> {
    Spectrum::all()
        .iter()
        .map(|s| {
            (
                s.name().to_string(),
                render_spectrum(s, grid, entities, light, cam_x, cam_y, vw, vh),
            )
        })
        .collect()
}

pub fn format_all_spectrums(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    light: Option<&crate::render::lighting::LightGrid>,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let spectrums = render_all_spectrums(grid, entities, light, cam_x, cam_y, vw, vh);
    let mut out = String::new();
    for (name, view) in &spectrums {
        out.push_str(&format!("--- {} ---\n", name));
        out.push_str(view);
        out.push('\n');
    }
    out
}

fn render_gas(grid: &ChunkedGrid, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if !grid.in_bounds(x, y) {
                buf.push(' ');
            } else {
                let (gt, gd) = grid.get_gas(x, y);
                let ch = if gd == 0 {
                    ' '
                } else {
                    match gt {
                        1 => '.', // smoke
                        2 => 'x', // poison
                        3 => 'o', // CO2
                        4 => '~', // steam
                        _ => '?',
                    }
                };
                buf.push(ch);
            }
        }
        buf.push('\n');
    }
    buf
}

fn render_pressure(grid: &ChunkedGrid, cam_x: i32, cam_y: i32, vw: usize, vh: usize) -> String {
    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if !grid.in_bounds(x, y) {
                buf.push(' ');
            } else {
                let p = grid.get_pressure(x, y);
                let ch = if p < 80 {
                    'L'
                } else if p < 120 {
                    '-'
                } else if p < 140 {
                    '.'
                } else if p < 180 {
                    '+'
                } else if p < 220 {
                    'o'
                } else {
                    '#'
                };
                buf.push(ch);
            }
        }
        buf.push('\n');
    }
    buf
}
