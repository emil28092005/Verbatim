use crate::ai::action::AiAction;
use crate::ai::scenario::{format_results, load_scenario, run_all_scenarios, run_scenario};
use crate::ai::session::GameSession;
use crate::ai::state::GameState;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Init {
        #[serde(default)]
        seed: Option<u64>,
        #[serde(default)]
        mode: Option<String>,
    },
    Step {
        n: u32,
    },
    Action {
        action: AiAction,
        #[serde(default)]
        step: Option<u32>,
    },
    GetState {
        #[serde(default)]
        view_w: Option<usize>,
        #[serde(default)]
        view_h: Option<usize>,
    },
    GetView {
        #[serde(default)]
        w: Option<usize>,
        #[serde(default)]
        h: Option<usize>,
    },
    GetSpectrum {
        spectrum: String,
        #[serde(default)]
        w: Option<usize>,
        #[serde(default)]
        h: Option<usize>,
    },
    GetAllSpectrums {
        #[serde(default)]
        w: Option<usize>,
        #[serde(default)]
        h: Option<usize>,
    },
    GetViewAt {
        cam_x: i32,
        cam_y: i32,
        w: usize,
        h: usize,
    },
    GetCell {
        x: i32,
        y: i32,
    },
    GetRegion {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    GetEntities,
    GetPlayer,
    CountMaterial {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        material: String,
    },
    FindMaterial {
        material: String,
    },
    RecordStart,
    RecordStop,
    ReplaySave {
        path: String,
    },
    RunScenario {
        path: String,
    },
    RunAllScenarios {
        dir: String,
    },
    Quit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GameState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectrums: Option<Vec<(String, String)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<crate::ai::state::CellInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<Vec<crate::ai::state::CellInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<crate::ai::state::EntityInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<crate::ai::state::EntityInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario_result: Option<crate::ai::scenario::ScenarioResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario_results: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording: Option<bool>,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            state: None,
            view: None,
            spectrums: None,
            cell: None,
            region: None,
            entities: None,
            player: None,
            count: None,
            found: None,
            scenario_result: None,
            scenario_results: None,
            recording: None,
        }
    }

    pub fn err(msg: &str) -> Self {
        Self {
            ok: false,
            error: Some(msg.to_string()),
            state: None,
            view: None,
            spectrums: None,
            cell: None,
            region: None,
            entities: None,
            player: None,
            count: None,
            found: None,
            scenario_result: None,
            scenario_results: None,
            recording: None,
        }
    }

    pub fn with_state(mut self, s: GameState) -> Self {
        self.state = Some(s);
        self
    }
}

pub fn run_pipe_protocol() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut session: Option<GameSession> = None;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let cmd: Command = match serde_json::from_str(line) {
            Ok(c) => c,
            Err(e) => {
                let resp = Response::err(&format!("Parse error: {}", e));
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&resp).unwrap_or_default()
                )
                .ok();
                stdout.flush().ok();
                continue;
            }
        };

        let response = handle_command(cmd, &mut session);
        let json = serde_json::to_string(&response).unwrap_or_default();
        writeln!(stdout, "{}", json).ok();
        stdout.flush().ok();

        if session.is_none() && response.ok {
            break;
        }
    }
}

