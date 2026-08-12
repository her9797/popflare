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
pub enum EffectStyle {
    ColorBurst,
    ColorRings,
    PinkSparkles,
    ColorSparkles,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRingBurst {
    pub origin: Point,
    pub scale: f32,
    pub opacity: f32,
    pub age_seconds: f32,
    pub lifetime_seconds: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SparkleKind {
    Plus,
    Diamond,
    Star,
    Twinkle,
    Dot,
    Asterisk,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sparkle {
    pub position: Point,
    pub velocity: Point,
    pub kind: SparkleKind,
    pub size: f32,
    pub rotation: f32,
    pub color: Color,
    pub age_seconds: f32,
    pub lifetime_seconds: f32,
}

#[derive(Debug, Default)]
pub struct FlareEngine {
    particles: Vec<Particle>,
    color_rings: Vec<ColorRingBurst>,
    sparkles: Vec<Sparkle>,
}

impl FlareEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn burst(&mut self, origin: Point, style: EffectStyle) {
        match style {
            EffectStyle::ColorBurst => self.color_burst(origin),
            EffectStyle::ColorRings => self.spawn_color_rings(origin),
            EffectStyle::PinkSparkles => self.spawn_pink_sparkles(origin),
            EffectStyle::ColorSparkles => self.spawn_color_sparkles(origin),
        }
    }

    fn color_burst(&mut self, origin: Point) {
        const PARTICLE_COUNT: usize = 12;

        for index in 0..PARTICLE_COUNT {
            let angle = (index as f32 / PARTICLE_COUNT as f32) * TAU;
            let start_radius = 11.0 + (index % 2) as f32 * 1.5;
            let speed = 105.0 + ((index % 3) as f32 * 12.0);

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
                length: 8.0 + (index % 3) as f32 * 1.5,
                radius: 1.8,
                color,
                age_seconds: 0.0,
                lifetime_seconds: 0.42 + (index % 3) as f32 * 0.04,
            });
        }
    }

    fn spawn_color_rings(&mut self, origin: Point) {
        self.color_rings.push(ColorRingBurst {
            origin,
            scale: 0.38,
            opacity: 0.95,
            age_seconds: 0.0,
            lifetime_seconds: 0.55,
        });
    }

    fn spawn_pink_sparkles(&mut self, origin: Point) {
        let pink = Color { r: 1.0, g: 0.48, b: 0.67, a: 0.96 };
        let soft_pink = Color { r: 1.0, g: 0.58, b: 0.74, a: 0.92 };
        let specs = [
            (16.0, -10.0, SparkleKind::Twinkle, 12.0, 0.18, pink),
            (-15.0, 13.0, SparkleKind::Twinkle, 10.5, -0.28, pink),
            (3.0, 17.0, SparkleKind::Twinkle, 11.0, 0.36, pink),
            (-7.0, -16.0, SparkleKind::Twinkle, 8.5, 0.10, soft_pink),
            (-23.0, -6.0, SparkleKind::Twinkle, 5.5, -0.08, soft_pink),
            (22.0, 12.0, SparkleKind::Twinkle, 6.0, 0.24, soft_pink),
            (13.0, -22.0, SparkleKind::Twinkle, 5.0, -0.20, soft_pink),
            (-20.0, 22.0, SparkleKind::Twinkle, 4.8, 0.16, soft_pink),
        ];

        self.spawn_sparkles(origin, &specs);
    }

    fn spawn_color_sparkles(&mut self, origin: Point) {
        let yellow = Color { r: 0.96, g: 0.72, b: 0.08, a: 0.95 };
        let lavender = Color { r: 0.62, g: 0.66, b: 1.0, a: 0.95 };
        let mint = Color { r: 0.56, g: 0.78, b: 0.44, a: 0.95 };
        let cyan = Color { r: 0.45, g: 0.84, b: 0.90, a: 0.95 };
        let specs = [
            (0.0, 0.0, SparkleKind::Star, 10.5, 0.00, yellow),
            (-19.0, -12.0, SparkleKind::Plus, 8.0, -0.04, lavender),
            (20.0, 9.0, SparkleKind::Plus, 8.0, 0.02, lavender),
            (-14.0, 17.0, SparkleKind::Asterisk, 7.6, 0.16, mint),
            (14.0, -15.0, SparkleKind::Asterisk, 7.4, -0.18, mint),
            (22.0, -13.0, SparkleKind::Diamond, 6.5, 0.10, yellow),
            (-23.0, 1.0, SparkleKind::Dot, 2.5, 0.00, cyan),
            (14.0, 19.0, SparkleKind::Dot, 2.8, 0.00, cyan),
            (4.0, 21.0, SparkleKind::Dot, 2.5, 0.00, cyan),
            (-6.0, -22.0, SparkleKind::Dot, 2.4, 0.00, cyan),
        ];

        self.spawn_sparkles(origin, &specs);
    }

