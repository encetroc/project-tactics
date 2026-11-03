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
                genome.get(*field).unwrap_or(&serde_json::Value::Null)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    crate::editor::render(
        frame,
        area,
        "genome_release_5 editor",
        format!(
            "This typed island has no compatible active function\n{values}\n\nDraft: {draft}\nEnter applies\n{}",
            error.unwrap_or("")
        ),
        focused,
    );
}

#[cfg(test)]
mod tests {
    use super::apply;
    use genome_lab::example_lab;

    #[test]
    fn isolated_genome_editor_applies_only_finite_model_valid_values() {
        let lab = example_lab();
        let model = lab
            .models
            .iter()
            .find(|model| model.name() == "genome_release_5")
            .unwrap();
        let mut genome = model.initial();
        apply(model.as_ref(), &mut genome, "value", "2.5").unwrap();
        assert_eq!(genome["value"], serde_json::json!(2.5));
        assert!(apply(model.as_ref(), &mut genome, "value", "NaN").is_err());
        assert!(apply(model.as_ref(), &mut genome, "other", "3").is_err());
        assert_eq!(genome["value"], serde_json::json!(2.5));
    }
}
