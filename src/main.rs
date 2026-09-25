use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

mod braille;
mod shape;

use braille::{Framebuffer, Rgb};

const CYCLE_SECONDS: f32 = 12.0;
const FORM_END: f32 = 4.0;
const HOLD_DURATION: f32 = 3.5;
const DISSOLVE_DURATION: f32 = 2.0;
const HOLD_END: f32 = FORM_END + HOLD_DURATION;
const DISSOLVE_END: f32 = HOLD_END + DISSOLVE_DURATION;
const TAU: f32 = std::f32::consts::PI * 2.0;
const DEFAULT_SEED: u32 = 0x0A17_AA93;

const SAFFRON: Rgb = Rgb::new(1.0, 0.42, 0.06);
const GOLD: Rgb = Rgb::new(1.0, 0.78, 0.18);
const CRIMSON: Rgb = Rgb::new(0.72, 0.04, 0.08);
const INDIGO: Rgb = Rgb::new(0.22, 0.16, 0.58);

#[derive(Clone, Copy, Default)]
struct Point {
    x: f32,
    y: f32,
}

struct Particle {
    start: Point,
    target: Point,
    position: Point,
    previous: Point,
    color: Rgb,
    phase: f32,
    drift: f32,
}

struct Options {
    frame_limit: Option<u32>,
    seed: u32,
    plain: bool,
    benchmark: bool,
    version: bool,
    help: bool,
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("aavajan: {message}");
            std::process::exit(2);
        }
    };
    if options.help {
        print_help();
        return;
    }
    if options.version {
        println!("aavajan {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let result = if options.benchmark {
        run_benchmark(options.frame_limit.unwrap_or(600), options.seed)
    } else if options.plain {
        run_plain(options.frame_limit, options.seed)
    } else {
        run(options.frame_limit, options.seed)
    };
    if let Err(error) = result {
        eprintln!("aavajan: {error}");
        std::process::exit(1);
    }
}

fn run(frame_limit: Option<u32>, seed: u32) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let (columns, rows) = terminal::size().unwrap_or((80, 24));
    let mut framebuffer = Framebuffer::new(columns.clamp(40, 160), rows.clamp(12, 60));
    let mut particles = create_particles(seed);
    let started = Instant::now();
    let result = run_loop(
        &mut stdout,
        &mut framebuffer,
        &mut particles,
        started,
        frame_limit,
    );

    terminal::disable_raw_mode()?;
    execute!(
        stdout,
        cursor::Show,
        terminal::LeaveAlternateScreen,
        terminal::Clear(ClearType::All)
    )?;
    result
}

fn run_plain(frame_limit: Option<u32>, seed: u32) -> io::Result<()> {
    let (columns, rows) = terminal::size().unwrap_or((80, 24));
    let mut framebuffer = Framebuffer::new(columns.clamp(40, 160), rows.clamp(12, 60));
    let mut particles = create_particles(seed);
    let frame_count = frame_limit.unwrap_or(1);
    let start = if frame_limit.is_some() {
        0.0
    } else {
        HOLD_END + 0.5
    };
    if frame_limit.is_none() {
        for particle in &mut particles {
            particle.position = particle.target;
            particle.previous = particle.target;
        }
    }
    let mut output = io::stdout();
    for frame in 0..frame_count {
        if frame > 0 {
            output.write_all(b"\n")?;
        }
        render(
            &mut framebuffer,
            &mut particles,
            start + frame as f32 / 60.0,
        );
        output.write_all(framebuffer.plain_frame().as_bytes())?;
    }
    Ok(())
}

fn run_benchmark(frame_count: u32, seed: u32) -> io::Result<()> {
    let (columns, rows) = terminal::size().unwrap_or((80, 24));
    let mut framebuffer = Framebuffer::new(columns.clamp(40, 160), rows.clamp(12, 60));
    let mut particles = create_particles(seed);
    let started = Instant::now();
    for frame in 0..frame_count {
        render(&mut framebuffer, &mut particles, frame as f32 / 60.0);
        std::hint::black_box(framebuffer.plain_frame());
    }
    let elapsed = started.elapsed().as_secs_f32().max(0.001);
    println!(
        "benchmark: {frame_count} frames in {elapsed:.2}s ({:.1} fps)",
        frame_count as f32 / elapsed
    );
    Ok(())
}

fn run_loop<W: Write>(
    output: &mut W,
    framebuffer: &mut Framebuffer,
    particles: &mut [Particle],
    started: Instant,
    frame_limit: Option<u32>,
) -> io::Result<()> {
    let mut frame = 0u32;
    loop {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key)
                    if key.code == KeyCode::Esc
                        || key.code == KeyCode::Char('q')
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)) =>
                {
                    return Ok(());
                }
                Event::Resize(columns, rows) => {
                    framebuffer.resize(columns.clamp(40, 160), rows.clamp(12, 60));
                }
                _ => {}
            }
        }

        let frame_started = Instant::now();
        let elapsed = started.elapsed().as_secs_f32();
        render(framebuffer, particles, elapsed);
        framebuffer.present(output)?;
        frame += 1;
        if frame_limit.is_some_and(|limit| frame >= limit) {
            return Ok(());
        }

        let frame_time = frame_started.elapsed();
        let target = Duration::from_secs_f64(1.0 / 60.0);
        if frame_time < target {
            thread::sleep(target - frame_time);
        }
    }
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        frame_limit: None,
        seed: DEFAULT_SEED,
        plain: false,
        benchmark: false,
        version: false,
        help: false,
    };
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => options.help = true,
            "--version" | "-V" => options.version = true,
            "--plain" => options.plain = true,
            "--benchmark" => options.benchmark = true,
            "--seed" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--seed needs an unsigned integer".to_string())?;
                options.seed = value
                    .parse::<u32>()
                    .map_err(|_| "--seed needs an unsigned integer".to_string())?;
            }
            "--frames" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--frames needs a positive number".to_string())?;
                options.frame_limit = Some(
                    value
                        .parse::<u32>()
                        .ok()
                        .filter(|limit| *limit > 0)
                        .ok_or_else(|| "--frames needs a positive number".to_string())?,
                );
            }
            _ => return Err(format!("unknown option: {argument}")),
        }
    }
    Ok(options)
}

