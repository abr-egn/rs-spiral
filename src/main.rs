use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::{Instant, Duration};

use ggez::{
        Context, ContextBuilder, GameResult,
        graphics, nalgebra as na, timer,
    };
use ggez::conf;
use ggez::event;

const TARGET_FPS: u32 = 60;
const TICK_SCALE: f32 = 1.0 / (TARGET_FPS as f32);
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / (TARGET_FPS as u64));
const STAR_DELAY: Duration = Duration::from_millis(100);
const STAR_SPEED: f32 = 10.0;
const ANGLE_ACCEL: f32 = 0.01;
const R_SCALE: f32 = 0.2;
const G_SCALE: f32 = 0.3;
const B_SCALE: f32 = 0.5;
const SEGMENT_LEN: f32 = 3.0;

fn main() {
    let (ctx, events) = &mut ContextBuilder::new("spiral", "Abraham Egnor")
        .window_setup(conf::WindowSetup {
            title: "Spiral!".to_owned(),
            samples: conf::NumSamples::Four,
            ..Default::default()
        })
        .window_mode(conf::WindowMode {
            width: 1000.0,
            height: 1000.0,
            ..Default::default()
        })
		.build()
		.expect("aieee, could not create ggez context!");
    let mut screen = graphics::screen_coordinates(ctx);
    screen.translate(na::Vector2::new(-screen.w/2.0, -screen.h/2.0));
    graphics::set_screen_coordinates(ctx, screen).unwrap();

    let mut my_game = MyGame::new(ctx).unwrap();

    match event::run(ctx, events, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

struct MyGame {
    star_mesh: graphics::Mesh,
    angle: f32,
    angle_delta: f32,
    stars: VecDeque<Star>,
    last_star: Instant,
    now: Instant,
    start: Instant,
}

impl MyGame {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let now = Instant::now();
        Ok(MyGame {
            star_mesh: graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(0.0, 0.0),
                /* radius */ 2.0,
                /* tolerance */ 0.1,
                graphics::WHITE,
            )?,
            angle: 0.0,
            angle_delta: 0.0,
            stars: VecDeque::new(),
            last_star: now,
            now: now,
            start: now,
        })
    }

    fn tick(&mut self, screen: &graphics::Rect) {
        self.now += TICK_DURATION;
        if self.now.duration_since(self.last_star) >= STAR_DELAY {
            self.last_star = self.now;
            let now_f32: f32 = timer::duration_to_f64(self.now.duration_since(self.start)) as f32;
            self.stars.push_back(Star::spawn(self.angle, now_f32));
        }
        while self.stars.front().map_or(false, |s| !screen.contains(s.pos)) {
            self.stars.pop_front();
        }
        for star in &mut self.stars {
            star.tick();
        }
        self.angle += self.angle_delta;
        if self.angle > 2.0*PI {
            self.angle -= 2.0*PI;
        }
        self.angle_delta += ANGLE_ACCEL * TICK_SCALE;
        if self.angle_delta > 2.0*PI {
            self.angle_delta -= 2.0*PI;
        }
    }
}

impl event::EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let screen = graphics::screen_coordinates(ctx);
        while timer::check_update_time(ctx, TARGET_FPS) {
            self.tick(&screen);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let stars_len = self.stars.len();
        for (ix, star) in self.stars.iter().enumerate() {
            /*
            graphics::draw(ctx,
                &self.star_mesh,
                graphics::DrawParam::new()
                    .color(star.color)
                    .dest(star.pos),
            )?;
            */
            if ix >= stars_len-1 { continue }
            let mut nearest = self.stars.back().unwrap();
            let mut nearest_dist_sqr = star.distance_sqr_to(nearest);
            for other_ix in (ix+1)..stars_len {
                let other = &self.stars[other_ix];
                let other_dist_sqr = star.distance_sqr_to(other);
                if other_dist_sqr < nearest_dist_sqr {
                    nearest = other;
                    nearest_dist_sqr = other_dist_sqr;
                }
            }
            let mut pos = star.pos;
            let pos_vec = nearest.pos - star.pos;
            let segments_f32 = (pos_vec.norm() / SEGMENT_LEN).ceil();
            let segments = segments_f32 as i32;
            let pos_delta = pos_vec / segments_f32;
            let mut color = star.color;
            let color_delta = graphics::Color {
                r: (nearest.color.r - star.color.r) / segments_f32,
                g: (nearest.color.g - star.color.g) / segments_f32,
                b: (nearest.color.b - star.color.b) / segments_f32,
                a: 1.0,
            };
            for _ in 0..segments {
                let next = pos + pos_delta;
                let line = graphics::Mesh::new_polyline(ctx,
                    graphics::DrawMode::Stroke(
                        graphics::StrokeOptions::default()
                            .with_start_cap(graphics::LineCap::Round)
                            .with_end_cap(graphics::LineCap::Round)
                            .with_line_width(4.0)
                    ),
                    &[pos, next],
                    color,
                ).unwrap();
                graphics::draw(ctx, &line, graphics::DrawParam::default())?;
                pos = next;
                color = graphics::Color {
                    r: color.r + color_delta.r,
                    g: color.g + color_delta.g,
                    b: color.b + color_delta.b,
                    a: 1.0,
                };
            }
        }
        graphics::present(ctx)?;
        Ok(())
    }
}

struct Star {
    pos: na::Point2<f32>,
    delta: na::Vector2<f32>,
    color: graphics::Color,
}

impl Star {
    fn spawn(angle: f32, now: f32) -> Self {
        let r = 0.5 + (0.5 * (now * R_SCALE).sin());
        let g = 0.5 + (0.5 * (now * G_SCALE).sin());
        let b = 0.5 + (0.5 * (now * B_SCALE).sin());
        Star {
            pos: na::Point2::new(0.0, 0.0),
            delta: na::Vector2::new(angle.cos(), angle.sin()) * STAR_SPEED,
            color: graphics::Color::new(r, g, b, 1.0),
        }
    }

    fn distance_sqr_to(&self, other: &Star) -> f32 {
        (other.pos.x - self.pos.x).powi(2) + (other.pos.y - self.pos.y).powi(2)
    }

    fn tick(&mut self) {
        self.pos += self.delta * TICK_SCALE;
    }
}
