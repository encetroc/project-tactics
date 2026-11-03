use genome_lab::GenomeModel;
use ratatui::{Frame, layout::Rect};

pub fn handle_character(buffer: &mut String, character: char) {
    crate::editor::input_character(buffer, character);
}

pub fn apply(
    model: &dyn GenomeModel,
    genome: &mut serde_json::Value,
    field: &str,
    draft: &str,
) -> Result<(), String> {
    let number = crate::editor::finite_number(draft)?;
    let value = draft
        .parse::<i64>()
        .map(serde_json::Value::from)
        .unwrap_or_else(|_| serde_json::json!(number));
    model.edit(genome, field, value)
}
pub fn render(
    frame: &mut Frame,
    area: Rect,
    model: &dyn GenomeModel,
    genome: &serde_json::Value,
    selected: usize,
    draft: &str,
    error: Option<&str>,
    parameters: &str,
    focused: bool,
) {
    let values = model
        .editable_fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            format!(
                "{}{} = {}",
                if index == selected { "▶ " } else { "  " },
                field,
                genome
                    .get(field.split('.').next().unwrap_or_default())
                    .and_then(|value| value.get(field.split('.').nth(1).unwrap_or_default()))
                    .unwrap_or(&serde_json::Value::Null)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    crate::editor::render(
        frame,
        area,
        "genome_dev_1 editor",
        format!(
            "Development allele model\n{values}\n\nDraft: {draft}\nEnter applies\n{}\n\nPreparation: {}",
            error.unwrap_or(""),
            parameters
        ),
        focused,
    );
}
