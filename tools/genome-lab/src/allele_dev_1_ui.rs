use ratatui::{Frame, layout::Rect};
pub fn render(frame: &mut Frame, area: Rect, focused: bool) {
    crate::editor::render(
        frame,
        area,
        "allele_dev_1 parameters",
        "Dev interpretation; no editable parameters".into(),
        focused,
    );
}
