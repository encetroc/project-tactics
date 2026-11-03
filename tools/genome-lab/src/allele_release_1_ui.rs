use ratatui::{Frame, layout::Rect};

pub fn handle_character(buffer: &mut String, character: char) {
    crate::editor::input_character(buffer, character);
}

pub fn apply(
    values: &mut std::collections::BTreeMap<String, serde_json::Value>,
    name: &str,
    draft: &str,
) -> Result<(), String> {
    values.insert(
        name.to_owned(),
        serde_json::json!(crate::editor::finite_number(draft)?),
    );
    Ok(())
}
pub fn render(
    frame: &mut Frame,
    area: Rect,
    parameters: &[(&str, f64)],
    values: &std::collections::BTreeMap<String, serde_json::Value>,
    selected: usize,
    draft: &str,
    error: Option<&str>,
    focused: bool,
) {
    let body = if parameters.is_empty() {
        "No editable parameters".to_owned()
    } else {
        parameters
            .iter()
            .enumerate()
            .map(|(index, (name, default))| {
                format!(
                    "{}{} = {}",
                    if index == selected { "▶ " } else { "  " },
                    name,
                    values
                        .get(*name)
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(*default)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    crate::editor::render(
        frame,
        area,
        "allele_release_1 parameters",
        format!(
            "{body}\n\nDraft: {draft}\nEnter applies\n{}",
            error.unwrap_or("")
        ),
        focused,
    );
}