    fn spawn_sparkles(
        &mut self,
        origin: Point,
        specs: &[(f32, f32, SparkleKind, f32, f32, Color)],
    ) {
        for (index, (offset_x, offset_y, kind, size, rotation, color)) in specs.iter().copied().enumerate() {
            let angle = (index as f32 / specs.len() as f32) * TAU;
            self.sparkles.push(Sparkle {
                position: Point {
                    x: origin.x + offset_x,
                    y: origin.y + offset_y,
                },
                velocity: Point {
                    x: angle.cos() * 15.0,
                    y: angle.sin() * 15.0,
                },
                kind,
                size,
                rotation,
                color,
                age_seconds: 0.0,
                lifetime_seconds: 0.68 + (index % 3) as f32 * 0.06,
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

        for ring in &mut self.color_rings {
            ring.age_seconds += delta_seconds;
            let progress = ring.age_seconds / ring.lifetime_seconds;
            ring.scale = 0.38 + progress * 0.10;
            ring.opacity = (1.0 - progress).clamp(0.0, 1.0) * 0.95;
        }

        for sparkle in &mut self.sparkles {
            sparkle.age_seconds += delta_seconds;
            sparkle.position.x += sparkle.velocity.x * delta_seconds;
            sparkle.position.y += sparkle.velocity.y * delta_seconds;
            sparkle.velocity.x *= 0.96;
            sparkle.velocity.y *= 0.96;

            let progress = sparkle.age_seconds / sparkle.lifetime_seconds;
            sparkle.color.a = (1.0 - progress).clamp(0.0, 1.0) * 0.95;
            sparkle.size *= 0.992;
            sparkle.rotation += 0.05;
        }

        self.particles
            .retain(|particle| particle.age_seconds < particle.lifetime_seconds);
        self.color_rings
            .retain(|ring| ring.age_seconds < ring.lifetime_seconds);
        self.sparkles
            .retain(|sparkle| sparkle.age_seconds < sparkle.lifetime_seconds);
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn color_rings(&self) -> &[ColorRingBurst] {
        &self.color_rings
    }

    pub fn sparkles(&self) -> &[Sparkle] {
        &self.sparkles
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_burst_creates_particles() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorBurst);

        assert_eq!(engine.particles().len(), 12);
        assert!(engine.color_rings().is_empty());
        assert!(engine.sparkles().is_empty());
    }

    #[test]
    fn color_rings_creates_ring_effect() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorRings);

        assert_eq!(engine.color_rings().len(), 1);
        assert!(engine.particles().is_empty());
        assert!(engine.sparkles().is_empty());
    }

    #[test]
    fn pink_sparkles_creates_sparkle_effect() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::PinkSparkles);

        assert_eq!(engine.sparkles().len(), 8);
        assert!(engine.particles().is_empty());
        assert!(engine.color_rings().is_empty());
    }

    #[test]
    fn color_sparkles_creates_sparkle_effect() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorSparkles);

        assert_eq!(engine.sparkles().len(), 10);
        assert!(engine.particles().is_empty());
        assert!(engine.color_rings().is_empty());
    }

    #[test]
    fn effects_expire_after_lifetime() {
        let mut engine = FlareEngine::new();
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorBurst);
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorRings);
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::PinkSparkles);
        engine.burst(Point { x: 100.0, y: 200.0 }, EffectStyle::ColorSparkles);
        engine.update(2.0);

        assert!(engine.particles().is_empty());
        assert!(engine.color_rings().is_empty());
        assert!(engine.sparkles().is_empty());
    }
}
