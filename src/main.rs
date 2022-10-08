use chip_8_emulator::chip_8::{self, Chip8};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs::File;
use std::io::{self, Read};
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Style},
    text::Span,
    widgets::{canvas::Canvas, Block, Borders},
    Frame, Terminal,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg()]
    file_path: String,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let mut file = File::options()
        .read(true)
        .create(false)
        .open(args.file_path)?;
    let mut file_contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_contents)?;

    let mut chip_8 = Chip8::start(&file_contents[..]);

    enable_raw_mode()?;
    let mut output = io::stdout();
    execute!(output, EnterAlternateScreen, EnableMouseCapture)?;
    let crossterm = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(crossterm)?;

    let result = run_app(&mut terminal, &mut chip_8);

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;

    result
}

#[allow(unused_must_use)]
fn run_app<B>(terminal: &mut Terminal<B>, chip8: &mut Chip8) -> io::Result<()>
where
    B: Backend,
{
    let mut previous_instant = Instant::now();
    let mut opcodes_per_cycle: u8 = 8;
    let mut timer_hz: u8 = 60;
    let mut screen_heigth: usize = chip_8::SCREEN_HEIGHT;
    let mut screen_width: usize = chip_8::SCREEN_WIDTH;

    //keyboard events, updating the timers, executing opcode, drawing
    loop {
        //there is a little bit of "loss of time" since the method "from_millis" accepts u64 values only
        thread::sleep(Duration::from_millis(
            ((1.0 / timer_hz as f64) * 1000.0) as u64,
        ));
        chip8.update();

        for _i in 0..opcodes_per_cycle {
            if event::poll(Duration::from_micros(1))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        //chip8 commands
                        KeyCode::Char('w') => chip8.set_key(5),
                        KeyCode::Char('a') => chip8.set_key(7),
                        KeyCode::Char('s') => chip8.set_key(8),
                        KeyCode::Char('d') => chip8.set_key(9),
                        KeyCode::Char('q') => chip8.set_key(4),
                        KeyCode::Char('e') => chip8.set_key(6),
                        KeyCode::Char('1') => chip8.set_key(1),
                        KeyCode::Char('2') => chip8.set_key(2),
                        KeyCode::Char('3') => chip8.set_key(3),
                        KeyCode::Char('4') => chip8.set_key(12),
                        KeyCode::Char('x') => chip8.set_key(0),
                        KeyCode::Char('z') => chip8.set_key(10),
                        KeyCode::Char('c') => chip8.set_key(11),
                        KeyCode::Char('r') => chip8.set_key(13),
                        KeyCode::Char('f') => chip8.set_key(14),
                        KeyCode::Char('v') => chip8.set_key(15),
                        //new emulator commands
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Up => {
                            opcodes_per_cycle += if opcodes_per_cycle < u8::MAX { 1 } else { 0 };
                            Ok(())
                        }
                        KeyCode::Down => {
                            opcodes_per_cycle -= if opcodes_per_cycle > 1 { 1 } else { 0 };
                            Ok(())
                        }
                        KeyCode::Right => {
                            timer_hz += if timer_hz < u8::MAX { 1 } else { 0 };
                            Ok(())
                        }
                        KeyCode::Left => {
                            timer_hz -= if timer_hz > 1 { 1 } else { 0 };
                            Ok(())
                        }
                        KeyCode::Char('l') => {
                            screen_heigth = if screen_heigth == chip_8::SCREEN_HEIGHT {
                                terminal.size().unwrap().height as usize
                            } else {
                                chip_8::SCREEN_HEIGHT
                            };
                            screen_width = if screen_width == chip_8::SCREEN_WIDTH {
                                terminal.size().unwrap().width as usize
                            } else {
                                chip_8::SCREEN_WIDTH
                            };
                            Ok(())
                        }
                        _ => Ok(()),
                    };
                }
            }

            chip8.execute_next_opcode();

            terminal
                .draw(|f| {
                    ui(
                        f,
                        &chip8,
                        opcodes_per_cycle,
                        timer_hz,
                        screen_heigth,
                        screen_width,
                    )
                })
                .expect("it was not possible to draw anymore");

            if (Instant::now() - previous_instant).as_millis() >= 16 {
                previous_instant = Instant::now();
            }
        }
    }
}

fn ui<B>(
    f: &mut Frame<B>,
    chip8: &Chip8,
    opcodes_per_cycle: u8,
    timer_hz: u8,
    screen_height: usize,
    screen_width: usize,
) where
    B: Backend,
{
    let title = format!(
        "Chip8|opcodes per cycle:{}|timer between cpu cycles:{} hz",
        opcodes_per_cycle, timer_hz
    );
    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.to_owned()),
        )
        .paint(|ctx| {
            for y in 0..chip_8::SCREEN_HEIGHT {
                for x in 0..chip_8::SCREEN_WIDTH {
                    if chip8.get_pixel(x, y) == 1 {
                        ctx.print(
                            x as f64,
                            (screen_height - y) as f64,
                            Span::styled("█", Style::default().fg(Color::White)),
                        );
                    }
                }
            }
        })
        .x_bounds([0.0, screen_width as f64])
        .y_bounds([0.0, screen_height as f64]);

    f.render_widget(canvas, f.size());
}
