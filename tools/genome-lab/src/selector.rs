use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Render a selector whose entries each occupy exactly one terminal row.
pub fn render_selector(
    frame: &mut Frame,
    area: Rect,
    block: Block<'_>,
    items: Vec<ListItem<'_>>,
    state: &mut ListState,
) {
    let count = items.len();
    let inner = block.inner(area);
    let visible_rows = usize::from(inner.height);
    let max_offset = count.saturating_sub(visible_rows);
    // Shrinking lists and growing terminals must not leave avoidable blank rows.
    *state.offset_mut() = state.offset().min(max_offset);
    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("▶ ")
            .block(block),
        area,
        state,
    );
    if max_offset == 0 || inner.is_empty() {
        return;
    }

    // Ratatui 0.29 treats content_length - 1 as the maximum scroll position.
    // Use the number of possible window offsets, not the number of choices.
    let mut scrollbar = ScrollbarState::new(max_offset + 1)
        .viewport_content_length(visible_rows)
        .position(state.offset());
    // Replace only the right border's interior, preserving the title and corners.
    let track = Rect::new(area.right().saturating_sub(1), inner.y, 1, inner.height);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█"),
        track,
        &mut scrollbar,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, widgets::Borders};

    fn draw(count: usize, selected: usize, height: u16, state: &mut ListState) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(24, height)).unwrap();
        state.select((count > 0).then_some(selected));
        terminal
            .draw(|frame| {
                render_selector(
                    frame,
                    frame.area(),
                    Block::default().borders(Borders::ALL).title("Choices"),
                    (0..count)
                        .map(|i| ListItem::new(format!("Choice {i}")))
                        .collect(),
                    state,
                );
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn track(buffer: &Buffer) -> String {
        (1..buffer.area.height.saturating_sub(1))
            .map(|y| buffer[(23, y)].symbol())
            .collect()
    }

    #[test]
    fn selection_moves_without_scrolling_until_it_leaves_the_window() {
        let mut state = ListState::default();
        let initial_track = track(&draw(10, 0, 7, &mut state));
        assert!(initial_track.starts_with('█'));
        assert!(initial_track.ends_with('│'));
        for selected in 1..5 {
            assert_eq!(track(&draw(10, selected, 7, &mut state)), initial_track);
            assert_eq!(state.offset(), 0);
        }
        let scrolled = draw(10, 5, 7, &mut state);
        assert_eq!(state.offset(), 1);
        assert_ne!(track(&scrolled), initial_track);
        assert_eq!(
            scrolled[(3, 5)].symbol(),
            "C",
            "selected last row remains visible"
        );
        let bottom_track = track(&draw(10, 9, 7, &mut state));
        assert_eq!(state.offset(), 5);
        assert!(bottom_track.starts_with('│'));
        assert!(bottom_track.ends_with('█'), "thumb must reach bottom");
        // Moving upwards within the same window should not scroll either.
        for selected in (5..9).rev() {
            assert_eq!(track(&draw(10, selected, 7, &mut state)), bottom_track);
            assert_eq!(state.offset(), 5);
        }
        draw(10, 4, 7, &mut state);
        assert_eq!(state.offset(), 4);
        // Wrap back to the top and then to the bottom.
        assert_eq!(track(&draw(10, 0, 7, &mut state)), initial_track);
        assert_eq!(track(&draw(10, 9, 7, &mut state)), bottom_track);
    }

    #[test]
    fn one_hidden_choice_shows_real_overflow_without_overwriting_corners() {
        let mut state = ListState::default();
        let top = draw(6, 0, 7, &mut state);
        assert_eq!(track(&top), "████│");
        assert_eq!(top[(23, 0)].symbol(), "┐");
        assert_eq!(top[(23, 6)].symbol(), "┘");
        let bottom = draw(6, 5, 7, &mut state);
        assert_eq!(track(&bottom), "│████");
        assert_eq!(state.offset(), 1);
    }

    #[test]
    fn resizing_and_list_shrink_recompute_the_viewport_and_remove_unneeded_scrollbars() {
        let mut state = ListState::default();
        draw(10, 9, 7, &mut state);
        assert_eq!(state.offset(), 5);
        // More rows: all choices now fit, including those above the selection.
        assert_eq!(track(&draw(10, 9, 12, &mut state)), "│".repeat(10));
        assert_eq!(state.offset(), 0);
        // Fewer rows: selection stays visible, with the thumb at the bottom.
        let small = draw(10, 9, 5, &mut state);
        assert_eq!(state.offset(), 7);
        assert!(track(&small).ends_with('█'));
        // Replacing the choices must not retain a stale viewport offset.
        assert_eq!(track(&draw(2, 1, 7, &mut state)), "│".repeat(5));
        assert_eq!(state.offset(), 0);
    }

    #[test]
    fn tiny_areas_are_safe_and_never_overwrite_border_corners() {
        for height in 0..=2 {
            let mut state = ListState::default();
            let buffer = draw(10, 9, height, &mut state);
            assert!(!buffer.content.iter().any(|cell| cell.symbol() == "█"));
        }
    }

    #[test]
    fn fitting_lists_never_suggest_hidden_choices() {
        for count in [0, 1, 2, 5] {
            let mut state = ListState::default();
            for selected in 0..count.max(1) {
                let buffer = draw(count, selected, 7, &mut state);
                assert_eq!(
                    track(&buffer),
                    "│││││",
                    "count={count}, selected={selected}"
                );
                assert_eq!(state.offset(), 0);
            }
        }
    }
}
