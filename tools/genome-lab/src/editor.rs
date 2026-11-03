use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Small shared drawing primitive; coded editor modules own field selection and interaction.
pub fn render(frame: &mut Frame, area: Rect, title: &str, body: String, focused: bool) {
    let block = Block::default().borders(Borders::ALL).title(title);
    let block = if focused {
        block.border_style(Style::default().fg(Color::Yellow))
    } else {
        block
    };
    frame.render_widget(
        Paragraph::new(body).wrap(Wrap { trim: true }).block(block),
        area,
    );
}

pub fn input_character(buffer: &mut String, character: char) {
    if character.is_ascii_digit() || matches!(character, '-' | '.') {
        buffer.push(character);
    }
}

pub fn finite_number(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| "Enter a finite numeric value".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_draft_accepts_numeric_keystrokes_and_rejects_incomplete_values() {
        let mut draft = String::new();
        input_character(&mut draft, '-');
        input_character(&mut draft, '1');
        input_character(&mut draft, '.');
        input_character(&mut draft, '5');
        input_character(&mut draft, 'x');
        assert_eq!(draft, "-1.5");
        assert_eq!(finite_number(&draft).unwrap(), -1.5);
        assert!(finite_number("-").is_err());
        assert!(finite_number("NaN").is_err());
    }
}
