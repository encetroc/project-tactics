mod allele_dev_1_ui;
mod allele_release_1_ui;
mod allele_release_2_ui;
mod editor;
mod genome_dev_1_ui;
mod genome_release_1_ui;
mod genome_release_2_ui;
mod genome_release_3_ui;
mod genome_release_4_ui;
mod genome_release_5_ui;
mod selector;
mod vector_release_1_ui;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use genome_lab::{
    ExperimentDefinition, Lifecycle, PreviewState, Store, compare_creatures, example_lab,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, ListItem, ListState, Paragraph},
};
use selector::render_selector;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lab = example_lab();
    let path = store_path();
    let mut store = Store::load(&path)?;
    let releases: Vec<_> = lab
        .interpretations
        .iter()
        .filter(|i| i.lifecycle() == Lifecycle::Release)
        .collect();
    let models: Vec<_> = lab
        .models
        .iter()
        .filter(|m| m.lifecycle() == Lifecycle::Release)
        .collect();
    let mut missing = store.validate_registry(releases.iter().map(|i| i.name()));
    missing.extend(store.validate_genome_registry(models.iter().map(|m| m.name())));
    if !missing.is_empty() {
        return Err(missing.join("\n").into());
    }
    for model in models {
        store.register_model(model.as_ref())?;
    }
    for interpretation in releases {
        store.register(interpretation.as_ref())?;
    }
    store.save(&path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let result = run(&mut terminal, &lab, &mut store, &path);
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result?;
    Ok(())
}

fn displayed_preview_can_be_saved(result_index: usize, stale: bool, preview_exists: bool) -> bool {
    preview_exists && result_index == 0 && !stale
}

fn matching_historical_runs<'a>(
    runs: &'a [genome_lab::Run],
    genome_model: &str,
    interpretation: &str,
) -> Vec<&'a genome_lab::Run> {
    runs.iter()
        .rev()
        .filter(|run| run.genome_model == genome_model && run.interpretation == interpretation)
        .collect()
}

fn editor_focused(active_column: usize, active_section: usize, editor_column: usize) -> bool {
    active_column == editor_column && active_section == 1
}

fn historical_run_index(result_index: usize, preview_exists: bool) -> Option<usize> {
    let history_index = result_index.checked_sub(usize::from(preview_exists))?;
    Some(history_index)
}

fn historical_run_detail(run: &genome_lab::Run) -> String {
    format!(
        "Historical run metadata:\nGenome model: {}\nGenome snapshot: {}\nGenome parameters: {}\nFunction: {}\nFunction parameters: {}\nFunction deprecated: {}\nGenome deprecated: {}\n\nOutput:\n{}",
        run.genome_model,
        serde_json::to_string_pretty(&run.genome_snapshot).unwrap_or_default(),
        serde_json::to_string_pretty(&run.genome_parameters).unwrap_or_default(),
        run.interpretation,
        serde_json::to_string_pretty(&run.parameters).unwrap_or_default(),
        run.deprecated,
        run.genome_deprecated,
        serde_json::to_string_pretty(&run.creature.components).unwrap_or_default(),
    )
}

fn reset_function_selection(
    parameters: &mut BTreeMap<String, Value>,
    selected_parameter: &mut usize,
    draft_text: &mut String,
    draft_dirty: &mut bool,
    draft_valid: &mut bool,
    editor_error: &mut Option<String>,
    preview_key: &mut Option<String>,
) {
    parameters.clear();
    *selected_parameter = 0;
    draft_text.clear();
    *draft_dirty = false;
    *draft_valid = true;
    *editor_error = None;
    *preview_key = None;
}

fn discard_draft(
    draft_text: &mut String,
    draft_dirty: &mut bool,
    draft_valid: &mut bool,
    editor_error: &mut Option<String>,
    preview_key: &mut Option<String>,
) {
    draft_text.clear();
    *draft_dirty = false;
    *draft_valid = true;
    *editor_error = None;
    *preview_key = None;
}