fn print_help() {
    println!("Aavajan - Ganesha Particle Renderer");
    println!("Usage: aavajan [--plain|--benchmark] [--frames COUNT] [--seed VALUE]");
    println!("  --plain         print a plain formed snapshot");
    println!("  --benchmark     measure render throughput without output");
    println!("  --frames COUNT  stop after COUNT rendered frames");
    println!("  --seed VALUE    use a deterministic particle seed");
    println!("  -V, --version   show the package version");
    println!("  -h, --help      show this help");
}

fn create_particles(seed: u32) -> Vec<Particle> {
    let targets = shape::targets();
    let mut seed = seed;
    let mut particles = Vec::with_capacity(targets.len());
    for target in targets {
        let angle = random(&mut seed) * TAU;
        let radius = 0.35 + random(&mut seed) * 0.8;
        let start = Point {
            x: 0.5 + angle.cos() * radius * 1.25,
            y: 0.48 + angle.sin() * radius * 0.72,
        };
        let color = if target.y < 0.32 {
            GOLD
        } else if target.x < 0.32 {
            SAFFRON
        } else if target.y > 0.68 {
            CRIMSON
        } else {
            INDIGO
        };
        particles.push(Particle {
            start,
            target: Point {
                x: target.x,
                y: target.y,
            },
            position: start,
            previous: start,
            color,
            phase: random(&mut seed) * TAU,
            drift: 0.4 + random(&mut seed) * 0.8,
        });
    }
    particles
}

fn render(framebuffer: &mut Framebuffer, particles: &mut [Particle], elapsed: f32) {
    framebuffer.clear();
    let cycle = elapsed % CYCLE_SECONDS;

    for particle in particles {
        particle.previous = particle.position;
        let energy;
        let mut radius = 1;

        if cycle < FORM_END {
            let progress = ease(cycle / FORM_END);
            let remaining = 1.0 - progress;
            particle.position.x = lerp(particle.start.x, particle.target.x, progress)
                + (remaining * 0.045 * (particle.phase + cycle * 4.0).sin()).sin();
            particle.position.y = lerp(particle.start.y, particle.target.y, progress)
                + (remaining * 0.028 * (particle.phase * 1.7 + cycle * 5.0).cos()).sin();
            energy = 0.35 + progress * 0.55;
        } else if cycle < HOLD_END {
            let local = cycle - FORM_END;
            particle.position.x =
                particle.target.x + (local * 1.7 + particle.phase).sin() * 0.006 * particle.drift;
            particle.position.y = particle.target.y
                + (local * 2.1 + particle.phase * 1.4).cos() * 0.008 * particle.drift;
            energy = 0.9;
            radius = 0;
        } else if cycle < DISSOLVE_END {
            let progress = ease((cycle - HOLD_END) / (DISSOLVE_END - HOLD_END));
            let direction_x = particle.target.x - 0.5;
            let direction_y = particle.target.y - 0.48;
            let length = (direction_x * direction_x + direction_y * direction_y)
                .sqrt()
                .max(0.001);
            let distance = 0.08 + progress * 0.95;
            let swirl = (progress * std::f32::consts::PI + particle.phase).sin() * 0.12;
            particle.position.x =
                particle.target.x + direction_x / length * distance - direction_y / length * swirl;
            particle.position.y = particle.target.y
                + direction_y / length * distance
                + direction_x / length * swirl
                + (progress * 8.0 + particle.phase).sin() * 0.025;
            energy = 0.9 * (1.0 - progress * 0.72);
        } else {
            let progress = (cycle - DISSOLVE_END) / (CYCLE_SECONDS - DISSOLVE_END);
            particle.position.x = particle.target.x + direction(particle).0 * progress * 1.4;
            particle.position.y = particle.target.y + direction(particle).1 * progress * 1.4;
            energy = 0.2 * (1.0 - progress);
        }

        framebuffer.segment_normalized(
            (particle.previous.x, particle.previous.y),
            (particle.position.x, particle.position.y),
            particle.color,
            energy,
            radius,
        );
        if (FORM_END..HOLD_END).contains(&cycle) {
            framebuffer.plot_normalized(particle.position.x, particle.position.y, GOLD, 0.65, 0);
        }
    }
}

fn direction(particle: &Particle) -> (f32, f32) {
    let x = particle.target.x - 0.5;
    let y = particle.target.y - 0.48;
    let length = (x * x + y * y).sqrt().max(0.001);
    (x / length, y / length)
}

fn random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f32 / 16_777_216.0
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

fn ease(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    1.0 - (1.0 - value).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_timing_matches_the_animation_contract() {
        assert_eq!(HOLD_END - FORM_END, 3.5);
        assert_eq!(DISSOLVE_END - HOLD_END, 2.0);
        assert_eq!(CYCLE_SECONDS - DISSOLVE_END, 2.5);
    }
}
