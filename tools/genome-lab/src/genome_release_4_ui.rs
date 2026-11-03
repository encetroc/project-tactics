use ratatui::{Frame, layout::Rect};
pub fn render(frame: &mut Frame, area: Rect, focused: bool) {
    crate::editor::render(
        frame,
        area,
        "genome_release_4 archived",
        "Archived genomes are historical references and cannot be edited".into(),
        focused,
    );
}
