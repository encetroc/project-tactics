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
    let fields = model.editable_fields();
    let values = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            format!(
                "{}{} = {}",
                if index == selected { "▶ " } else { "  " },
                field,
                field_value(genome, field)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "{}\n\nEdit: {}\nEnter applies · type numeric value\n{}\n\nPreparation: {}",
        values,
        draft,
        error.unwrap_or(""),
        parameters
    );
    crate::editor::render(frame, area, "genome_release_1 editor", body, focused);
}
fn field_value(genome: &serde_json::Value, field: &str) -> String {
    let mut parts = field.split('.');
    let root = parts.next().unwrap_or_default();
    let value = parts
        .next()
        .and_then(|index| index.parse::<usize>().ok())
        .and_then(|index| genome.get(root)?.get(index))
        .unwrap_or(&serde_json::Value::Null);
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coded_genome_editor_applies_only_model_valid_values() {
        let lab = genome_lab::example_lab();
        let model = lab
            .models
            .iter()
            .find(|model| model.name() == "genome_release_1")
            .unwrap();
        let mut genome = model.initial();
        apply(model.as_ref(), &mut genome, "strength.0", "9").unwrap();
        assert_eq!(genome["strength"][0], 9);
        let before = genome.clone();
        assert!(apply(model.as_ref(), &mut genome, "strength.0", "-1").is_err());
        assert_eq!(genome, before);
    }
}
