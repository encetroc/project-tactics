use ratatui::{Frame, layout::Rect};
pub fn render(frame: &mut Frame, area: Rect, focused: bool) {
    crate::editor::render(
        frame,
        area,
        "allele_release_2 archived",
        "Archived interpretations remain in history but cannot be selected for authoring".into(),
        focused,
    );
}
