use genome_lab::{
    AlleleCombat, Creature, ExperimentDefinition, Interpretation, Lifecycle, Store,
    compare_creatures, example_lab,
};

struct DeprecatedAllele;
impl Interpretation for DeprecatedAllele {
    fn name(&self) -> &'static str {
        AlleleCombat.name()
    }
    fn genome_model(&self) -> &'static str {
        AlleleCombat.genome_model()
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
    fn interpret(
        &self,
        genome: &serde_json::Value,
        parameters: &BTreeMap<String, serde_json::Value>,
    ) -> Result<Creature, String> {
        AlleleCombat.interpret(genome, parameters)
    }
}
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn typed_registry_runs_models_deterministically_and_rejects_mismatch() {
    let lab = example_lab();
    let genome = lab.models[0].initial();
    let params = BTreeMap::from([("scale".into(), json!(2.0))]);
    let a = lab
        .execute("allele-pair", &genome, "allele_release_1", params.clone())
        .unwrap();
    let b = lab
        .execute("allele-pair", &genome, "allele_release_1", params)
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.creature.components[0].value, json!(14.0));
    assert!(
        lab.execute("trait-vector", &genome, "allele_release_1", BTreeMap::new())
            .unwrap_err()
            .contains("requires")
    );
    assert_eq!(lab.compatible("allele-pair").len(), 2);
}
#[test]
fn genome_and_function_parameters_are_independent_and_genome_parameters_affect_execution() {
    let lab = example_lab();
    let model = &lab.models[1];
    assert_eq!(model.parameters(), &[("expression_gain", 1.0)]);
    let genome = model.initial();
    let run = lab
        .execute_with_genome_parameters(
            "trait-vector",
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
            "trait-vector",
            &genome,
            "vector_release_1",
            BTreeMap::from([("unknown".into(), json!(1.0))]),
            BTreeMap::new()
        )
        .is_err()
    );
}

#[test]
fn models_validate_edits_and_all_output_components_are_preserved() {
    let lab = example_lab();
    let model = &lab.models[1];
    let mut genome = model.initial();
    model.edit(&mut genome, "armor", json!(10.0)).unwrap();
    assert!(model.edit(&mut genome, "unknown", json!(1)).is_err());
    let run = lab
        .execute("trait-vector", &genome, "vector_release_1", BTreeMap::new())
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
    let result = lab.batch("allele-pair", &cases);
    assert_eq!(result.runs.len(), 2);
    assert_eq!(result.failures[0].index, 1);
}
#[test]
fn release_lifecycle_is_persistent_scoped_and_requires_confirmation() {
    let mut lab = example_lab();
    let mut store = Store::default();
    store.register(&AlleleCombat).unwrap();
    let run = lab
        .execute(
            "allele-pair",
            &lab.models[0].initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    store.save_run(run.clone()).unwrap();
    store.register(&genome_lab::VectorCombat).unwrap();
    store
        .save_run(
            lab.execute(
                "trait-vector",
                &lab.models[1].initial(),
                "vector_release_1",
                BTreeMap::new(),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(store.delete_deprecated("allele_release_1", true).is_err());
    assert!(
        store
            .validate_registry(["vector_release_1"].into_iter())
            .iter()
            .any(|e| e.contains("allele_release_1"))
    );
    lab.interpretations[0] = Box::new(DeprecatedAllele);
    store.register(lab.interpretations[0].as_ref()).unwrap();
    let deprecated_run = lab
        .execute(
            "allele-pair",
            &lab.models[0].initial(),
            "allele_release_1",
            BTreeMap::new(),
        )
        .unwrap();
    assert!(deprecated_run.deprecated);
    store.save_run(deprecated_run).unwrap();
    assert!(
        !store.runs[0].deprecated,
        "historical run evidence is unchanged"
    );
    assert!(store.runs.last().unwrap().deprecated);
    assert!(store.delete_deprecated("allele_release_1", false).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.json");
    store.save(&path).unwrap();
    let mut loaded = Store::load(&path).unwrap();
    assert_eq!(loaded.runs.len(), 3);
    assert_eq!(loaded.runs[0].genome_snapshot, run.genome_snapshot);
    assert_eq!(
        loaded.delete_deprecated("allele_release_1", true).unwrap(),
        2
    );
    assert_eq!(loaded.runs.len(), 1);
    assert!(loaded.releases.contains_key("vector_release_1"));
    assert!(loaded.deleted_releases.contains("allele_release_1"));
    assert!(loaded.register(&DeprecatedAllele).is_err());
    loaded.save(&path).unwrap();
    let restarted = Store::load(&path).unwrap();
    assert!(restarted.deleted_releases.contains("allele_release_1"));
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
    assert!(loaded.delete_deprecated("allele_release_1", true).is_err());
    assert!(loaded.register(&genome_lab::AlleleMobility).is_err());
}

#[test]
fn sweeps_record_each_case_and_creature_comparison_uses_semantic_keys_and_units() {
    let lab = example_lab();
    let genome = lab.models[1].initial();
    let result = lab.sweep(
        "trait-vector",
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
fn experiment_definitions_survive_store_roundtrip() {
    let mut store = Store::default();
    let definition = ExperimentDefinition {
        name: "baseline".into(),
        genome_model: "trait-vector".into(),
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
            "allele-pair",
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
