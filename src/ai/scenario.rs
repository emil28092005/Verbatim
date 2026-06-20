use serde::{Deserialize, Serialize};
use crate::ai::action::AiAction;
use crate::ai::session::GameSession;
use crate::ai::state::GameState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub seed: u64,
    pub init_mode: String,
    pub setup: Vec<AiAction>,
    pub steps: u32,
    pub assertions: Vec<Assertion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    CellIs {
        x: i32,
        y: i32,
        material: String,
    },
    CellIsNot {
        x: i32,
        y: i32,
        material: String,
    },
    CellTempGreaterThan {
        x: i32,
        y: i32,
        temp: f32,
    },
    CellTempLessThan {
        x: i32,
        y: i32,
        temp: f32,
    },
    NoMaterialInRegion {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        material: String,
    },
    MaterialCountInRegion {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        material: String,
        min: usize,
        max: usize,
    },
    EntityAlive {
        id: u32,
    },
    EntityDead {
        id: u32,
    },
    EntityHealthLessThan {
        id: u32,
        health: f32,
    },
    EntityOnFire {
        id: u32,
    },
    PlayerOnGround,
    PlayerAlive,
    PlayerDead,
    TickEquals {
        tick: u64,
    },
    Custom {
        description: String,
        check: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub final_state: Option<GameState>,
    pub error: Option<String>,
}

pub fn run_scenario(scenario: &Scenario) -> ScenarioResult {
    let mut session = GameSession::new_seeded(scenario.seed);

    match scenario.init_mode.as_str() {
        "empty" => session.init_empty(),
        "world" | _ => session.init(),
    }

    for action in &scenario.setup {
        session.perform_action(action);
    }

    if scenario.steps > 0 {
        session.step(scenario.steps);
    }

    let mut results = Vec::new();
    for assertion in &scenario.assertions {
        let result = check_assertion(&session, assertion);
        results.push(result);
    }

    let all_passed = results.iter().all(|r| r.passed);
    let final_state = session.get_state();

    ScenarioResult {
        name: scenario.name.clone(),
        passed: all_passed,
        assertions: results,
        final_state: Some(final_state),
        error: None,
    }
}

fn check_assertion(session: &GameSession, assertion: &Assertion) -> AssertionResult {
    match assertion {
        Assertion::CellIs { x, y, material } => {
            let cell = session.get_cell(*x, *y);
            let passed = cell.material == *material.to_lowercase();
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Cell({},{}) = '{}', expected '{}'", x, y, cell.material, material),
            }
        }
        Assertion::CellIsNot { x, y, material } => {
            let cell = session.get_cell(*x, *y);
            let passed = cell.material != *material.to_lowercase();
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Cell({},{}) = '{}', expected NOT '{}'", x, y, cell.material, material),
            }
        }
        Assertion::CellTempGreaterThan { x, y, temp } => {
            let cell = session.get_cell(*x, *y);
            let passed = cell.temp > *temp;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Cell({},{}) temp = {:.1}, expected > {:.1}", x, y, cell.temp, temp),
            }
        }
        Assertion::CellTempLessThan { x, y, temp } => {
            let cell = session.get_cell(*x, *y);
            let passed = cell.temp < *temp;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Cell({},{}) temp = {:.1}, expected < {:.1}", x, y, cell.temp, temp),
            }
        }
        Assertion::NoMaterialInRegion { x, y, w, h, material } => {
            let count = session.count_material_in_region(*x, *y, *w, *h, material);
            let passed = count == 0;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Region({},{},{},{}) has {} '{}' cells, expected 0", x, y, w, h, count, material),
            }
        }
        Assertion::MaterialCountInRegion { x, y, w, h, material, min, max } => {
            let count = session.count_material_in_region(*x, *y, *w, *h, material);
            let passed = count >= *min && count <= *max;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Region({},{},{},{}) has {} '{}' cells, expected {}-{}", x, y, w, h, count, material, min, max),
            }
        }
        Assertion::EntityAlive { id } => {
            let entities = session.get_entities();
            let entity = entities.iter().find(|e| e.id == *id);
            let passed = entity.map(|e| e.alive).unwrap_or(false);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Entity({}) alive = {}", id, if passed { "true" } else { "false/not found" }),
            }
        }
        Assertion::EntityDead { id } => {
            let entities = session.get_entities();
            let entity = entities.iter().find(|e| e.id == *id);
            let passed = entity.map(|e| !e.alive).unwrap_or(true);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Entity({}) dead = {}", id, if passed { "true" } else { "false" }),
            }
        }
        Assertion::EntityHealthLessThan { id, health } => {
            let entities = session.get_entities();
            let entity = entities.iter().find(|e| e.id == *id);
            let passed = entity.map(|e| e.health < *health).unwrap_or(false);
            let actual = entity.map(|e| e.health).unwrap_or(0.0);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Entity({}) health = {:.1}, expected < {:.1}", id, actual, health),
            }
        }
        Assertion::EntityOnFire { id } => {
            let entities = session.get_entities();
            let entity = entities.iter().find(|e| e.id == *id);
            let passed = entity.map(|e| e.on_fire).unwrap_or(false);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Entity({}) on_fire = {}", id, passed),
            }
        }
        Assertion::PlayerOnGround => {
            let passed = session.game.check_on_ground();
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Player on_ground = {}", passed),
            }
        }
        Assertion::PlayerAlive => {
            let player = session.get_player();
            let passed = player.map(|p| p.alive).unwrap_or(false);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Player alive = {}", passed),
            }
        }
        Assertion::PlayerDead => {
            let player = session.get_player();
            let passed = player.map(|p| !p.alive).unwrap_or(true);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Player dead = {}", passed),
            }
        }
        Assertion::TickEquals { tick } => {
            let passed = session.tick() == *tick;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: format!("Tick = {}, expected {}", session.tick(), tick),
            }
        }
        Assertion::Custom { description, check } => {
            AssertionResult {
                assertion: assertion.clone(),
                passed: false,
                message: format!("Custom check '{}' not implemented: {}", check, description),
            }
        }
    }
}

pub fn load_scenario(path: &str) -> Result<Scenario, String> {
    let data = std::fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path, e))?;
    serde_json::from_str(&data).map_err(|e| format!("Cannot parse {}: {}", path, e))
}

pub fn load_scenarios_from_dir(dir: &str) -> Vec<Scenario> {
    let mut scenarios = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(s) = load_scenario(path.to_str().unwrap_or("")) {
                    scenarios.push(s);
                }
            }
        }
    }
    scenarios
}

pub fn run_all_scenarios(dir: &str) -> Vec<ScenarioResult> {
    let scenarios = load_scenarios_from_dir(dir);
    scenarios.iter().map(|s| run_scenario(s)).collect()
}

pub fn format_results(results: &[ScenarioResult]) -> String {
    let mut out = String::new();
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();

    out.push_str(&format!("=== Scenario Results: {}/{} passed ===\n\n", passed, total));

    for r in results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!("[{}] {} - {} assertions\n", status, r.name, r.assertions.len()));
        if !r.passed {
            for a in &r.assertions {
                if !a.passed {
                    out.push_str(&format!("  FAIL: {}\n", a.message));
                }
            }
        }
    }

    out
}
