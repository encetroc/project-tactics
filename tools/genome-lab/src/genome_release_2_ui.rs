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
            let mut parts = field.split('.');
            let root = parts.next().unwrap_or_default();
            let value = parts
                .next()
                .and_then(|part| part.parse::<usize>().ok())
                .and_then(|index| genome.get(root)?.get(index))
                .unwrap_or(&serde_json::Value::Null);
            format!(
                "{}{} = {}",
                if index == selected { "▶ " } else { "  " },
                field,
                value
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    crate::editor::render(
        frame,
        area,
        "genome_release_2 editor",
        format!(
            "Shared allele contract\n{values}\n\nDraft: {draft}\nEnter applies\n{}\n\nPreparation: {}",
            error.unwrap_or(""),
            parameters
        ),
        focused,
    );
}