fn store_path() -> PathBuf {
    std::env::var_os("GENOME_LAB_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".local/genome-lab.json"))
}

#[cfg(test)]
mod host_interaction_tests {
    use super::{
        current_field_value, discard_draft, displayed_preview_can_be_saved, editor_focused,
        historical_run_detail, historical_run_index, matching_historical_runs,
        reset_function_selection,
    };
    use genome_lab::example_lab;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn save_is_enabled_only_for_the_displayed_current_preview() {
        assert!(displayed_preview_can_be_saved(0, false, true));
        assert!(!displayed_preview_can_be_saved(0, true, true));
        assert!(!displayed_preview_can_be_saved(1, false, true));
        assert!(!displayed_preview_can_be_saved(0, false, false));
    }

    #[test]
    fn editor_focus_tracks_tab_section_in_active_column() {
        assert!(editor_focused(0, 1, 0));
        assert!(!editor_focused(0, 0, 0));
        assert!(!editor_focused(1, 1, 0));
        assert!(editor_focused(1, 1, 1));
    }

    #[test]
    fn editor_reads_declared_top_level_field_values() {
        assert_eq!(
            current_field_value(&json!({"value": 1.25}), "value"),
            Some("1.25".into())
        );
    }

    #[test]
    fn historical_results_are_scoped_to_the_selected_model_and_function_pair() {
        let lab = example_lab();
        let allele = lab
            .execute(
                "genome_release_1",
                &lab.models[0].initial(),
                "allele_release_1",
                BTreeMap::new(),
            )
            .unwrap();
        let vector = lab
            .execute(
                "genome_release_3",
                &lab.models[2].initial(),
                "vector_release_1",
                BTreeMap::new(),
            )
            .unwrap();
        let dev_function = lab
            .execute(
                "genome_release_1",
                &lab.models[0].initial(),
                "allele_dev_1",
                BTreeMap::new(),
            )
            .unwrap();
        let runs = vec![allele, vector, dev_function];
        let visible = matching_historical_runs(&runs, "genome_release_1", "allele_release_1");
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].interpretation, "allele_release_1");
        assert_eq!(visible[0].genome_model, "genome_release_1");
    }

    #[test]
    fn history_metadata_index_accounts_for_absent_preview() {
        assert_eq!(historical_run_index(0, false), Some(0));
        assert_eq!(historical_run_index(1, false), Some(1));
        assert_eq!(historical_run_index(1, true), Some(0));
        assert_eq!(historical_run_index(0, true), None);
    }

    #[test]
    fn historical_details_include_snapshot_parameters_lifecycle_and_output() {
        let lab = example_lab();
        let model = &lab.models[0];
        let function = &lab.interpretations[0];
        let run = lab
            .execute(
                model.name(),
                &model.initial(),
                function.name(),
                BTreeMap::new(),
            )
            .unwrap();
        let detail = historical_run_detail(&run);
        for expected in [
            "Genome snapshot:",
            "Genome parameters:",
            "Function parameters:",
            "Function deprecated:",
            "Genome deprecated:",
            "Output:",
        ] {
            assert!(detail.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn navigation_discards_invalid_draft_before_reenabling_preview() {
        let mut draft = "-".to_owned();
        let mut dirty = true;
        let mut valid = false;
        let mut error = Some("incomplete".to_owned());
        let mut key = Some("old".to_owned());
        discard_draft(&mut draft, &mut dirty, &mut valid, &mut error, &mut key);
        assert!(draft.is_empty());
        assert!(!dirty && valid && error.is_none() && key.is_none());
    }

    #[test]
    fn switching_function_discards_overrides_and_invalid_draft_before_execution() {
        let lab = example_lab();
        let mut parameters = BTreeMap::from([("scale".into(), json!(2.0))]);
        let mut selected = 3;
        let mut draft = "-".to_owned();
        let mut dirty = true;
        let mut valid = false;
        let mut error = Some("incomplete".to_owned());
        let mut preview_key = Some("old".to_owned());
        reset_function_selection(
            &mut parameters,
            &mut selected,
            &mut draft,
            &mut dirty,
            &mut valid,
            &mut error,
            &mut preview_key,
        );
        assert!(parameters.is_empty());
        assert_eq!(selected, 0);
        assert!(!dirty && valid && error.is_none());
        let run = lab
            .execute(
                "genome_release_1",
                &lab.models[0].initial(),
                "allele_dev_1",
                parameters,
            )
            .unwrap();
        assert_eq!(run.interpretation, "allele_dev_1");
    }
}

fn prompt<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    question: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    eprint!("{question}: ");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(answer.trim().to_owned())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    lab: &genome_lab::Lab,
    store: &mut Store,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut model_index = 0usize;
    let mut interpretation_index = 0usize;
    let mut result_index = 0usize;
    let mut active_column = 0usize;
    let mut genome: Value = lab
        .models
        .iter()
        .find(|model| !model.archived())
        .map_or(Value::Null, |model| model.initial());
    let mut genome_parameters = BTreeMap::<String, Value>::new();
    let mut parameters = BTreeMap::<String, Value>::new();
    let mut genome_parameter_index = 0usize;
    let mut selected_parameter = 0usize;
    let mut active_section = 0usize;
    let mut message = "Ready".to_owned();
    let mut preview = PreviewState::default();
    let mut preview_key: Option<String> = None;
    let mut draft_valid = true;
    let mut draft_text = String::new();
    let mut draft_dirty = false;
    let mut edit_genome_parameter = false;
    let mut editor_error: Option<String> = None;
    let mut field_index = 0usize;
    let mut output_scroll = 0u16;
    let mut model_state = ListState::default();
    let mut function_state = ListState::default();
    let mut result_state = ListState::default();

    loop {
        let active_models: Vec<_> = lab
            .models
            .iter()
            .filter(|model| !model.archived())
            .collect();
        if active_models.is_empty() {
            let historical = &store.runs;
            if historical.is_empty() {
                result_index = 0;
            } else {
                result_index %= historical.len();
            }
            terminal.draw(|frame| {
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(7), Constraint::Min(3)])
                    .split(frame.area());
                frame.render_widget(
                    Paragraph::new("No active genome models are registered")
                        .block(Block::default().borders(Borders::ALL).title("Genome Lab")),
                    rows[0],
                );
                let label = historical
                    .iter()
                    .rev()
                    .nth(result_index)
                    .map(|run| format!("{} · {}", run.genome_model, run.interpretation))
                    .unwrap_or_else(|| "No historical runs".into());
                let detail = historical
                    .iter()
                    .rev()
                    .nth(result_index)
                    .map(|run| serde_json::to_string_pretty(run).unwrap_or_default())
                    .unwrap_or_else(|| "No stored history".into());
                frame.render_widget(
                    Paragraph::new(format!("{label}\n\n{detail}")).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Historical run details"),
                    ),
                    rows[1],
                );
            })?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up if !historical.is_empty() => {
                        result_index = (result_index + 1) % historical.len()
                    }
                    KeyCode::Down if !historical.is_empty() => {
                        result_index = (result_index + historical.len() - 1) % historical.len()
                    }
                    _ => {}
                }
            }
            continue;
        }
        model_index %= active_models.len();
        let model = active_models[model_index].as_ref();
        if field_index >= model.editable_fields().len() {
            field_index = 0;
        }
        let compatible: Vec<_> = lab
            .compatible(model.name())
            .into_iter()
            .filter(|i| !i.archived())
            .collect();
        if compatible.is_empty() {
            let historical: Vec<_> = store
                .runs
                .iter()
                .filter(|run| run.genome_model == model.name())
                .collect();
            let labels = if historical.is_empty() {
                vec![ListItem::new("No historical runs")]
            } else {
                historical
                    .iter()
                    .rev()
                    .map(|run| {
                        ListItem::new(format!("{} · {}", run.genome_model, run.interpretation))
                    })
                    .collect()
            };
            result_index = result_index.min(labels.len().saturating_sub(1));
            let models = active_models
                .iter()
                .map(|entry| ListItem::new(entry.name()))
                .collect();
            model_state.select(Some(model_index));
            result_state.select(Some(result_index));
            terminal.draw(|frame| {
                let columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ])
                    .split(frame.area());
                let areas: Vec<_> = columns
                    .iter()
                    .map(|column| {
                        Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Length(7), Constraint::Min(3)])
                            .split(*column)
                    })
                    .collect();
                render_selector(
                    frame,
                    areas[0][0],
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Genome selection"),
                    models,
                    &mut model_state,
                );
                if model.name() == "genome_release_5" {
                    genome_release_5_ui::render(
                        frame,
                        areas[0][1],
                        model,
                        &genome,
                        field_index,
                        &draft_text,
                        editor_error.as_deref(),
                        editor_focused(active_column, active_section, 0),
                    );
                } else {
                    editor::render(
                        frame,
                        areas[0][1],
                        "Genome input",
                        format!(
                            "{}\n{}",
                            model.description(),
                            serde_json::to_string_pretty(&genome).unwrap_or_default()
                        ),
                        editor_focused(active_column, active_section, 0),
                    );
                }
                frame.render_widget(
                    Paragraph::new("No active compatible function").block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Function selection"),
                    ),
                    areas[1][0],
                );
                let function_editor_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Function editor");
                let function_editor_block = if editor_focused(active_column, active_section, 1) {
                    function_editor_block.border_style(Style::default().fg(Color::Yellow))
                } else {
                    function_editor_block
                };
                frame.render_widget(
                    Paragraph::new("No function editor available").block(function_editor_block),
                    areas[1][1],
                );
                render_selector(
                    frame,
                    areas[2][0],
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Historical results"),
                    labels,
                    &mut result_state,
                );
                let detail = historical
                    .iter()
                    .rev()
                    .nth(result_index)
                    .map(|run| historical_run_detail(run))
                    .unwrap_or_else(|| "No stored output for this genome".into());
                frame.render_widget(
                    Paragraph::new(detail).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Historical output"),
                    ),
                    areas[2][1],
                );
            })?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => active_column = active_column.saturating_sub(1),
                    KeyCode::Right => active_column = (active_column + 1).min(2),
                    KeyCode::Tab => active_section = 1 - active_section,
                    KeyCode::Up | KeyCode::Down
                        if active_column == 0
                            && active_section == 1
                            && model.name() == "genome_release_5" =>
                    {
                        let fields = model.editable_fields();
                        if !fields.is_empty() {
                            field_index = if key.code == KeyCode::Down {
                                (field_index + 1) % fields.len()
                            } else {
                                (field_index + fields.len() - 1) % fields.len()
                            };
                            draft_text = current_field_value(&genome, fields[field_index])
                                .unwrap_or_default();
                            draft_dirty = false;
                            draft_valid = true;
                            editor_error = None;
                        }
                    }
                    KeyCode::Char(character)
                        if active_column == 0
                            && active_section == 1
                            && model.name() == "genome_release_5" =>
                    {
                        if !draft_dirty {
                            draft_text.clear();
                            draft_dirty = true;
                        }
                        genome_release_5_ui::handle_character(&mut draft_text, character);
                        draft_valid = false;
                        editor_error = None;
                    }
                    KeyCode::Backspace
                        if active_column == 0
                            && active_section == 1
                            && model.name() == "genome_release_5" =>
                    {
                        if !draft_dirty {
                            draft_text.clear();
                            draft_dirty = true;
                        } else {
                            draft_text.pop();
                        }
                        draft_valid = false;
                    }
                    KeyCode::Enter
                        if active_column == 0
                            && active_section == 1
                            && model.name() == "genome_release_5" =>
                    {
                        if let Some(field) = model.editable_fields().get(field_index).copied() {
                            let mut candidate = genome.clone();
                            match genome_release_5_ui::apply(
                                model,
                                &mut candidate,
                                field,
                                &draft_text,
                            ) {
                                Ok(()) => {
                                    genome = candidate;
                                    draft_valid = true;
                                    draft_dirty = false;
                                    editor_error = None;
                                    draft_text =
                                        current_field_value(&genome, field).unwrap_or_default();
                                    message = format!(
                                        "Updated {field}; no compatible function is available"
                                    );
                                }
                                Err(error) => {
                                    draft_valid = false;
                                    editor_error = Some(error);
                                }
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Down if active_column == 0 => {
                        let down = key.code == KeyCode::Down;
                        model_index = if down {
                            (model_index + 1) % active_models.len()
                        } else {
                            (model_index + active_models.len() - 1) % active_models.len()
                        };
                        genome = active_models[model_index].initial();
                        genome_parameters.clear();
                        parameters.clear();
                        field_index = 0;
                        genome_parameter_index = 0;
                        selected_parameter = 0;
                        edit_genome_parameter = false;
                        draft_text.clear();
                        draft_dirty = false;
                        draft_valid = true;
                        editor_error = None;
                        result_index = 0;
                        preview_key = None;
                    }
                    KeyCode::Up | KeyCode::Down if active_column == 2 && !historical.is_empty() => {
                        result_index = if key.code == KeyCode::Down {
                            (result_index + 1) % historical.len()
                        } else {
                            (result_index + historical.len() - 1) % historical.len()
                        };
                    }
                    _ => {}
                }
            }
            continue;
        }
        interpretation_index %= compatible.len();
        let interpretation = compatible[interpretation_index];
        let defaults = interpretation.parameters();
        let genome_defaults = model.parameters();
        if selected_parameter >= defaults.len() {
            selected_parameter = 0;
        }
        if genome_parameter_index >= genome_defaults.len() {
            genome_parameter_index = 0;
        }
        if draft_text.is_empty() && !draft_dirty {
            if active_column == 0 && edit_genome_parameter {
                if let Some((name, default)) = genome_defaults.get(genome_parameter_index) {
                    draft_text = genome_parameters
                        .get(*name)
                        .and_then(Value::as_f64)
                        .unwrap_or(*default)
                        .to_string();
                }
            } else if active_column == 0 {
                draft_text = model
                    .editable_fields()
                    .get(field_index)
                    .and_then(|field| current_field_value(&genome, field))
                    .unwrap_or_default();
            } else if let Some((name, default)) = defaults.get(selected_parameter) {
                draft_text = parameters
                    .get(*name)
                    .and_then(Value::as_f64)
                    .unwrap_or(*default)
                    .to_string();
            }
        }
        let genome_parameter_text = if genome_defaults.is_empty() {
            "no editable parameters".to_owned()
        } else {
            genome_defaults
                .iter()
                .enumerate()
                .map(|(i, (name, default))| {
                    let value = genome_parameters
                        .get(*name)
                        .and_then(Value::as_f64)
                        .unwrap_or(*default);
                    format!(
                        "{}{}={value}",
                        if i == genome_parameter_index {
                            "▶"
                        } else {
                            "  "
                        },
                        name
                    )
                })
                .collect::<Vec<_>>()
                .join("  ")
        };
        let parameter_text = if defaults.is_empty() {
            "no editable parameters".to_owned()
        } else {
            defaults
                .iter()
                .enumerate()
                .map(|(i, (name, default))| {
                    let value = parameters
                        .get(*name)
                        .and_then(Value::as_f64)
                        .unwrap_or(*default);
                    format!(
                        "{}{}={value}",
                        if i == selected_parameter { "▶" } else { "  " },
                        name
                    )
                })
                .collect::<Vec<_>>()
                .join("  ")
        };
        let current_key = serde_json::to_string(&(
            model.name(),
            &genome,
            interpretation.name(),
            &genome_parameters,
            &parameters,
        ))?;
        if preview_key.as_deref() != Some(&current_key) || !draft_valid {
            preview.refresh(
                lab,
                model.name(),
                &genome,
                interpretation.name(),
                effective_parameters(genome_defaults, &genome_parameters),
                effective_parameters(defaults, &parameters),
                draft_valid,
            );
            preview_key = Some(current_key);
        }
        let latest = preview.run.as_ref();
        let preview_stale = preview.stale;
        if let Some(error) = &preview.error {
            message = format!(
                "Preview {}: {error}",
                if preview_stale {
                    "stale"
                } else {
                    "unavailable"
                }
            );
        }
        let historical_runs =
            matching_historical_runs(&store.runs, model.name(), interpretation.name());
        let mut results: Vec<(String, String)> = historical_runs
            .iter()
            .map(|run| {
                (
                    if run.deprecated || run.genome_deprecated {
                        format!("Saved {} [deprecated]", run.interpretation)
                    } else {
                        format!("Saved {}", run.interpretation)
                    },
                    serde_json::to_string_pretty(&run.creature.components)
                        .unwrap_or_else(|error| error.to_string()),
                )
            })
            .collect();
        if let Some(run) = latest {
            results.insert(
                0,
                (
                    format!(
                        "Latest{} · {}",
                        if preview_stale { " [STALE]" } else { "" },
                        run.interpretation
                    ),
                    serde_json::to_string_pretty(&run.creature.components)
                        .unwrap_or_else(|error| error.to_string()),
                ),
            );
        }
        if results.is_empty() {
            results.push(("No results yet".into(), message.clone()));
        } else {
            result_index = result_index.min(results.len() - 1);
        }
        let models = active_models
            .iter()
            .map(|model| {
                let label = if store
                    .genome_releases
                    .get(model.name())
                    .is_some_and(|record| record.deprecated)
                {
                    format!("{} [deprecated]", model.name())
                } else {
                    model.name().to_owned()
                };
                ListItem::new(label)
            })
            .collect::<Vec<_>>();
        let functions = compatible
            .iter()
            .map(|interpretation| {
                let label = if store
                    .releases
                    .get(interpretation.name())
                    .is_some_and(|release| release.deprecated)
                {
                    format!("{} [deprecated]", interpretation.name())
                } else {
                    interpretation.name().to_owned()
                };
                ListItem::new(label)
            })
            .collect::<Vec<_>>();
        let result_labels = results
            .iter()
            .map(|(label, _)| ListItem::new(label.clone()))
            .collect::<Vec<_>>();
        model_state.select(Some(model_index));
        function_state.select(Some(interpretation_index));
        result_state.select(Some(result_index));
        terminal.draw(|frame| {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ]
                    .as_ref(),
                )
                .split(frame.area());
            let genome_areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(7), Constraint::Min(3)])
                .split(columns[0]);
            let function_areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(7), Constraint::Min(3)])
                .split(columns[1]);
            let result_areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(7), Constraint::Min(3)])
                .split(columns[2]);
            let block = |title: &'static str, column: usize, section: usize| {
                let block = Block::default().borders(Borders::ALL).title(title);
                if active_column == column && active_section == section {
                    block.border_style(Style::default().fg(Color::Yellow))
                } else {
                    block
                }
            };
            render_selector(
                frame,
                genome_areas[0],
                block("Genome selection", 0, 0),
                models,
                &mut model_state,
            );
            match model.name() {
                "genome_release_1" => genome_release_1_ui::render(
                    frame,
                    genome_areas[1],
                    model,
                    &genome,
                    field_index,
                    &draft_text,
                    editor_error.as_deref(),
                    &genome_parameter_text,
                    editor_focused(active_column, active_section, 0),
                ),
                "genome_release_2" => genome_release_2_ui::render(
                    frame,
                    genome_areas[1],
                    model,
                    &genome,
                    field_index,
                    &draft_text,
                    editor_error.as_deref(),
                    &genome_parameter_text,
                    editor_focused(active_column, active_section, 0),
                ),
                "genome_release_3" => genome_release_3_ui::render(
                    frame,
                    genome_areas[1],
                    model,
                    &genome,
                    field_index,
                    &draft_text,
                    editor_error.as_deref(),
                    &genome_parameter_text,
                    editor_focused(active_column, active_section, 0),
                ),
                "genome_release_4" => genome_release_4_ui::render(
                    frame,
                    genome_areas[1],
                    editor_focused(active_column, active_section, 0),
                ),
                "genome_release_5" => genome_release_5_ui::render(
                    frame,
                    genome_areas[1],
                    model,
                    &genome,
                    field_index,
                    &draft_text,
                    editor_error.as_deref(),
                    editor_focused(active_column, active_section, 0),
                ),
                "genome_dev_1" => genome_dev_1_ui::render(
                    frame,
                    genome_areas[1],
                    model,
                    &genome,
                    field_index,
                    &draft_text,
                    editor_error.as_deref(),
                    &genome_parameter_text,
                    editor_focused(active_column, active_section, 0),
                ),
                _ => editor::render(
                    frame,
                    genome_areas[1],
                    "Genome editor",
                    format!("{}\n{}", model.description(), genome_parameter_text),
                    editor_focused(active_column, active_section, 0),
                ),
            }
            render_selector(
                frame,
                function_areas[0],
                block("Function selection", 1, 0),
                functions,
                &mut function_state,
            );
            match interpretation.name() {
                "allele_release_1" => allele_release_1_ui::render(
                    frame,
                    function_areas[1],
                    defaults,
                    &parameters,
                    selected_parameter,
                    &draft_text,
                    editor_error.as_deref(),
                    editor_focused(active_column, active_section, 1),
                ),
                "allele_dev_1" => allele_dev_1_ui::render(
                    frame,
                    function_areas[1],
                    editor_focused(active_column, active_section, 1),
                ),
                "allele_release_2" => allele_release_2_ui::render(
                    frame,
                    function_areas[1],
                    editor_focused(active_column, active_section, 1),
                ),
                "vector_release_1" => vector_release_1_ui::render(
                    frame,
                    function_areas[1],
                    defaults,
                    &parameters,
                    selected_parameter,
                    &draft_text,
                    editor_error.as_deref(),
                    editor_focused(active_column, active_section, 1),
                ),
                _ => editor::render(
                    frame,
                    function_areas[1],
                    "Function editor",
                    parameter_text.clone(),
                    editor_focused(active_column, active_section, 1),
                ),
            }
            render_selector(
                frame,
                result_areas[0],
                block("Results", 2, 0),
                result_labels,
                &mut result_state,
            );
            let selected_result = &results[result_index];
            let historical_detail = historical_run_index(result_index, latest.is_some())
                .and_then(|index| historical_runs.get(index).copied())
                .map(historical_run_detail)
                .unwrap_or_default();
            frame.render_widget(
                Paragraph::new(format!(
                    "{}\n\n{}\n\nParameters: {}\n\n{}\n\n{}",
                    selected_result.1,
                    historical_detail,
                    parameter_text,
                    message,
                    selected_result.0
                ))
                .scroll((output_scroll, 0))
                .block(block("Selected result", 2, 1)),
                result_areas[1],
            );
        })?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::PageUp => output_scroll = output_scroll.saturating_sub(5),
                KeyCode::PageDown => output_scroll = output_scroll.saturating_add(5),
                KeyCode::Left => {
                    active_column = active_column.saturating_sub(1);
                    discard_draft(
                        &mut draft_text,
                        &mut draft_dirty,
                        &mut draft_valid,
                        &mut editor_error,
                        &mut preview_key,
                    );
                }
                KeyCode::Right => {
                    active_column = (active_column + 1).min(2);
                    discard_draft(
                        &mut draft_text,
                        &mut draft_dirty,
                        &mut draft_valid,
                        &mut editor_error,
                        &mut preview_key,
                    );
                }
                KeyCode::Up | KeyCode::Down => {
                    let down = key.code == KeyCode::Down;
                    if active_section == 1 {
                        match active_column {
                            0 if edit_genome_parameter && !genome_defaults.is_empty() => {
                                genome_parameter_index = if down {
                                    (genome_parameter_index + 1) % genome_defaults.len()
                                } else {
                                    (genome_parameter_index + genome_defaults.len() - 1)
                                        % genome_defaults.len()
                                };
                                let (name, default) = genome_defaults[genome_parameter_index];
                                draft_text = genome_parameters
                                    .get(name)
                                    .and_then(Value::as_f64)
                                    .unwrap_or(default)
                                    .to_string();
                                draft_dirty = false;
                                draft_valid = true;
                                editor_error = None;
                            }
                            0 if !model.editable_fields().is_empty() => {
                                field_index = if down {
                                    (field_index + 1) % model.editable_fields().len()
                                } else {
                                    (field_index + model.editable_fields().len() - 1)
                                        % model.editable_fields().len()
                                };
                                draft_text = model
                                    .editable_fields()
                                    .get(field_index)
                                    .and_then(|field| current_field_value(&genome, field))
                                    .unwrap_or_default();
                                draft_dirty = false;
                                draft_valid = true;
                                editor_error = None;
                            }
                            0 if !genome_defaults.is_empty() => {
                                genome_parameter_index = if down {
                                    (genome_parameter_index + 1) % genome_defaults.len()
                                } else {
                                    (genome_parameter_index + genome_defaults.len() - 1)
                                        % genome_defaults.len()
                                };
                                let (name, default) = genome_defaults[genome_parameter_index];
                                draft_text = genome_parameters
                                    .get(name)
                                    .and_then(Value::as_f64)
                                    .unwrap_or(default)
                                    .to_string();
                                draft_dirty = false;
                            }
                            1 if !defaults.is_empty() => {
                                selected_parameter = if down {
                                    (selected_parameter + 1) % defaults.len()
                                } else {
                                    (selected_parameter + defaults.len() - 1) % defaults.len()
                                };
                                let (name, default) = defaults[selected_parameter];
                                draft_text = parameters
                                    .get(name)
                                    .and_then(Value::as_f64)
                                    .unwrap_or(default)
                                    .to_string();
                                draft_valid = true;
                                editor_error = None;
                                draft_dirty = false;
                            }
                            2 if !results.is_empty() => {
                                result_index = if down {
                                    (result_index + 1) % results.len()
                                } else {
                                    (result_index + results.len() - 1) % results.len()
                                };
                            }
                            _ => {}
                        }
                    } else {
                        match active_column {
                            0 => {
                                model_index = if down {
                                    (model_index + 1) % active_models.len()
                                } else {
                                    (model_index + active_models.len() - 1) % active_models.len()
                                };
                                genome = active_models[model_index].initial();
                                genome_parameters.clear();
                                genome_parameter_index = 0;
                                selected_parameter = 0;
                                field_index = 0;
                                edit_genome_parameter = false;
                                draft_text.clear();
                                draft_dirty = false;
                                editor_error = None;
                                interpretation_index = 0;
                                result_index = 0;
                                parameters.clear();
                                draft_valid = true;
                                preview_key = None;
                                message = "Selected model and sample genome".into();
                            }
                            1 => {
                                if !compatible.is_empty() {
                                    interpretation_index = if down {
                                        (interpretation_index + 1) % compatible.len()
                                    } else {
                                        (interpretation_index + compatible.len() - 1)
                                            % compatible.len()
                                    };
                                    result_index = 0;
                                    reset_function_selection(
                                        &mut parameters,
                                        &mut selected_parameter,
                                        &mut draft_text,
                                        &mut draft_dirty,
                                        &mut draft_valid,
                                        &mut editor_error,
                                        &mut preview_key,
                                    );
                                }
                            }
                            2 => {
                                if !results.is_empty() {
                                    if down {
                                        result_index = (result_index + 1) % results.len();
                                    } else {
                                        result_index =
                                            (result_index + results.len() - 1) % results.len();
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                KeyCode::Tab => {
                    active_section = 1 - active_section;
                    discard_draft(
                        &mut draft_text,
                        &mut draft_dirty,
                        &mut draft_valid,
                        &mut editor_error,
                        &mut preview_key,
                    );
                }
                KeyCode::Char('p')
                    if active_section == 1 && active_column == 0 && !genome_defaults.is_empty() =>
                {
                    edit_genome_parameter = !edit_genome_parameter;
                    discard_draft(
                        &mut draft_text,
                        &mut draft_dirty,
                        &mut draft_valid,
                        &mut editor_error,
                        &mut preview_key,
                    );
                    message = if edit_genome_parameter {
                        "Editing genome preparation parameter".into()
                    } else {
                        "Editing concrete genome input".into()
                    };
                }
                KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('-')
                    if active_section == 1 && (active_column == 1 || edit_genome_parameter) =>
                {
                    let delta = if key.code == KeyCode::Char('-') {
                        -0.1
                    } else {
                        0.1
                    };
                    if active_column == 0
                        && active_section == 1
                        && edit_genome_parameter
                        && !genome_defaults.is_empty()
                    {
                        let (name, default) = genome_defaults[genome_parameter_index];
                        let current = genome_parameters
                            .get(name)
                            .and_then(Value::as_f64)
                            .unwrap_or(default);
                        genome_parameters.insert(name.into(), json!(current + delta));
                        preview_key = None;
                        draft_valid = !draft_dirty;
                    } else if active_column == 1 && active_section == 1 && !defaults.is_empty() {
                        let (name, default) = defaults[selected_parameter];
                        let current = parameters
                            .get(name)
                            .and_then(Value::as_f64)
                            .unwrap_or(default);
                        parameters.insert(name.into(), json!(current + delta));
                        if !draft_dirty {
                            draft_text = (current + delta).to_string();
                        }
                        preview_key = None;
                        draft_valid = !draft_dirty;
                    }
                }
                KeyCode::Char(character)
                    if active_section == 1
                        && active_column == 0
                        && (character.is_ascii_digit() || matches!(character, '-' | '.')) =>
                {
                    if !draft_dirty {
                        draft_text.clear();
                        draft_dirty = true;
                    }
                    genome_editor_input(model.name(), &mut draft_text, character);
                    draft_valid = false;
                    preview_key = None;
                    editor_error = None;
                }
                KeyCode::Backspace if active_section == 1 && active_column < 2 => {
                    if !draft_dirty {
                        draft_text.clear();
                        draft_dirty = true;
                    } else {
                        draft_text.pop();
                    }
                    draft_valid = false;
                    preview_key = None;
                }
                KeyCode::Char(character)
                    if active_section == 1
                        && active_column == 1
                        && (character.is_ascii_digit() || matches!(character, '-' | '.')) =>
                {
                    if !draft_dirty {
                        draft_text.clear();
                        draft_dirty = true;
                    }
                    function_editor_input(interpretation.name(), &mut draft_text, character);
                    draft_valid = false;
                    preview_key = None;
                    editor_error = None;
                }
                KeyCode::Char('e') if active_section == 1 && active_column < 2 => {
                    editor_error = Some("Edit the selected numeric draft, then press Enter".into());
                }
                KeyCode::Enter
                    if active_section == 1 && active_column == 0 && edit_genome_parameter =>
                {
                    if let Some((name, _)) = genome_defaults.get(genome_parameter_index) {
                        match editor::finite_number(&draft_text) {
                            Ok(value) => {
                                genome_parameters.insert((*name).to_owned(), json!(value));
                                draft_valid = true;
                                draft_dirty = false;
                                preview_key = None;
                                editor_error = None;
                                message =
                                    format!("Updated genome parameter {name}; preview refreshed");
                            }
                            Err(error) => {
                                draft_valid = false;
                                editor_error = Some(error.clone());
                                preview.error = Some(error);
                                preview.stale = preview.run.is_some();
                            }
                        }
                    }
                }
                KeyCode::Enter if active_section == 1 && active_column == 0 => {
                    let field = model
                        .editable_fields()
                        .get(field_index)
                        .copied()
                        .unwrap_or("");
                    match draft_text
                        .parse::<f64>()
                        .ok()
                        .filter(|number| number.is_finite())
                    {
                        Some(_) => {
                            let mut candidate = genome.clone();
                            let result = match model.name() {
                                "genome_release_1" => genome_release_1_ui::apply(
                                    model,
                                    &mut candidate,
                                    field,
                                    &draft_text,
                                ),
                                "genome_release_2" => genome_release_2_ui::apply(
                                    model,
                                    &mut candidate,
                                    field,
                                    &draft_text,
                                ),
                                "genome_release_3" => genome_release_3_ui::apply(
                                    model,
                                    &mut candidate,
                                    field,
                                    &draft_text,
                                ),
                                "genome_dev_1" => genome_dev_1_ui::apply(
                                    model,
                                    &mut candidate,
                                    field,
                                    &draft_text,
                                ),
                                _ => Err("No coded editor is registered for this genome".into()),
                            };
                            match result {
                                Ok(()) => {
                                    genome = candidate;
                                    draft_valid = true;
                                    draft_dirty = false;
                                    preview_key = None;
                                    editor_error = None;
                                    message = format!("Updated {field}; preview refreshed");
                                }
                                Err(error) => {
                                    draft_valid = false;
                                    editor_error = Some(error.clone());
                                    preview.error = Some(error);
                                    preview.stale = preview.run.is_some();
                                }
                            }
                        }
                        None => {
                            draft_valid = false;
                            editor_error = Some("Enter a finite numeric value".into());
                            preview.stale = preview.run.is_some();
                        }
                    }
                }
                KeyCode::Enter
                    if active_section == 1 && active_column == 1 && !defaults.is_empty() =>
                {
                    let (name, _) = defaults[selected_parameter];
                    let result = match interpretation.name() {
                        "allele_release_1" => {
                            allele_release_1_ui::apply(&mut parameters, name, &draft_text)
                        }
                        "vector_release_1" => {
                            vector_release_1_ui::apply(&mut parameters, name, &draft_text)
                        }
                        _ => Err("This function has no editable parameters".into()),
                    };
                    match result {
                        Ok(()) => {
                            draft_valid = true;
                            draft_dirty = false;
                            preview_key = None;
                            editor_error = None;
                            message = format!("Updated {name}; preview refreshed");
                        }
                        Err(error) => {
                            draft_valid = false;
                            editor_error = Some(error.clone());
                            preview.error = Some(error);
                            preview.stale = preview.run.is_some();
                        }
                    }
                }
                KeyCode::Enter => {
                    preview_key = None;
                    draft_valid = true;
                    message = "Preview refreshed".into();
                }
                KeyCode::Char('S') => {
                    if interpretation.lifecycle() != Lifecycle::Release
                        || model.lifecycle() != Lifecycle::Release
                    {
                        message = "Only release/release runs can be saved".into();
                    } else if displayed_preview_can_be_saved(
                        result_index,
                        preview_stale,
                        latest.is_some(),
                    ) {
                        if let Some(run) = preview.current_run().cloned() {
                            if model.archived() || interpretation.archived() {
                                message = "Archived entries cannot be saved as a new run".into();
                            } else if store.save_run(run).is_ok() {
                                store.save(path)?;
                                message = "Saved release preview".into();
                            } else {
                                message = "Preview is not eligible for persistence".into();
                            }
                        } else {
                            message = "No current successful preview to save".into();
                        }
                    } else {
                        message = "Select the current preview before saving; historical results are read-only".into();
                    }
                }
                KeyCode::Char('c') => {
                    let skip_latest = usize::from(store.runs.last() == latest);
                    let prior = store.runs.iter().rev().skip(skip_latest).find(|r| {
                        r.genome_model == model.name() && r.interpretation == interpretation.name()
                    });
                    message = match (latest, prior) {
                        (Some(current), Some(previous)) => format!("Semantic comparison (compatible={}):\n{}", compare_creatures(&previous.creature, &current.creature).compatible, serde_json::to_string_pretty(&compare_creatures(&previous.creature, &current.creature).differences)?),
                        _ => "Run this creature and save a release run before comparing, or run a release twice".into(),
                    };
                }
                KeyCode::Char('b') => {
                    let good = (
                        genome.clone(),
                        interpretation.name().to_owned(),
                        effective_parameters(defaults, &parameters),
                    );
                    let bad = (
                        json!({}),
                        interpretation.name().to_owned(),
                        effective_parameters(defaults, &parameters),
                    );
                    let result = lab.batch_with_genome_parameters(
                        model.name(),
                        &effective_parameters(genome_defaults, &genome_parameters),
                        &[good.clone(), bad, good],
                    );
                    message = format!(
                        "Batch result ({} successful, {} failed):\n{}",
                        result.runs.len(),
                        result.failures.len(),
                        serde_json::to_string_pretty(&result)?
                    );
                }
                KeyCode::Char('s') if !defaults.is_empty() => {
                    let (name, _) = defaults[selected_parameter];
                    let raw = prompt(terminal, &format!("Comma-separated values for {name}"))?;
                    let values: Result<Vec<Value>, _> = raw
                        .split(',')
                        .map(|v| v.trim().parse::<f64>().map(|value| json!(value)))
                        .collect();
                    message = match values {
                        Ok(values) => {
                            let result = lab.sweep_with_genome_parameters(
                                model.name(),
                                &genome,
                                interpretation.name(),
                                &effective_parameters(genome_defaults, &genome_parameters),
                                &effective_parameters(defaults, &parameters),
                                name,
                                &values,
                            );
                            format!(
                                "Sweep result ({} successful, {} failed):\n{}",
                                result.runs.len(),
                                result.failures.len(),
                                serde_json::to_string_pretty(&result)?
                            )
                        }
                        Err(_) => "Sweep values must be comma-separated finite numbers".into(),
                    };
                }
                KeyCode::Char('w') => {
                    let name = prompt(terminal, "Experiment definition name")?;
                    let definition = ExperimentDefinition {
                        name: name.clone(),
                        genome_model: model.name().into(),
                        genome: genome.clone(),
                        interpretation: interpretation.name().into(),
                        genome_parameters: effective_parameters(
                            genome_defaults,
                            &genome_parameters,
                        ),
                        parameters: effective_parameters(defaults, &parameters),
                    };
                    message = match store.save_experiment(definition) {
                        Ok(()) => {
                            store.save(path)?;
                            format!("Saved experiment {name}")
                        }
                        Err(e) => e,
                    };
                }
                KeyCode::Char('l') => {
                    let name = prompt(terminal, "Saved experiment name")?;
                    if let Some(definition) = store.experiments.get(&name).cloned() {
                        let saved_model = lab
                            .models
                            .iter()
                            .find(|model| model.name() == definition.genome_model);
                        if saved_model.is_some_and(|model| model.archived()) {
                            message =
                                "Saved genome is archived; historical definition was not activated"
                                    .into();
                        } else if let Some(mi) = active_models
                            .iter()
                            .position(|model| model.name() == definition.genome_model)
                        {
                            let active = lab.compatible(&definition.genome_model);
                            if let Some(ii) = active
                                .iter()
                                .position(|function| function.name() == definition.interpretation)
                            {
                                model_index = mi;
                                interpretation_index = ii;
                                genome = definition.genome;
                                genome_parameters = definition.genome_parameters;
                                parameters = definition.parameters;
                                field_index = 0;
                                genome_parameter_index = 0;
                                selected_parameter = 0;
                                edit_genome_parameter = false;
                                editor_error = None;
                                preview_key = None;
                                draft_valid = true;
                                draft_text.clear();
                                draft_dirty = false;
                                message = format!("Loaded experiment {name}");
                            } else if lab.interpretations.iter().any(|function| {
                                function.name() == definition.interpretation && function.archived()
                            }) {
                                message = "Saved interpretation is archived; historical definition was not activated".into();
                            } else {
                                message = "Saved interpretation is missing or incompatible".into();
                            }
                        } else {
                            message = "Saved genome model is missing or archived; historical definition was not activated".into();
                        }
                    } else {
                        message = "Unknown saved experiment".into();
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn genome_editor_input(name: &str, draft: &mut String, character: char) {
    match name {
        "genome_release_1" => genome_release_1_ui::handle_character(draft, character),
        "genome_release_2" => genome_release_2_ui::handle_character(draft, character),
        "genome_release_3" => genome_release_3_ui::handle_character(draft, character),
        "genome_dev_1" => genome_dev_1_ui::handle_character(draft, character),
        _ => {}
    }
}

fn function_editor_input(name: &str, draft: &mut String, character: char) {
    match name {
        "allele_release_1" => allele_release_1_ui::handle_character(draft, character),
        "vector_release_1" => vector_release_1_ui::handle_character(draft, character),
        _ => {}
    }
}

fn current_field_value(genome: &Value, field: &str) -> Option<String> {
    let mut parts = field.split('.');
    let root = parts.next()?;
    let value = if let Some(index) = parts.next() {
        genome.get(root)?.get(index.parse::<usize>().ok()?)?
    } else if let Some(value) = genome.get(root) {
        value
    } else {
        genome.get("traits")?.get(root)?
    };
    Some(value.to_string())
}

fn effective_parameters(
    defaults: &[(&str, f64)],
    overrides: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    defaults
        .iter()
        .map(|(name, default)| {
            (
                (*name).to_owned(),
                overrides
                    .get(*name)
                    .cloned()
                    .unwrap_or_else(|| json!(*default)),
            )
        })
        .collect()
}
