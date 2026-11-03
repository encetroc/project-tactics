mod selector;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use genome_lab::{ExperimentDefinition, Lifecycle, Store, compare_creatures, example_lab};
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
        .filter(|i| {
            i.lifecycle() == Lifecycle::Release && !store.deleted_releases.contains(i.name())
        })
        .collect();
    let missing = store.validate_registry(releases.iter().map(|i| i.name()));
    if !missing.is_empty() {
        return Err(missing.join("\n").into());
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

fn store_path() -> PathBuf {
    std::env::var_os("GENOME_LAB_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".local/genome-lab.json"))
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
    let mut genome: Value = lab.models[0].initial();
    let mut genome_parameters = BTreeMap::<String, Value>::new();
    let mut parameters = BTreeMap::<String, Value>::new();
    let mut genome_parameter_index = 0usize;
    let mut selected_parameter = 0usize;
    let mut active_section = 0usize;
    let mut message = "Ready".to_owned();
    let mut latest: Option<genome_lab::Run> = None;
    let mut output_scroll = 0u16;
    let mut model_state = ListState::default();
    let mut function_state = ListState::default();
    let mut result_state = ListState::default();

    loop {
        let model = &lab.models[model_index];
        let compatible: Vec<_> = lab
            .compatible(model.name())
            .into_iter()
            .filter(|i| !store.deleted_releases.contains(i.name()))
            .collect();
        if compatible.is_empty() {
            terminal.draw(|frame| {
                frame.render_widget(
                    Paragraph::new(format!(
                        "No active interpretation is registered for {}.",
                        model.name()
                    ))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("No active interpretations"),
                    ),
                    frame.area(),
                )
            })?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Up => {
                        model_index = if key.code == KeyCode::Down {
                            (model_index + 1) % lab.models.len()
                        } else {
                            (model_index + lab.models.len() - 1) % lab.models.len()
                        };
                        genome = lab.models[model_index].initial();
                        interpretation_index = 0;
                    }
                    _ => {}
                }
            }
            continue;
        }
        interpretation_index %= compatible.len();
        let interpretation = compatible[interpretation_index];
        let defaults = interpretation.parameters();
        if selected_parameter >= defaults.len() {
            selected_parameter = 0;
        }
        let genome_defaults = model.parameters();
        if genome_parameter_index >= genome_defaults.len() {
            genome_parameter_index = 0;
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
        let mut results: Vec<(String, String)> = store
            .runs
            .iter()
            .rev()
            .map(|run| {
                (
                    if run.deprecated {
                        format!("{} [deprecated]", run.interpretation)
                    } else {
                        run.interpretation.clone()
                    },
                    serde_json::to_string_pretty(&run.creature.components)
                        .unwrap_or_else(|error| error.to_string()),
                )
            })
            .collect();
        if let Some(run) = &latest
            && store.runs.last() != Some(run)
        {
            results.insert(
                0,
                (
                    format!("Latest · {}", run.interpretation),
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
        let models = lab
            .models
            .iter()
            .map(|model| ListItem::new(model.name()))
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
            frame.render_widget(
                Paragraph::new(format!(
                    "{}\n\nInput: {}\n\n{}",
                    model.description(),
                    serde_json::to_string(&genome).unwrap_or_default(),
                    genome_parameter_text
                ))
                .block(block("Genome parameters", 0, 1)),
                genome_areas[1],
            );
            render_selector(
                frame,
                function_areas[0],
                block("Function selection", 1, 0),
                functions,
                &mut function_state,
            );
            frame.render_widget(
                Paragraph::new(parameter_text.clone()).block(block("Function parameters", 1, 1)),
                function_areas[1],
            );
            render_selector(
                frame,
                result_areas[0],
                block("Results", 2, 0),
                result_labels,
                &mut result_state,
            );
            let selected_result = &results[result_index];
            frame.render_widget(
                Paragraph::new(format!(
                    "{}\n\nParameters: {}\n\n{}\n\n{}",
                    selected_result.1, parameter_text, message, selected_result.0
                ))
                .scroll((output_scroll, 0))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Selected result"),
                ),
                result_areas[1],
            );
        })?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::PageUp => output_scroll = output_scroll.saturating_sub(5),
                KeyCode::PageDown => output_scroll = output_scroll.saturating_add(5),
                KeyCode::Left => active_column = active_column.saturating_sub(1),
                KeyCode::Right => active_column = (active_column + 1).min(2),
                KeyCode::Up | KeyCode::Down => {
                    let down = key.code == KeyCode::Down;
                    if active_section == 1 {
                        match active_column {
                            0 if !genome_defaults.is_empty() => {
                                genome_parameter_index = if down {
                                    (genome_parameter_index + 1) % genome_defaults.len()
                                } else {
                                    (genome_parameter_index + genome_defaults.len() - 1)
                                        % genome_defaults.len()
                                }
                            }
                            1 if !defaults.is_empty() => {
                                selected_parameter = if down {
                                    (selected_parameter + 1) % defaults.len()
                                } else {
                                    (selected_parameter + defaults.len() - 1) % defaults.len()
                                }
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
                                    (model_index + 1) % lab.models.len()
                                } else {
                                    (model_index + lab.models.len() - 1) % lab.models.len()
                                };
                                genome = lab.models[model_index].initial();
                                interpretation_index = 0;
                                parameters.clear();
                                latest = None;
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
                                    latest = None;
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
                KeyCode::Tab => active_section = 1 - active_section,
                KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('-') => {
                    let delta = if key.code == KeyCode::Char('-') {
                        -0.1
                    } else {
                        0.1
                    };
                    if active_column == 0 && active_section == 1 && !genome_defaults.is_empty() {
                        let (name, default) = genome_defaults[genome_parameter_index];
                        let current = genome_parameters
                            .get(name)
                            .and_then(Value::as_f64)
                            .unwrap_or(default);
                        genome_parameters.insert(name.into(), json!(current + delta));
                    } else if active_column == 1 && active_section == 1 && !defaults.is_empty() {
                        let (name, default) = defaults[selected_parameter];
                        let current = parameters
                            .get(name)
                            .and_then(Value::as_f64)
                            .unwrap_or(default);
                        parameters.insert(name.into(), json!(current + delta));
                    }
                }
                KeyCode::Char('e') => {
                    let field = prompt(
                        terminal,
                        &format!("Exposed fields [{}]", model.editable_fields().join(", ")),
                    )?;
                    let raw = prompt(terminal, "New numeric value")?;
                    let value = raw
                        .parse::<i64>()
                        .map(|value| json!(value))
                        .or_else(|_| raw.parse::<f64>().map(|value| json!(value)));
                    message = match value {
                        Ok(value) => match model.edit(&mut genome, &field, value) {
                            Ok(()) => format!("Edited {field}"),
                            Err(e) => e,
                        },
                        Err(_) => "Enter an integer or finite decimal value".into(),
                    };
                }
                KeyCode::Enter => {
                    let params = effective_parameters(defaults, &parameters);
                    match lab.execute_with_genome_parameters(
                        model.name(),
                        &genome,
                        interpretation.name(),
                        genome_parameters.clone(),
                        params,
                    ) {
                        Ok(run) => {
                            message = serde_json::to_string_pretty(&run.creature.components)?;
                            if interpretation.lifecycle() == Lifecycle::Release {
                                store.save_run(run.clone())?;
                                store.save(path)?;
                            }
                            latest = Some(run);
                        }
                        Err(error) => message = error,
                    }
                }
                KeyCode::Char('c') => {
                    let skip_latest = usize::from(store.runs.last() == latest.as_ref());
                    let prior = store.runs.iter().rev().skip(skip_latest).find(|r| {
                        r.genome_model == model.name() && r.interpretation == interpretation.name()
                    });
                    message = match (latest.as_ref(), prior) {
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
                    let result = lab.batch(model.name(), &[good.clone(), bad, good]);
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
                            let result = lab.sweep(
                                model.name(),
                                &genome,
                                interpretation.name(),
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
                        if let Some(mi) = lab
                            .models
                            .iter()
                            .position(|m| m.name() == definition.genome_model)
                        {
                            let active: Vec<_> = lab
                                .compatible(&definition.genome_model)
                                .into_iter()
                                .filter(|i| !store.deleted_releases.contains(i.name()))
                                .collect();
                            if let Some(ii) = active
                                .iter()
                                .position(|i| i.name() == definition.interpretation)
                            {
                                model_index = mi;
                                interpretation_index = ii;
                                genome = definition.genome;
                                genome_parameters = definition.genome_parameters;
                                parameters = definition.parameters;
                                message = format!("Loaded experiment {name}");
                            } else {
                                message = "Saved interpretation no longer registered".into();
                            }
                        } else {
                            message = "Saved genome model no longer registered".into();
                        }
                    } else {
                        message = "Unknown saved experiment".into();
                    }
                }
                KeyCode::Char('x') if interpretation.lifecycle() == Lifecycle::Release => {
                    if store
                        .releases
                        .get(interpretation.name())
                        .is_some_and(|r| r.deprecated)
                    {
                        let count = store
                            .runs
                            .iter()
                            .filter(|r| r.interpretation == interpretation.name())
                            .count();
                        message = format!(
                            "Deletion removes release {} and {count} runs. Confirm with y; any other key cancels",
                            interpretation.name()
                        );
                        terminal.draw(|frame| {
                            frame.render_widget(
                                Paragraph::new(format!("CONFIRM DELETE\n\n{}\n{} stored runs will be removed.\n\nPress y to confirm, any other key to cancel.", interpretation.name(), count))
                                    .block(Block::default().borders(Borders::ALL).title("Destructive operation")),
                                frame.area(),
                            );
                        })?;
                        if let Event::Key(confirm) = event::read()? {
                            if confirm.code == KeyCode::Char('y') {
                                store.delete_deprecated(interpretation.name(), true)?;
                                store.save(path)?;
                                latest = None;
                                message = format!(
                                    "Deleted {} and its associated runs",
                                    interpretation.name()
                                );
                            } else {
                                message = "Deletion cancelled".into();
                            }
                        }
                    } else {
                        message = "Mark this release deprecated in its Rust Interpretation implementation before deletion".into();
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
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
