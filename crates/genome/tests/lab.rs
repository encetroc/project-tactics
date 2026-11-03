use genome_lab::{
    AlleleCombat, Creature, ExperimentDefinition, Interpretation, Lifecycle, PreviewState, Store,
    compare_creatures, example_lab,
};

struct DeprecatedAllele;
impl Interpretation for DeprecatedAllele {
    fn name(&self) -> &'static str {
        AlleleCombat.name()
    }
    fn input_type(&self) -> std::any::TypeId {
        AlleleCombat.input_type()
    }
    fn interpret_typed(
        &self,
        genome: &(dyn std::any::Any + Send + Sync),
        parameters: &BTreeMap<String, serde_json::Value>,
    ) -> Result<Creature, String> {
        AlleleCombat.interpret_typed(genome, parameters)
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn deprecated(&self) -> bool {
        true
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        AlleleCombat.parameters()
    }
}
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn non_finite_interpretation_output_is_rejected_instead_of_returning_invalid_creature() {
    let lab = example_lab();
    let result = lab.execute(
        "genome_release_1",
        &lab.models[0].initial(),
        "allele_release_1",
        BTreeMap::from([("scale".into(), json!(1.0e308))]),
    );
    assert!(result.is_err(), "overflowing output must be rejected");
}

#[test]
fn typed_registry_runs_models_deterministically_and_rejects_mismatch() {
    let lab = example_lab();
    let genome = lab.models[0].initial();
    let params = BTreeMap::from([("scale".into(), json!(2.0))]);
    let a = lab
        .execute(
            "genome_release_1",
            &genome,
            "allele_release_1",
            params.clone(),
        )
        .unwrap();
    let b = lab
        .execute("genome_release_1", &genome, "allele_release_1", params)
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.creature.components[0].value, json!(14.0));
    assert!(
        lab.execute(
            "genome_release_3",
            &genome,
            "allele_release_1",
            BTreeMap::new()
        )
        .unwrap_err()
        .contains("requires")
    );
    assert_eq!(lab.compatible("genome_release_1").len(), 2);
    assert_eq!(lab.compatible("genome_release_2").len(), 2);
    assert!(lab.compatible("genome_release_5").is_empty());
    assert!(
        lab.execute(
            "genome_release_5",
            &serde_json::json!({"value":1.0}),
            "allele_release_1",
            BTreeMap::new()
        )
        .is_err()
    );
    let shared = lab
        .execute(
            "genome_release_2",
            &genome,
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    assert_eq!(shared.creature.components[0].value, json!(7.0));
}

