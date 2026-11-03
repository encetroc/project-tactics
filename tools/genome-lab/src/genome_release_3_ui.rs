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
    model.edit(genome, field, serde_json::json!(number))
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
    let traits = genome.get("traits").and_then(serde_json::Value::as_object);
    let values = model
        .editable_fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            format!(
                "{}{} = {}",
                if index == selected { "▶ " } else { "  " },
                field,
                traits
                    .and_then(|items| items.get(*field))
                    .unwrap_or(&serde_json::Value::Null)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    crate::editor::render(
        frame,
        area,
        "genome_release_3 editor",
        format!(
            "Named trait coordinates\n{values}\n\nDraft: {draft}\nEnter applies\n{}\n\nPreparation: {}",
            error.unwrap_or(""),
            parameters
        ),
        focused,
    );
}
