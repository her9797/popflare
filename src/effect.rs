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
    pub angle: f32,
    pub length: f32,
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
        const PARTICLE_COUNT: usize = 12;

        for index in 0..PARTICLE_COUNT {
            let angle = (index as f32 / PARTICLE_COUNT as f32) * TAU;
            let start_radius = 16.0 + (index % 2) as f32 * 2.0;
            let speed = 150.0 + ((index % 3) as f32 * 18.0);

            let color = match index % 6 {
                0 => Color { r: 1.0, g: 0.20, b: 0.18, a: 0.96 },
                1 => Color { r: 1.0, g: 0.64, b: 0.12, a: 0.96 },
                2 => Color { r: 1.0, g: 0.92, b: 0.22, a: 0.96 },
                3 => Color { r: 0.26, g: 0.86, b: 0.42, a: 0.96 },
                4 => Color { r: 0.24, g: 0.64, b: 1.0, a: 0.96 },
                _ => Color { r: 0.86, g: 0.32, b: 1.0, a: 0.96 },
            };

            self.particles.push(Particle {
                position: Point {
                    x: origin.x + angle.cos() * start_radius,
                    y: origin.y + angle.sin() * start_radius,
                },
                velocity: Point {
                    x: angle.cos() * speed,
                    y: angle.sin() * speed,
                },
                angle,
                length: 11.0 + (index % 3) as f32 * 2.0,
                radius: 2.4,
                color,
                age_seconds: 0.0,
                lifetime_seconds: 0.42 + (index % 3) as f32 * 0.04,
            });
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        for particle in &mut self.particles {
            particle.age_seconds += delta_seconds;
            particle.position.x += particle.velocity.x * delta_seconds;
            particle.position.y += particle.velocity.y * delta_seconds;
            particle.velocity.x *= 0.94;
            particle.velocity.y *= 0.94;

            let progress = particle.age_seconds / particle.lifetime_seconds;
            particle.color.a = (1.0 - progress).clamp(0.0, 1.0) * 0.96;
            particle.length *= 0.99;
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

        assert_eq!(engine.particles().len(), 12);
    }

    #[test]
    fn particles_expire_after_lifetime() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 });
        engine.update(2.0);

        assert!(engine.particles().is_empty());
    }
}