fn handle_command(cmd: Command, session: &mut Option<GameSession>) -> Response {
    match cmd {
        Command::Init { seed, mode } => {
            let mut s = match seed {
                Some(seed) => GameSession::new_seeded(seed),
                None => GameSession::new(),
            };
            match mode.as_deref().unwrap_or("world") {
                "empty" => s.init_empty(),
                "world" | _ => s.init(),
            }
            let state = s.get_state();
            *session = Some(s);
            Response::ok().with_state(state)
        }

        Command::Step { n } => {
            let s = match session.as_mut() {
                Some(s) => s,
                None => return Response::err("No session. Send {\"cmd\":\"init\"} first."),
            };
            s.step(n);
            let state = s.get_state();
            Response::ok().with_state(state)
        }

        Command::Action { action, step } => {
            let s = match session.as_mut() {
                Some(s) => s,
                None => return Response::err("No session. Send {\"cmd\":\"init\"} first."),
            };
            s.perform_action(&action);
            if let Some(n) = step {
                s.step(n);
            }
            let state = s.get_state();
            Response::ok().with_state(state)
        }

        Command::GetState { view_w, view_h } => {
            let s = match session.as_mut() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            if let (Some(w), Some(h)) = (view_w, view_h) {
                s.view_width = w;
                s.view_height = h;
            }
            let state = s.get_state();
            Response::ok().with_state(state)
        }

        Command::GetView { w, h } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let vw = w.unwrap_or(80);
            let vh = h.unwrap_or(25);
            let view = s.get_view(vw, vh);
            Response {
                view: Some(view),
                ..Response::ok()
            }
        }

        Command::GetSpectrum { spectrum, w, h } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let vw = w.unwrap_or(80);
            let vh = h.unwrap_or(25);
            let spec = match spectrum.as_str() {
                "materials" => crate::ai::spectrum::Spectrum::Materials,
                "temperature" | "temp" => crate::ai::spectrum::Spectrum::Temperature,
                "light" => crate::ai::spectrum::Spectrum::Light,
                "entities" => crate::ai::spectrum::Spectrum::Entities,
                "density" => crate::ai::spectrum::Spectrum::Density,
                "velocity" => crate::ai::spectrum::Spectrum::Velocity,
                _ => return Response::err("Unknown spectrum. Use: materials, temperature, light, entities, density, velocity"),
            };
            let view = s.get_spectrum(&spec, vw, vh);
            Response {
                spectrums: Some(vec![(spectrum, view)]),
                ..Response::ok()
            }
        }

        Command::GetAllSpectrums { w, h } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let vw = w.unwrap_or(80);
            let vh = h.unwrap_or(25);
            let spectrums = s.get_all_spectrums(vw, vh);
            Response {
                spectrums: Some(spectrums),
                ..Response::ok()
            }
        }

        Command::GetViewAt { cam_x, cam_y, w, h } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let view = s.get_view_at(cam_x, cam_y, w, h);
            Response {
                view: Some(view),
                ..Response::ok()
            }
        }

        Command::GetCell { x, y } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let cell = s.get_cell(x, y);
            Response {
                cell: Some(cell),
                ..Response::ok()
            }
        }

        Command::GetRegion { x, y, w, h } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let region = s.get_region(x, y, w, h);
            Response {
                region: Some(region),
                ..Response::ok()
            }
        }

        Command::GetEntities => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let entities = s.get_entities();
            Response {
                entities: Some(entities),
                ..Response::ok()
            }
        }

        Command::GetPlayer => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let player = s.get_player();
            Response {
                player,
                ..Response::ok()
            }
        }

        Command::CountMaterial {
            x,
            y,
            w,
            h,
            material,
        } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let count = s.count_material_in_region(x, y, w, h, &material);
            Response {
                count: Some(count),
                ..Response::ok()
            }
        }

        Command::FindMaterial { material } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            let found = s.find_material(&material);
            Response {
                found,
                ..Response::ok()
            }
        }

        Command::RecordStart => {
            let s = match session.as_mut() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            s.set_recording(true);
            Response {
                recording: Some(true),
                ..Response::ok()
            }
        }

        Command::RecordStop => {
            let s = match session.as_mut() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            s.set_recording(false);
            Response {
                recording: Some(false),
                ..Response::ok()
            }
        }

        Command::ReplaySave { path } => {
            let s = match session.as_ref() {
                Some(s) => s,
                None => return Response::err("No session."),
            };
            match s.save_replay(&path) {
                Ok(()) => Response::ok(),
                Err(e) => Response::err(&format!("Save error: {}", e)),
            }
        }

        Command::RunScenario { path } => match load_scenario(&path) {
            Ok(scenario) => {
                let result = run_scenario(&scenario);
                Response {
                    scenario_result: Some(result),
                    ..Response::ok()
                }
            }
            Err(e) => Response::err(&e),
        },

        Command::RunAllScenarios { dir } => {
            let results = run_all_scenarios(&dir);
            let report = format_results(&results);
            Response {
                scenario_results: Some(report),
                ..Response::ok()
            }
        }

        Command::Quit => {
            *session = None;
            Response::ok()
        }
    }
}
