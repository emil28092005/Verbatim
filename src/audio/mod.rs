use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

pub struct AudioEngine {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sounds: HashMap<&'static str, &'static [u8]>,
    last_played: HashMap<&'static str, Instant>,
    volume: f32,
    enabled: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Audio init failed: {}", e);
                return Self::disabled();
            }
        };

        let mut sounds = HashMap::new();
        sounds.insert(
            "jump",
            include_bytes!("../../assets/sounds/jump.wav") as &[u8],
        );
        sounds.insert("shoot", include_bytes!("../../assets/sounds/shoot.wav"));
        sounds.insert("hit", include_bytes!("../../assets/sounds/hit.wav"));
        sounds.insert(
            "explosion",
            include_bytes!("../../assets/sounds/explosion.wav"),
        );
        sounds.insert("death", include_bytes!("../../assets/sounds/death.wav"));
        sounds.insert("pickup", include_bytes!("../../assets/sounds/pickup.wav"));
        sounds.insert("descend", include_bytes!("../../assets/sounds/descend.wav"));
        sounds.insert("powerup", include_bytes!("../../assets/sounds/powerup.wav"));
        sounds.insert("step", include_bytes!("../../assets/sounds/step.wav"));
        sounds.insert(
            "lava_bubble",
            include_bytes!("../../assets/sounds/lava_bubble.wav"),
        );
        sounds.insert(
            "acid_sizzle",
            include_bytes!("../../assets/sounds/acid_sizzle.wav"),
        );
        sounds.insert(
            "water_splash",
            include_bytes!("../../assets/sounds/water_splash.wav"),
        );
        sounds.insert(
            "fire_crackle",
            include_bytes!("../../assets/sounds/fire_crackle.wav"),
        );
        sounds.insert(
            "ui_click",
            include_bytes!("../../assets/sounds/ui_click.wav"),
        );
        sounds.insert(
            "goblin_growl",
            include_bytes!("../../assets/sounds/goblin_growl.wav"),
        );

        Self {
            _stream: Some(stream),
            handle: Some(handle),
            sounds,
            last_played: HashMap::new(),
            volume: 0.5,
            enabled: true,
        }
    }

    fn disabled() -> Self {
        Self {
            _stream: None,
            handle: None,
            sounds: HashMap::new(),
            last_played: HashMap::new(),
            volume: 0.0,
            enabled: false,
        }
    }

    pub fn play(&mut self, name: &'static str) {
        self.play_throttled(name, 50);
    }

    pub fn play_throttled(&mut self, name: &'static str, min_interval_ms: u64) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        if let Some(&last) = self.last_played.get(name) {
            if now.duration_since(last).as_millis() < min_interval_ms as u128 {
                return;
            }
        }
        self.last_played.insert(name, now);

        let data = match self.sounds.get(name) {
            Some(d) => *d,
            None => return,
        };
        let handle = match &self.handle {
            Some(h) => h,
            None => return,
        };
        let sink = match Sink::try_new(handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        if let Ok(decoder) = rodio::Decoder::new(Cursor::new(data.to_vec())) {
            let source = decoder.amplify(self.volume).convert_samples::<f32>();
            sink.append(source);
            sink.detach();
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}
