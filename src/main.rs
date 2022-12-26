use crossterm::{
    event::{self, Event as CEvent, KeyCode, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    error,
    sync::mpsc,
    time::{Duration, Instant},
    thread
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs, Widget},
    Frame, Terminal,
};


enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    enable_raw_mode().expect("terminal can run in raw mode");

    let (tx, rx) = mpsc::channel(); 
    let tickrate = Duration::from_millis(100);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tickrate 
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("events poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events over mpsc channel");
                }
            }

            if last_tick.elapsed() >= tickrate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    loop {
        // render ui
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                    ].as_ref(),
                )
                .split(size);

            let main = Block::default()
                .style(Style::default().fg(Color::Gray))
                .borders(Borders::ALL)
                .title(
                    format!("[ trinitty v{} ]", option_env!("CARGO_PKG_VERSION")
                        .expect("can read environment variable CARGO_PKG_VERSION")
                    )
                )
                .border_type(BorderType::Plain);
            rect.render_widget(main, chunks[0]);
        });

        // handle events
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                    terminal.show_cursor()?;
                },
                // TODO more keycodes
                _ => {}
            },
            Event::Tick => {},
        }
    }



    Ok(())
}

