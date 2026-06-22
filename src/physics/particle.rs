#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: u32,
    pub max_life: u32,
    pub color: [u8; 4],
    pub size: f32,
    pub gravity: f32,
}

impl Particle {
    pub fn alive(&self) -> bool {
        self.life > 0
    }
}

pub struct ParticleManager {
    particles: Vec<Particle>,
    capacity: usize,
}

impl ParticleManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn spawn(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        life: u32,
        color: [u8; 4],
        size: f32,
        gravity: f32,
    ) {
        if self.particles.len() >= self.capacity {
            return;
        }
        self.particles.push(Particle {
            x,
            y,
            vx,
            vy,
            life,
            max_life: life,
            color,
            size,
            gravity,
        });
    }

    pub fn spawn_burst(
        &mut self,
        x: f32,
        y: f32,
        count: usize,
        color: [u8; 4],
        speed: f32,
        life: u32,
        size: f32,
        gravity: f32,
    ) {
        let mut rng = 0x12345u32;
        for _ in 0..count {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            let angle = (rng as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let s = speed * (0.3 + 0.7 * ((rng >> 16) as f32 / u16::MAX as f32));
            let vx = angle.cos() * s;
            let vy = angle.sin() * s - speed * 0.3;
            self.spawn(x, y, vx, vy, life, color, size, gravity);
        }
    }

    pub fn update(&mut self) {
        for p in &mut self.particles {
            p.life = p.life.saturating_sub(1);
            p.vy += p.gravity;
            p.x += p.vx;
            p.y += p.vy;
            p.vx *= 0.96;
            p.vy *= 0.96;
        }
        self.particles.retain(|p| p.alive());
    }

    pub fn all(&self) -> &[Particle] {
        &self.particles
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for ParticleManager {
    fn default() -> Self {
        Self::new(2000)
    }
}
