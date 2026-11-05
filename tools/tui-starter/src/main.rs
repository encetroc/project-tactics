use std::{error::Error, io};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::Paragraph};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }

    let result =
        Terminal::new(CrosstermBackend::new(stdout)).and_then(|mut terminal| run(&mut terminal));
    let restore_result = restore_terminal();
    result?;
    restore_result?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new("Ratatui starter — press q to quit"),
                frame.area(),
            );
        })?;

        if let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            return Ok(());
        }
    }
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend, widgets::Paragraph};

    #[test]
    fn starter_screen_renders_on_test_backend() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    Paragraph::new("Ratatui starter — press q to quit"),
                    frame.area(),
                );
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let text: String = (0..buffer.area.width)
            .map(|x| buffer[(x, 0)].symbol())
            .collect();
        assert!(text.contains("Ratatui starter"));
    }
}