#[test]
fn deprecated_genome_and_function_releases_remain_executable_and_saveable_independently() {
    let lab = example_lab();
    let release_genome = lab
        .models
        .iter()
        .find(|model| model.name() == "genome_release_1")
        .unwrap();
    let deprecated_genome = lab
        .models
        .iter()
        .find(|model| model.name() == "genome_release_2")
        .unwrap();

    let mut function_deprecated = Store::default();
    function_deprecated
        .register_model(release_genome.as_ref())
        .unwrap();
    function_deprecated.register(&DeprecatedAllele).unwrap();
    let function_only_run = lab
        .execute(
            release_genome.name(),
            &release_genome.initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    assert!(!function_only_run.genome_deprecated);
    function_deprecated.save_run(function_only_run).unwrap();
    assert!(function_deprecated.runs[0].deprecated);
    assert!(!function_deprecated.runs[0].genome_deprecated);

    let mut genome_deprecated = Store::default();
    genome_deprecated
        .register_model(deprecated_genome.as_ref())
        .unwrap();
    genome_deprecated.register(&AlleleCombat).unwrap();
    let genome_only_run = lab
        .execute(
            deprecated_genome.name(),
            &deprecated_genome.initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    assert!(genome_only_run.genome_deprecated);
    assert!(!genome_only_run.deprecated);
    genome_deprecated.save_run(genome_only_run).unwrap();
    assert!(genome_deprecated.runs[0].genome_deprecated);
    assert!(!genome_deprecated.runs[0].deprecated);
}

#[test]
fn archiving_is_independent_and_preserves_history_while_hiding_authoring_choices() {
    let lab = example_lab();
    let mut store = Store::default();
    store.register_model(lab.models[0].as_ref()).unwrap();
    store.register(&AlleleCombat).unwrap();
    let historical = lab
        .execute(
            "genome_release_1",
            &lab.models[0].initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    store.save_run(historical.clone()).unwrap();
    store
        .save_experiment(ExperimentDefinition {
            name: "historic".into(),
            genome_model: "genome_release_4".into(),
            genome: serde_json::json!({"strength":[3,5],"vitality":[8,6]}),
            genome_parameters: BTreeMap::new(),
            interpretation: "allele_release_2".into(),
            parameters: BTreeMap::new(),
        })
        .unwrap();
    let archived_model = lab
        .models
        .iter()
        .find(|model| model.name() == "genome_release_4")
        .unwrap();
    let archived_function = lab
        .interpretations
        .iter()
        .find(|function| function.name() == "allele_release_2")
        .unwrap();
    store.register_model(archived_model.as_ref()).unwrap();
    store.register(archived_function.as_ref()).unwrap();
    assert!(archived_model.archived());
    assert!(archived_function.archived());
    assert!(lab.compatible("genome_release_4").is_empty());
    assert!(
        lab.compatible("genome_release_1")
            .iter()
            .any(|function| function.name() == "allele_release_1")
    );
    assert!(
        lab.execute(
            "genome_release_4",
            &historical.genome_snapshot,
            "allele_release_1",
            BTreeMap::new()
        )
        .unwrap_err()
        .contains("archived")
    );
    assert_eq!(store.runs, vec![historical]);
    assert_eq!(
        store.experiments["historic"].genome_model,
        "genome_release_4"
    );
    assert!(
        store
            .validate_registry(std::iter::empty())
            .iter()
            .any(|entry| entry.contains("allele_release_2"))
    );
    assert!(
        store
            .validate_genome_registry(std::iter::empty())
            .iter()
            .any(|entry| entry.contains("genome_release_4"))
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("archived.json");
    store.save(&path).unwrap();
    let restored = Store::load(&path).unwrap();
    assert_eq!(restored.runs, store.runs);
    assert_eq!(restored.experiments, store.experiments);
    assert!(restored.genome_releases["genome_release_4"].archived);
    assert!(restored.releases["allele_release_2"].archived);
}

#[test]
fn every_lifecycle_pair_executes_but_only_exact_release_pair_is_persistable() {
    let lab = example_lab();
    let release = &lab.models[0];
    let dev = &lab.models[3];
    let input = release.initial();
    let pairs = [
        (release.name(), "allele_release_1"),
        (release.name(), "allele_dev_1"),
        (dev.name(), "allele_release_1"),
        (dev.name(), "allele_dev_1"),
    ];
    let mut store = Store::default();
    store.register_model(release.as_ref()).unwrap();
    store.register(&AlleleCombat).unwrap();
    for (model, function) in pairs {
        let run = lab
            .execute(model, &input, function, BTreeMap::new())
            .unwrap();
        if model == release.name() && function == "allele_release_1" {
            store.save_run(run).unwrap();
        } else {
            assert!(store.save_run(run).is_err());
        }
    }
    assert_eq!(store.runs.len(), 1);
}

#[test]
fn valid_changes_refresh_preview_and_invalid_drafts_retain_only_stale_output() {
    let lab = example_lab();
    let genome = lab.models[0].initial();
    let mut preview = PreviewState::default();
    preview.refresh(
        &lab,
        "genome_release_1",
        &genome,
        "allele_release_1",
        BTreeMap::new(),
        BTreeMap::new(),
        true,
    );
    let first = preview.current_run().unwrap().clone();
    assert_eq!(first.creature.components[0].value, json!(7.0));
    let changed = json!({"strength":[5,7],"vitality":[8,6]});
    preview.refresh(
        &lab,
        "genome_release_1",
        &changed,
        "allele_release_1",
        BTreeMap::new(),
        BTreeMap::new(),
        true,
    );
    let second = preview.current_run().unwrap().clone();
    assert_eq!(second.creature.components[1].value, json!(6.0));
    preview.refresh(
        &lab,
        "genome_release_1",
        &json!({}),
        "allele_release_1",
        BTreeMap::new(),
        BTreeMap::new(),
        false,
    );
    assert!(preview.stale);
    assert!(preview.current_run().is_none());
    assert_eq!(preview.run.as_ref(), Some(&second));
    assert!(preview.error.as_deref().unwrap().contains("Draft"));
    preview.refresh(
        &lab,
        "genome_release_1",
        &genome,
        "vector_release_1",
        BTreeMap::new(),
        BTreeMap::new(),
        true,
    );
    assert!(
        preview.stale,
        "a failed valid selection must stale, not replace, the previous result"
    );
    assert!(
        preview
            .error
            .as_deref()
            .unwrap()
            .contains("different Rust input contract")
    );
}

#[test]
fn genome_and_function_parameters_are_independent_and_genome_parameters_affect_execution() {
    let lab = example_lab();
    let model = &lab.models[2];
    assert_eq!(model.parameters(), &[("expression_gain", 1.0)]);
    let genome = model.initial();
    let run = lab
        .execute_with_genome_parameters(
            "genome_release_3",
            &genome,
            "vector_release_1",
            BTreeMap::from([("expression_gain".into(), json!(2.0))]),
            BTreeMap::from([("gain".into(), json!(0.5))]),
        )
        .unwrap();
    assert_eq!(run.genome_parameters["expression_gain"], json!(2.0));
    assert_eq!(run.parameters["gain"], json!(0.5));
    assert_eq!(
        run.creature
            .components
            .iter()
            .find(|c| c.key == "armor")
            .unwrap()
            .value,
        json!(4.0)
    );
    assert!(
        lab.execute_with_genome_parameters(
            "genome_release_3",
            &genome,
            "vector_release_1",
            BTreeMap::from([("unknown".into(), json!(1.0))]),
            BTreeMap::new()
        )
        .is_err()
    );
}

#[test]
fn batches_and_sweeps_apply_genome_preparation_parameters() {
    let lab = example_lab();
    let genome = lab.models[2].initial();
    let genome_parameters = BTreeMap::from([("expression_gain".into(), json!(2.0))]);
    let cases = vec![(genome.clone(), "vector_release_1".into(), BTreeMap::new())];
    let batch = lab.batch_with_genome_parameters("genome_release_3", &genome_parameters, &cases);
    assert_eq!(batch.runs[0].genome_parameters, genome_parameters);
    assert_eq!(batch.runs[0].creature.components[0].value, json!(8.0));

    let sweep = lab.sweep_with_genome_parameters(
        "genome_release_3",
        &genome,
        "vector_release_1",
        &genome_parameters,
        &BTreeMap::new(),
        "gain",
        &[json!(0.5), json!(1.0)],
    );
    assert_eq!(sweep.runs.len(), 2);
    assert_eq!(
        sweep.runs[1].genome_parameters["expression_gain"],
        json!(2.0)
    );
    assert_eq!(sweep.runs[1].creature.components[0].value, json!(8.0));
}

#[test]
fn models_validate_edits_and_all_output_components_are_preserved() {
    let lab = example_lab();
    let model = &lab.models[2];
    let mut genome = model.initial();
    model.edit(&mut genome, "armor", json!(10.0)).unwrap();
    assert!(model.edit(&mut genome, "unknown", json!(1)).is_err());
    let run = lab
        .execute(
            "genome_release_3",
            &genome,
            "vector_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    assert_eq!(run.creature.components.len(), 3);
    assert!(run.creature.components.iter().any(|c| c.key == "mobility"));
}
#[test]
fn batch_keeps_going_and_records_invalid_case() {
    let lab = example_lab();
    let g = lab.models[0].initial();
    let cases = vec![
        (g.clone(), "allele_release_1".into(), BTreeMap::new()),
        (json!({}), "allele_release_1".into(), BTreeMap::new()),
        (g, "allele_release_1".into(), BTreeMap::new()),
    ];
    let result = lab.batch("genome_release_1", &cases);
    assert_eq!(result.runs.len(), 2);
    assert_eq!(result.failures[0].index, 1);
}
#[test]
fn release_lifecycle_preserves_runs_and_detects_missing_registrations() {
    let lab = example_lab();
    let mut store = Store::default();
    store.register_model(lab.models[0].as_ref()).unwrap();
    store.register(&AlleleCombat).unwrap();
    let run = lab
        .execute(
            "genome_release_1",
            &lab.models[0].initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    store.save_run(run.clone()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.json");
    store.save(&path).unwrap();
    let loaded = Store::load(&path).unwrap();
    assert_eq!(loaded.runs, vec![run]);
    assert!(
        loaded
            .validate_registry(std::iter::empty())
            .iter()
            .any(|e| e.contains("allele_release_1"))
    );
}

#[test]
fn registration_replaces_persisted_deprecation_with_code_metadata() {
    let mut store = Store::default();
    store.register(&DeprecatedAllele).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.json");
    store.save(&path).unwrap();
    let mut loaded = Store::load(&path).unwrap();
    assert!(loaded.releases["allele_release_1"].deprecated);
    loaded.register(&AlleleCombat).unwrap();
    assert!(!loaded.releases["allele_release_1"].deprecated);
    assert!(loaded.register(&genome_lab::AlleleMobility).is_err());
}

#[test]
fn sweeps_record_each_case_and_creature_comparison_uses_semantic_keys_and_units() {
    let lab = example_lab();
    let genome = lab.models[2].initial();
    let result = lab.sweep(
        "genome_release_3",
        &genome,
        "vector_release_1",
        &BTreeMap::new(),
        "gain",
        &[json!(0.5), json!(1.0), json!("invalid")],
    );
    assert_eq!(result.runs.len(), 2);
    assert_eq!(result.failures.len(), 1);
    let left = &result.runs[0].creature;
    let right = &result.runs[1].creature;
    let comparison = compare_creatures(left, right);
    assert!(comparison.compatible);
    assert_eq!(comparison.differences.len(), 3);
    assert_ne!(
        comparison.differences[0].left,
        comparison.differences[0].right
    );
}

#[test]
fn same_display_label_does_not_make_different_units_comparable() {
    let left = genome_lab::Creature {
        components: vec![genome_lab::Component {
            key: "movement.distance".into(),
            label: "Power".into(),
            value: json!(3),
            unit: "tiles".into(),
        }],
    };
    let right = genome_lab::Creature {
        components: vec![genome_lab::Component {
            key: "movement.distance".into(),
            label: "Power".into(),
            value: json!(3),
            unit: "meters".into(),
        }],
    };
    let comparison = compare_creatures(&left, &right);
    assert!(!comparison.compatible);
    assert_eq!(comparison.differences[0].right, None);
    assert_eq!(
        comparison.differences[0].left_unit.as_deref(),
        Some("tiles")
    );
    assert_eq!(
        comparison.differences[0].right_unit.as_deref(),
        Some("meters")
    );
}

#[test]
fn legacy_store_roundtrip_preserves_unversioned_records_and_tombstone_evidence() {
    let legacy = json!({
        "releases": {"old_release_1": {"name":"old_release_1", "deprecated":true}},
        "runs": [{"interpretation":"old_release_1", "genome_model":"legacy-model", "genome_snapshot":{"genes":[1,2]}, "parameters":{"gain":0.5}, "creature":{"components":[{"key":"legacy.stat","label":"Legacy stat","value":3,"unit":"points"}]}, "deprecated":true}],
        "experiments": {"old": {"name":"old", "genome_model":"legacy-model", "genome":{"x":1}, "interpretation":"old_release_1", "parameters":{}}},
        "deleted_releases": ["gone_release_1"]
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.json");
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let store = Store::load(&path).unwrap();
    assert_eq!(store.schema_version, 1);
    assert!(store.releases["old_release_1"].deprecated);
    assert_eq!(store.experiments["old"].genome_model, "legacy-model");
    assert!(store.deleted_releases.contains("gone_release_1"));
    assert_eq!(store.runs[0].genome_snapshot, json!({"genes":[1,2]}));
    assert!(store.runs[0].deprecated);
    store.save(&path).unwrap();
    let after = Store::load(&path).unwrap();
    assert_eq!(after.experiments["old"].genome_model, "legacy-model");
    assert_eq!(after.runs[0].genome_model, "legacy-model");
    assert_eq!(after.runs[0].genome_snapshot, json!({"genes":[1,2]}));
    assert!(after.runs[0].deprecated);
    assert!(after.deleted_releases.contains("gone_release_1"));
}

#[test]
fn experiment_definitions_survive_store_roundtrip() {
    let mut store = Store::default();
    let definition = ExperimentDefinition {
        name: "baseline".into(),
        genome_model: "genome_release_3".into(),
        genome: json!({"traits":{"armor":4.0}}),
        genome_parameters: BTreeMap::new(),
        interpretation: "vector_release_1".into(),
        parameters: BTreeMap::from([("gain".into(), json!(1.25))]),
    };
    store.save_experiment(definition.clone()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.json");
    store.save(&path).unwrap();
    assert_eq!(
        Store::load(&path).unwrap().experiments["baseline"],
        definition
    );
}

#[test]
fn dev_runs_are_not_persistable_and_edits_preserve_model_validation() {
    let lab = example_lab();
    let run = lab
        .execute(
            "genome_release_1",
            &lab.models[0].initial(),
            "allele_dev_1",
            BTreeMap::new(),
        )
        .unwrap();
    let mut store = Store::default();
    assert!(store.save_run(run).is_err());
    assert!(genome_lab::valid_name("foo_release_1", "release"));
    assert!(!genome_lab::valid_name("foo_dev_1", "release"));
}
