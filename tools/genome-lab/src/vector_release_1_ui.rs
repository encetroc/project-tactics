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
    let body = parameters
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
        .join("\n");
    crate::editor::render(
        frame,
        area,
        "vector_release_1 parameters",
        format!(
            "{body}\n\nDraft: {draft}\nEnter applies\n{}",
            error.unwrap_or("")
        ),
        focused,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coded_function_editor_accepts_finite_parameters_and_rejects_invalid_drafts() {
        let mut parameters = std::collections::BTreeMap::new();
        apply(&mut parameters, "gain", "1.25").unwrap();
        assert_eq!(parameters["gain"], serde_json::json!(1.25));
        assert!(apply(&mut parameters, "gain", "NaN").is_err());
        assert_eq!(parameters["gain"], serde_json::json!(1.25));
    }
}
