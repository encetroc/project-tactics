use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub key: String,
    pub label: String,
    pub value: Value,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Creature {
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Run {
    pub interpretation: String,
    pub genome_model: String,
    pub genome_snapshot: Value,
    #[serde(default)]
    pub genome_parameters: BTreeMap<String, Value>,
    pub parameters: BTreeMap<String, Value>,
    pub creature: Creature,
    pub deprecated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FailedCase {
    pub index: usize,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExperimentDefinition {
    pub name: String,
    pub genome_model: String,
    pub genome: Value,
    #[serde(default)]
    pub genome_parameters: BTreeMap<String, Value>,
    pub interpretation: String,
    pub parameters: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ComponentDifference {
    pub key: String,
    pub label: String,
    pub left_unit: Option<String>,
    pub right_unit: Option<String>,
    pub left: Option<Value>,
    pub right: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatureComparison {
    pub compatible: bool,
    pub differences: Vec<ComponentDifference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BatchResult {
    pub runs: Vec<Run>,
    pub failures: Vec<FailedCase>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lifecycle {
    Dev,
    Release,
}

pub trait GenomeModel: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn initial(&self) -> Value;
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[]
    }
    fn prepare(
        &self,
        genome: &Value,
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Value, String> {
        let mut effective = BTreeMap::new();
        for (name, default) in self.parameters() {
            let value = parameters
                .get(*name)
                .cloned()
                .unwrap_or_else(|| Value::from(*default));
            if !value.as_f64().is_some_and(f64::is_finite) {
                return Err(format!("genome parameter {name} must be a finite number"));
            }
            effective.insert((*name).to_owned(), value);
        }
        if let Some(extra) = parameters
            .keys()
            .find(|key| !self.parameters().iter().any(|(name, _)| name == key))
        {
            return Err(format!("undeclared genome parameter: {extra}"));
        }
        let _ = effective;
        Ok(genome.clone())
    }
    fn editable_fields(&self) -> &'static [&'static str];
    fn validate(&self, genome: &Value) -> Result<(), String>;
    fn edit(&self, genome: &mut Value, field: &str, value: Value) -> Result<(), String>;
}

pub trait Interpretation: Send + Sync {
    fn name(&self) -> &'static str;
    fn genome_model(&self) -> &'static str;
    fn lifecycle(&self) -> Lifecycle;
    /// Code-owned release metadata; override to mark a release eligible for confirmed deletion.
    fn deprecated(&self) -> bool {
        false
    }
    fn parameters(&self) -> &'static [(&'static str, f64)];
    fn interpret(
        &self,
        genome: &Value,
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Creature, String>;
}

#[derive(Default)]
pub struct Lab {
    pub models: Vec<Box<dyn GenomeModel>>,
    pub interpretations: Vec<Box<dyn Interpretation>>,
}

impl Lab {
    pub fn compatible(&self, model: &str) -> Vec<&dyn Interpretation> {
        self.interpretations
            .iter()
            .filter(|i| i.genome_model() == model)
            .map(|i| i.as_ref())
            .collect()
    }
    pub fn execute(
        &self,
        model_name: &str,
        genome: &Value,
        interpretation_name: &str,
        parameters: BTreeMap<String, Value>,
    ) -> Result<Run, String> {
        self.execute_with_genome_parameters(
            model_name,
            genome,
            interpretation_name,
            BTreeMap::new(),
            parameters,
        )
    }
    pub fn execute_with_genome_parameters(
        &self,
        model_name: &str,
        genome: &Value,
        interpretation_name: &str,
        genome_parameters: BTreeMap<String, Value>,
        parameters: BTreeMap<String, Value>,
    ) -> Result<Run, String> {
        let model = self
            .models
            .iter()
            .find(|m| m.name() == model_name)
            .ok_or_else(|| format!("unknown genome model: {model_name}"))?;
        let interpretation = self
            .interpretations
            .iter()
            .find(|i| i.name() == interpretation_name)
            .ok_or_else(|| format!("unknown interpretation: {interpretation_name}"))?;
        if interpretation.genome_model() != model_name {
            return Err(format!(
                "interpretation {} requires {}, not {}",
                interpretation.name(),
                interpretation.genome_model(),
                model_name
            ));
        }
        if let Some(extra) = genome_parameters
            .keys()
            .find(|key| !model.parameters().iter().any(|(name, _)| name == key))
        {
            return Err(format!("undeclared genome parameter: {extra}"));
        }
        let effective_genome_parameters: BTreeMap<String, Value> = model
            .parameters()
            .iter()
            .map(|(name, default)| {
                (
                    (*name).to_owned(),
                    genome_parameters
                        .get(*name)
                        .cloned()
                        .unwrap_or_else(|| Value::from(*default)),
                )
            })
            .collect();
        let prepared_genome = model.prepare(genome, &effective_genome_parameters)?;
        model.validate(&prepared_genome)?;
        let mut effective = BTreeMap::new();
        for (name, default) in interpretation.parameters() {
            let default = Value::from(*default);
            let value = parameters.get(*name).unwrap_or(&default);
            if value.is_number() {
                effective.insert((*name).to_owned(), value.clone());
            } else {
                return Err(format!("parameter {name} must be numeric"));
            }
        }
        if let Some(extra) = parameters
            .keys()
            .find(|k| !interpretation.parameters().iter().any(|(n, _)| n == k))
        {
            return Err(format!("undeclared parameter: {extra}"));
        }
        let creature = interpretation.interpret(&prepared_genome, &effective)?;
        validate_creature(&creature)?;
        Ok(Run {
            interpretation: interpretation.name().into(),
            genome_model: model_name.into(),
            genome_snapshot: prepared_genome,
            genome_parameters: effective_genome_parameters,
            parameters: effective,
            creature,
            deprecated: interpretation.deprecated(),
        })
    }
    pub fn sweep(
        &self,
        model: &str,
        genome: &Value,
        interpretation: &str,
        base_parameters: &BTreeMap<String, Value>,
        parameter: &str,
        values: &[Value],
    ) -> BatchResult {
        let cases: Vec<_> = values
            .iter()
            .map(|value| {
                let mut parameters = base_parameters.clone();
                parameters.insert(parameter.to_owned(), value.clone());
                (genome.clone(), interpretation.to_owned(), parameters)
            })
            .collect();
        self.batch(model, &cases)
    }

    pub fn batch(
        &self,
        model: &str,
        cases: &[(Value, String, BTreeMap<String, Value>)],
    ) -> BatchResult {
        let mut result = BatchResult {
            runs: Vec::new(),
            failures: Vec::new(),
        };
        for (index, (genome, interpretation, params)) in cases.iter().enumerate() {
            match self.execute(model, genome, interpretation, params.clone()) {
                Ok(run) => result.runs.push(run),
                Err(error) => result.failures.push(FailedCase { index, error }),
            }
        }
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReleaseRecord {
    pub name: String,
    pub deprecated: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Store {
    pub releases: BTreeMap<String, ReleaseRecord>,
    pub runs: Vec<Run>,
    #[serde(default)]
    pub experiments: BTreeMap<String, ExperimentDefinition>,
    #[serde(default)]
    pub deleted_releases: std::collections::BTreeSet<String>,
}
impl Store {
    pub fn save_experiment(&mut self, definition: ExperimentDefinition) -> Result<(), String> {
        if definition.name.trim().is_empty() {
            return Err("experiment name cannot be empty".into());
        }
        self.experiments.insert(definition.name.clone(), definition);
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
        fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| {
            let _ = fs::remove_file(&temporary);
            e.to_string()
        })
    }
    pub fn register(&mut self, interpretation: &dyn Interpretation) -> Result<(), String> {
        let name = interpretation.name();
        if interpretation.lifecycle() != Lifecycle::Release {
            return Err("only release interpretations may be registered".into());
        }
        if self.deleted_releases.contains(name) {
            return Err(format!(
                "release {name} was explicitly deleted; publish a new release name instead"
            ));
        }
        if !valid_name(name, "release") {
            return Err(format!("invalid release name: {name}"));
        }
        // Persist a mirror of code metadata, never a user-controlled lifecycle setting.
        self.releases.insert(
            name.into(),
            ReleaseRecord {
                name: name.into(),
                deprecated: interpretation.deprecated(),
            },
        );
        Ok(())
    }
    pub fn validate_registry<'a>(&self, registered: impl Iterator<Item = &'a str>) -> Vec<String> {
        let current: std::collections::BTreeSet<_> = registered.collect();
        self.releases
            .keys()
            .filter(|name| !current.contains(name.as_str()))
            .map(|n| format!("registered release missing from code: {n}"))
            .collect()
    }
    pub fn save_run(&mut self, run: Run) -> Result<(), String> {
        if !valid_name(&run.interpretation, "release")
            || !self.releases.contains_key(&run.interpretation)
        {
            return Err("only registered release interpretations may persist runs".into());
        }
        let mut run = run;
        run.deprecated = self.releases[&run.interpretation].deprecated;
        self.runs.push(run);
        Ok(())
    }
    pub fn delete_deprecated(&mut self, name: &str, confirmed: bool) -> Result<usize, String> {
        let release = self
            .releases
            .get(name)
            .ok_or_else(|| "unknown release".to_string())?;
        if !release.deprecated {
            return Err("release must be deprecated before deletion".into());
        }
        if !confirmed {
            return Err(format!(
                "confirmation required; deletion removes {} associated runs",
                self.runs
                    .iter()
                    .filter(|r| r.interpretation == name)
                    .count()
            ));
        }
        self.releases.remove(name);
        self.deleted_releases.insert(name.to_owned());
        let before = self.runs.len();
        self.runs.retain(|r| r.interpretation != name);
        Ok(before - self.runs.len())
    }
}
fn validate_creature(creature: &Creature) -> Result<(), String> {
    let mut identities = std::collections::BTreeSet::new();
    for component in &creature.components {
        if component.key.trim().is_empty() || component.unit.trim().is_empty() {
            return Err("creature components need semantic keys and units".into());
        }
        if !identities.insert(component.key.as_str()) {
            return Err(format!(
                "duplicate creature component key: {}",
                component.key
            ));
        }
    }
    Ok(())
}

pub fn compare_creatures(left: &Creature, right: &Creature) -> CreatureComparison {
    let mut keys = std::collections::BTreeSet::new();
    keys.extend(
        left.components
            .iter()
            .map(|component| component.key.as_str()),
    );
    keys.extend(
        right
            .components
            .iter()
            .map(|component| component.key.as_str()),
    );
    let mut compatible = true;
    let mut differences = Vec::new();
    for key in keys {
        let l = left
            .components
            .iter()
            .find(|component| component.key == key);
        let r = right
            .components
            .iter()
            .find(|component| component.key == key);
        if let (Some(a), Some(b)) = (l, r) {
            if a.unit != b.unit {
                compatible = false;
            }
            differences.push(ComponentDifference {
                key: key.to_owned(),
                label: a.label.clone(),
                left_unit: Some(a.unit.clone()),
                right_unit: Some(b.unit.clone()),
                left: Some(a.value.clone()),
                right: (a.unit == b.unit).then(|| b.value.clone()),
            });
        } else {
            let component = l.or(r).unwrap();
            differences.push(ComponentDifference {
                key: key.to_owned(),
                label: component.label.clone(),
                left_unit: l.map(|value| value.unit.clone()),
                right_unit: r.map(|value| value.unit.clone()),
                left: l.map(|value| value.value.clone()),
                right: r.map(|value| value.value.clone()),
            });
        }
    }
    CreatureComparison {
        compatible,
        differences,
    }
}

pub fn valid_name(name: &str, category: &str) -> bool {
    let parts: Vec<_> = name.split('_').collect();
    parts.len() >= 3
        && parts[0..parts.len() - 2]
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()))
        && parts[parts.len() - 2] == category
        && parts.last().is_some_and(|version| {
            !version.is_empty() && version.chars().all(|c| c.is_ascii_digit())
        })
}

pub struct AlleleModel;
impl GenomeModel for AlleleModel {
    fn name(&self) -> &'static str {
        "allele-pair"
    }
    fn description(&self) -> &'static str {
        "Unordered diploid loci: each trait has two allele values."
    }
    fn initial(&self) -> Value {
        serde_json::json!({"strength":[3,5],"vitality":[8,6]})
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        &["strength.0", "strength.1", "vitality.0", "vitality.1"]
    }
    fn validate(&self, g: &Value) -> Result<(), String> {
        let o = g.as_object().ok_or("genome must be an object")?;
        if o.is_empty() {
            return Err("genome must have loci".into());
        }
        for required in ["strength", "vitality"] {
            if !o.contains_key(required) {
                return Err(format!("required locus missing: {required}"));
            }
        }
        for (k, v) in o {
            let a = v
                .as_array()
                .ok_or_else(|| format!("locus {k} must be a pair"))?;
            if a.len() != 2 || a.iter().any(|x| x.as_i64().filter(|n| *n >= 0).is_none()) {
                return Err(format!("locus {k} must contain two nonnegative integers"));
            }
        }
        Ok(())
    }
    fn edit(&self, g: &mut Value, field: &str, value: Value) -> Result<(), String> {
        if !self.editable_fields().contains(&field) {
            return Err(format!("field is not exposed for editing: {field}"));
        }
        let parts: Vec<_> = field.split('.').collect();
        if parts.len() != 2 {
            return Err("edit field must be locus.allele_index".into());
        }
        let idx: usize = parts[1]
            .parse()
            .map_err(|_| "allele index must be 0 or 1")?;
        if idx > 1 || value.as_i64().filter(|n| *n >= 0).is_none() {
            return Err("allele index/value invalid".into());
        }
        g.get_mut(parts[0])
            .and_then(Value::as_array_mut)
            .and_then(|a| a.get_mut(idx))
            .map(|v| *v = value)
            .ok_or_else(|| "unknown locus".to_string())?;
        self.validate(g)
    }
}
pub struct TraitVectorModel;
impl GenomeModel for TraitVectorModel {
    fn name(&self) -> &'static str {
        "trait-vector"
    }
    fn description(&self) -> &'static str {
        "Named, independent real-valued trait coordinates; no loci or sequence."
    }
    fn initial(&self) -> Value {
        serde_json::json!({"traits":{"armor":4.0,"mobility":7.0,"focus":3.0}})
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[("expression_gain", 1.0)]
    }
    fn prepare(
        &self,
        genome: &Value,
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Value, String> {
        let gain = parameters
            .get("expression_gain")
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        if !gain.is_finite() {
            return Err("genome parameter expression_gain must be finite".into());
        }
        if parameters.keys().any(|key| key != "expression_gain") {
            return Err("undeclared genome parameter".into());
        }
        let mut output = genome.clone();
        let traits = output
            .get_mut("traits")
            .and_then(Value::as_object_mut)
            .ok_or("genome needs a traits object")?;
        for value in traits.values_mut() {
            let trait_value = value.as_f64().ok_or("traits must be numeric")?;
            *value = serde_json::json!(trait_value * gain);
        }
        Ok(output)
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        &["armor", "mobility", "focus"]
    }
    fn validate(&self, g: &Value) -> Result<(), String> {
        let o = g
            .get("traits")
            .and_then(Value::as_object)
            .ok_or("genome needs a traits object")?;
        if o.is_empty()
            || o.values()
                .any(|v| v.as_f64().is_none_or(|n| !n.is_finite()))
        {
            return Err("traits must be nonempty and finite".into());
        }
        Ok(())
    }
    fn edit(&self, g: &mut Value, field: &str, value: Value) -> Result<(), String> {
        if !self.editable_fields().contains(&field) {
            return Err(format!("field is not exposed for editing: {field}"));
        }
        let n = value
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or("trait value must be finite")?;
        let slot = g
            .get_mut("traits")
            .and_then(|v| v.get_mut(field))
            .ok_or("unknown trait")?;
        *slot = serde_json::json!(n);
        self.validate(g)
    }
}
pub struct AlleleCombat;
impl Interpretation for AlleleCombat {
    fn name(&self) -> &'static str {
        "allele_release_1"
    }
    fn genome_model(&self) -> &'static str {
        "allele-pair"
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[("scale", 1.0)]
    }
    fn interpret(&self, g: &Value, p: &BTreeMap<String, Value>) -> Result<Creature, String> {
        let scale = p["scale"].as_f64().unwrap();
        let mean = |k: &str| {
            g[k].as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .sum::<f64>()
                / 2.0
        };
        Ok(Creature {
            components: vec![
                component("vitality", "Vitality", mean("vitality") * scale, "points"),
                component("strength", "Strength", mean("strength") * scale, "points"),
            ],
        })
    }
}
pub struct AlleleMobility;
impl Interpretation for AlleleMobility {
    fn name(&self) -> &'static str {
        "allele_dev_1"
    }
    fn genome_model(&self) -> &'static str {
        "allele-pair"
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Dev
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[]
    }
    fn interpret(&self, g: &Value, _: &BTreeMap<String, Value>) -> Result<Creature, String> {
        Ok(Creature {
            components: vec![component(
                "strength",
                "Strength",
                g["strength"][0].as_i64().unwrap_or(0),
                "allele",
            )],
        })
    }
}
pub struct VectorCombat;
impl Interpretation for VectorCombat {
    fn name(&self) -> &'static str {
        "vector_release_1"
    }
    fn genome_model(&self) -> &'static str {
        "trait-vector"
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[("gain", 1.0)]
    }
    fn interpret(&self, g: &Value, p: &BTreeMap<String, Value>) -> Result<Creature, String> {
        let gain = p["gain"].as_f64().unwrap();
        let t = g["traits"].as_object().unwrap();
        Ok(Creature {
            components: t
                .iter()
                .map(|(k, v)| component(k, k, v.as_f64().unwrap() * gain, "trait units"))
                .collect(),
        })
    }
}
pub fn example_lab() -> Lab {
    Lab {
        models: vec![Box::new(AlleleModel), Box::new(TraitVectorModel)],
        interpretations: vec![
            Box::new(AlleleCombat),
            Box::new(AlleleMobility),
            Box::new(VectorCombat),
        ],
    }
}
fn component(key: &str, label: &str, value: impl Serialize, unit: &str) -> Component {
    Component {
        key: key.into(),
        label: label.into(),
        value: serde_json::to_value(value).unwrap(),
        unit: unit.into(),
    }
}
