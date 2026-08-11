use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub position: Point,
    pub velocity: Point,
    pub radius: f32,
    pub color: Color,
    pub age_seconds: f32,
    pub lifetime_seconds: f32,
}

#[derive(Debug, Default)]
pub struct FlareEngine {
    particles: Vec<Particle>,
}

impl FlareEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn burst(&mut self, origin: Point) {
        const PARTICLE_COUNT: usize = 18;

        for index in 0..PARTICLE_COUNT {
            let angle = (index as f32 / PARTICLE_COUNT as f32) * TAU;
            let speed = 260.0 + ((index % 5) as f32 * 24.0);
            let warm = index % 3;

            self.particles.push(Particle {
                position: origin,
                velocity: Point {
                    x: angle.cos() * speed,
                    y: angle.sin() * speed,
                },
                radius: 5.0 + (index % 4) as f32,
                color: match warm {
                    0 => Color { r: 1.0, g: 0.34, b: 0.18, a: 1.0 },
                    1 => Color { r: 1.0, g: 0.78, b: 0.22, a: 1.0 },
                    _ => Color { r: 0.35, g: 0.82, b: 1.0, a: 1.0 },
                },
                age_seconds: 0.0,
                lifetime_seconds: 0.55 + (index % 4) as f32 * 0.05,
            });
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        for particle in &mut self.particles {
            particle.age_seconds += delta_seconds;
            particle.position.x += particle.velocity.x * delta_seconds;
            particle.position.y += particle.velocity.y * delta_seconds;
            particle.velocity.y += 520.0 * delta_seconds;

            let progress = particle.age_seconds / particle.lifetime_seconds;
            particle.color.a = (1.0 - progress).clamp(0.0, 1.0);
            particle.radius *= 0.985;
        }

        self.particles
            .retain(|particle| particle.age_seconds < particle.lifetime_seconds);
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_creates_particles() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 });

        assert_eq!(engine.particles().len(), 18);
    }

    #[test]
    fn particles_expire_after_lifetime() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 });
        engine.update(2.0);

        assert!(engine.particles().is_empty());
    }
}
