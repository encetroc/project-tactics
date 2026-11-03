use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
    fs,
    path::Path,
};

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
    #[serde(default)]
    pub genome_deprecated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviewState {
    pub run: Option<Run>,
    pub stale: bool,
    pub error: Option<String>,
}
impl PreviewState {
    pub fn refresh(
        &mut self,
        lab: &Lab,
        model: &str,
        genome: &Value,
        interpretation: &str,
        genome_parameters: BTreeMap<String, Value>,
        parameters: BTreeMap<String, Value>,
        draft_valid: bool,
    ) {
        if !draft_valid {
            self.stale = self.run.is_some();
            self.error = Some("Draft is incomplete or invalid; preview was not executed".into());
            return;
        }
        match lab.execute_with_genome_parameters(
            model,
            genome,
            interpretation,
            genome_parameters,
            parameters,
        ) {
            Ok(run) => {
                self.run = Some(run);
                self.stale = false;
                self.error = None;
            }
            Err(error) => {
                self.stale = self.run.is_some();
                self.error = Some(error);
            }
        }
    }
    pub fn current_run(&self) -> Option<&Run> {
        (!self.stale).then_some(self.run.as_ref()).flatten()
    }
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
    /// Rust contract shared with compatible interpretations, independent of model identity.
    fn data_type(&self) -> TypeId;
    /// Deserialize the persisted/editable representation into the model's real Rust input type.
    fn decode(&self, genome: &Value) -> Result<Box<dyn Any + Send + Sync>, String>;
    fn encode(&self, genome: &(dyn Any + Send + Sync)) -> Result<Value, String>;
    fn prepare_typed(
        &self,
        genome: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String>;
    fn validate_typed(&self, genome: &(dyn Any + Send + Sync)) -> Result<(), String>;
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn deprecated(&self) -> bool {
        false
    }
    fn archived(&self) -> bool {
        false
    }
    fn description(&self) -> &'static str;
    fn initial(&self) -> Value;
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[]
    }
    fn editable_fields(&self) -> &'static [&'static str];
    fn validate(&self, genome: &Value) -> Result<(), String>;
    fn edit(&self, genome: &mut Value, field: &str, value: Value) -> Result<(), String>;
}

pub trait Interpretation: Send + Sync {
    fn name(&self) -> &'static str;
    /// Rust input contract; compatibility is a direct TypeId match, never a model-name match.
    fn input_type(&self) -> TypeId;
    /// Execute against the actual model data, not a JSON-shaped substitute.
    fn interpret_typed(
        &self,
        genome: &(dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Creature, String>;
    fn archived(&self) -> bool {
        false
    }
    fn lifecycle(&self) -> Lifecycle;
    /// Code-owned release metadata; deprecated releases remain executable and saveable.
    fn deprecated(&self) -> bool {
        false
    }
    fn parameters(&self) -> &'static [(&'static str, f64)];
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
            .filter(|i| {
                self.models
                    .iter()
                    .find(|m| m.name() == model)
                    .is_some_and(|m| {
                        !m.archived() && !i.archived() && i.input_type() == m.data_type()
                    })
            })
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
        if model.archived() {
            return Err(format!("genome model {model_name} is archived"));
        }
        let interpretation = self
            .interpretations
            .iter()
            .find(|i| i.name() == interpretation_name)
            .ok_or_else(|| format!("unknown interpretation: {interpretation_name}"))?;
        if interpretation.archived() {
            return Err(format!("interpretation {interpretation_name} is archived"));
        }
        if interpretation.input_type() != model.data_type() {
            return Err(format!(
                "interpretation {} requires a different Rust input contract than {}",
                interpretation.name(),
                model_name
            ));
        }
        if let Some(extra) = genome_parameters
            .keys()
            .find(|key| !model.parameters().iter().any(|(name, _)| name == key))
        {
            return Err(format!("undeclared genome parameter: {extra}"));
        }
        if genome_parameters
            .values()
            .any(|value| value.as_f64().is_none_or(|n| !n.is_finite()))
        {
            return Err("genome parameters must be finite numbers".into());
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
        let mut typed_genome = model.decode(genome)?;
        model.prepare_typed(typed_genome.as_mut(), &effective_genome_parameters)?;
        model.validate_typed(typed_genome.as_ref())?;
        let prepared_genome = model.encode(typed_genome.as_ref())?;
        let mut effective = BTreeMap::new();
        for (name, default) in interpretation.parameters() {
            let default = Value::from(*default);
            let value = parameters.get(*name).unwrap_or(&default);
            if value.as_f64().is_some_and(f64::is_finite) {
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
        let creature = interpretation.interpret_typed(typed_genome.as_ref(), &effective)?;
        validate_creature(&creature)?;
        Ok(Run {
            interpretation: interpretation.name().into(),
            genome_model: model_name.into(),
            genome_snapshot: prepared_genome,
            genome_parameters: effective_genome_parameters,
            parameters: effective,
            creature,
            deprecated: interpretation.deprecated(),
            genome_deprecated: model.deprecated(),
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
        self.sweep_with_genome_parameters(
            model,
            genome,
            interpretation,
            &BTreeMap::new(),
            base_parameters,
            parameter,
            values,
        )
    }

    pub fn sweep_with_genome_parameters(
        &self,
        model: &str,
        genome: &Value,
        interpretation: &str,
        genome_parameters: &BTreeMap<String, Value>,
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
        self.batch_with_genome_parameters(model, genome_parameters, &cases)
    }

    pub fn batch(
        &self,
        model: &str,
        cases: &[(Value, String, BTreeMap<String, Value>)],
    ) -> BatchResult {
        self.batch_with_genome_parameters(model, &BTreeMap::new(), cases)
    }

    pub fn batch_with_genome_parameters(
        &self,
        model: &str,
        genome_parameters: &BTreeMap<String, Value>,
        cases: &[(Value, String, BTreeMap<String, Value>)],
    ) -> BatchResult {
        let mut result = BatchResult {
            runs: Vec::new(),
            failures: Vec::new(),
        };
        for (index, (genome, interpretation, params)) in cases.iter().enumerate() {
            match self.execute_with_genome_parameters(
                model,
                genome,
                interpretation,
                genome_parameters.clone(),
                params.clone(),
            ) {
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
    #[serde(default)]
    pub archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenomeRecord {
    pub name: String,
    pub deprecated: bool,
    #[serde(default)]
    pub archived: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Store {
    /// Historical function release registry. Retained under its legacy key for lossless migration.
    pub releases: BTreeMap<String, ReleaseRecord>,
    #[serde(default)]
    pub genome_releases: BTreeMap<String, GenomeRecord>,
    #[serde(default)]
    pub schema_version: u32,
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
        let mut store: Self = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        // Version 1 is the additive compatibility envelope: old names and tombstone evidence
        // remain unchanged until a later explicit authoring save registers versioned identities.
        if store.schema_version == 0 {
            store.schema_version = 1;
        }
        Ok(store)
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
    pub fn register_model(&mut self, model: &dyn GenomeModel) -> Result<(), String> {
        if model.lifecycle() != Lifecycle::Release {
            return Err("only release genome models may be registered".into());
        }
        let name = model.name();
        if !valid_name(name, "release") {
            return Err(format!("invalid genome release name: {name}"));
        }
        self.genome_releases.insert(
            name.into(),
            GenomeRecord {
                name: name.into(),
                deprecated: model.deprecated(),
                archived: model.archived(),
            },
        );
        self.schema_version = 2;
        Ok(())
    }
    pub fn register(&mut self, interpretation: &dyn Interpretation) -> Result<(), String> {
        let name = interpretation.name();
        if interpretation.lifecycle() != Lifecycle::Release {
            return Err("only release interpretations may be registered".into());
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
                archived: interpretation.archived(),
            },
        );
        self.schema_version = 2;
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
    pub fn validate_genome_registry<'a>(
        &self,
        registered: impl Iterator<Item = &'a str>,
    ) -> Vec<String> {
        let current: std::collections::BTreeSet<_> = registered.collect();
        self.genome_releases
            .keys()
            .filter(|name| !current.contains(name.as_str()))
            .map(|name| format!("registered genome release missing from code: {name}"))
            .collect()
    }
    pub fn save_run(&mut self, run: Run) -> Result<(), String> {
        if !valid_name(&run.interpretation, "release")
            || !self.releases.contains_key(&run.interpretation)
            || !valid_name(&run.genome_model, "release")
            || !self.genome_releases.contains_key(&run.genome_model)
            || self
                .releases
                .get(&run.interpretation)
                .is_some_and(|record| record.archived)
            || self
                .genome_releases
                .get(&run.genome_model)
                .is_some_and(|record| record.archived)
        {
            return Err("only registered release/release pairs may persist runs".into());
        }
        let mut run = run;
        run.deprecated = self.releases[&run.interpretation].deprecated;
        run.genome_deprecated = self.genome_releases[&run.genome_model].deprecated;
        self.runs.push(run);
        Ok(())
    }
}
fn validate_creature(creature: &Creature) -> Result<(), String> {
    let mut identities = std::collections::BTreeSet::new();
    for component in &creature.components {
        if component.key.trim().is_empty() || component.unit.trim().is_empty() {
            return Err("creature components need semantic keys and units".into());
        }
        if component.value.is_null() || component.value.as_f64().is_some_and(|n| !n.is_finite()) {
            return Err(format!(
                "creature component {} has invalid non-finite output",
                component.key
            ));
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AlleleInput(pub BTreeMap<String, [u32; 2]>);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IsolatedInput {
    pub value: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitVectorInput {
    pub traits: BTreeMap<String, f64>,
}

pub struct AlleleModel;
impl GenomeModel for AlleleModel {
    fn data_type(&self) -> TypeId {
        TypeId::of::<AlleleInput>()
    }
    fn decode(&self, genome: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        serde_json::from_value::<AlleleInput>(genome.clone())
            .map(|input| Box::new(input) as Box<dyn Any + Send + Sync>)
            .map_err(|error| error.to_string())
    }
    fn encode(&self, genome: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        let input = genome
            .downcast_ref::<AlleleInput>()
            .ok_or("genome type contract mismatch")?;
        serde_json::to_value(input).map_err(|error| error.to_string())
    }
    fn prepare_typed(
        &self,
        genome: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        if !parameters.is_empty() {
            return Err("undeclared genome parameter".into());
        }
        self.validate_typed(genome)
    }
    fn validate_typed(&self, genome: &(dyn Any + Send + Sync)) -> Result<(), String> {
        let input = genome
            .downcast_ref::<AlleleInput>()
            .ok_or("genome type contract mismatch")?;
        if input.0.is_empty()
            || !input.0.contains_key("strength")
            || !input.0.contains_key("vitality")
        {
            return Err("genome must contain strength and vitality loci".into());
        }
        Ok(())
    }
    fn name(&self) -> &'static str {
        "genome_release_1"
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
        let typed = self.decode(g)?;
        self.validate_typed(typed.as_ref())
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
pub struct AlleleModelV2;
impl GenomeModel for AlleleModelV2 {
    fn name(&self) -> &'static str {
        "genome_release_2"
    }
    fn data_type(&self) -> TypeId {
        TypeId::of::<AlleleInput>()
    }
    fn decode(&self, genome: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        AlleleModel.decode(genome)
    }
    fn encode(&self, genome: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        AlleleModel.encode(genome)
    }
    fn prepare_typed(
        &self,
        genome: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        AlleleModel.prepare_typed(genome, parameters)
    }
    fn validate_typed(&self, genome: &(dyn Any + Send + Sync)) -> Result<(), String> {
        AlleleModel.validate_typed(genome)
    }
    fn deprecated(&self) -> bool {
        true
    }
    fn description(&self) -> &'static str {
        "A deprecated second coded model using the shared allele-pair data contract."
    }
    fn initial(&self) -> Value {
        AlleleModel.initial()
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        AlleleModel.editable_fields()
    }
    fn validate(&self, genome: &Value) -> Result<(), String> {
        AlleleModel.validate(genome)
    }
    fn edit(&self, genome: &mut Value, field: &str, value: Value) -> Result<(), String> {
        AlleleModel.edit(genome, field, value)
    }
}
pub struct IsolatedModel;
impl GenomeModel for IsolatedModel {
    fn name(&self) -> &'static str {
        "genome_release_5"
    }
    fn data_type(&self) -> TypeId {
        TypeId::of::<IsolatedInput>()
    }
    fn decode(&self, value: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        serde_json::from_value::<IsolatedInput>(value.clone())
            .map(|input| Box::new(input) as Box<dyn Any + Send + Sync>)
            .map_err(|error| error.to_string())
    }
    fn encode(&self, value: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        serde_json::to_value(
            value
                .downcast_ref::<IsolatedInput>()
                .ok_or("genome type contract mismatch")?,
        )
        .map_err(|error| error.to_string())
    }
    fn prepare_typed(
        &self,
        value: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        if !parameters.is_empty() {
            return Err("undeclared genome parameter".into());
        }
        self.validate_typed(value)
    }
    fn validate_typed(&self, value: &(dyn Any + Send + Sync)) -> Result<(), String> {
        let input = value
            .downcast_ref::<IsolatedInput>()
            .ok_or("genome type contract mismatch")?;
        if !input.value.is_finite() {
            return Err("value must be finite".into());
        }
        Ok(())
    }
    fn description(&self) -> &'static str {
        "An isolated typed genome island with no registered interpretation."
    }
    fn initial(&self) -> Value {
        serde_json::json!({"value": 1.0})
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        &["value"]
    }
    fn validate(&self, value: &Value) -> Result<(), String> {
        let typed = self.decode(value)?;
        self.validate_typed(typed.as_ref())
    }
    fn edit(&self, genome: &mut Value, field: &str, value: Value) -> Result<(), String> {
        if field != "value" {
            return Err("field is not exposed for editing: value".into());
        }
        let number = value
            .as_f64()
            .filter(|number| number.is_finite())
            .ok_or("value must be finite")?;
        genome["value"] = serde_json::json!(number);
        Ok(())
    }
}
pub struct AlleleArchivedModel;
impl GenomeModel for AlleleArchivedModel {
    fn name(&self) -> &'static str {
        "genome_release_4"
    }
    fn data_type(&self) -> TypeId {
        AlleleModel.data_type()
    }
    fn decode(&self, value: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        AlleleModel.decode(value)
    }
    fn encode(&self, value: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        AlleleModel.encode(value)
    }
    fn prepare_typed(
        &self,
        value: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        AlleleModel.prepare_typed(value, parameters)
    }
    fn validate_typed(&self, value: &(dyn Any + Send + Sync)) -> Result<(), String> {
        AlleleModel.validate_typed(value)
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn archived(&self) -> bool {
        true
    }
    fn description(&self) -> &'static str {
        "Archived allele model retained for historical references."
    }
    fn initial(&self) -> Value {
        AlleleModel.initial()
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        AlleleModel.editable_fields()
    }
    fn validate(&self, value: &Value) -> Result<(), String> {
        AlleleModel.validate(value)
    }
    fn edit(&self, value: &mut Value, field: &str, input: Value) -> Result<(), String> {
        AlleleModel.edit(value, field, input)
    }
}
pub struct AlleleDevModel;
impl GenomeModel for AlleleDevModel {
    fn name(&self) -> &'static str {
        "genome_dev_1"
    }
    fn data_type(&self) -> TypeId {
        AlleleModel.data_type()
    }
    fn decode(&self, value: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        AlleleModel.decode(value)
    }
    fn encode(&self, value: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        AlleleModel.encode(value)
    }
    fn prepare_typed(
        &self,
        value: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        AlleleModel.prepare_typed(value, parameters)
    }
    fn validate_typed(&self, value: &(dyn Any + Send + Sync)) -> Result<(), String> {
        AlleleModel.validate_typed(value)
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Dev
    }
    fn description(&self) -> &'static str {
        "Development allele-pair model."
    }
    fn initial(&self) -> Value {
        AlleleModel.initial()
    }
    fn editable_fields(&self) -> &'static [&'static str] {
        AlleleModel.editable_fields()
    }
    fn validate(&self, value: &Value) -> Result<(), String> {
        AlleleModel.validate(value)
    }
    fn edit(&self, value: &mut Value, field: &str, input: Value) -> Result<(), String> {
        AlleleModel.edit(value, field, input)
    }
}
pub struct TraitVectorModel;
impl GenomeModel for TraitVectorModel {
    fn data_type(&self) -> TypeId {
        TypeId::of::<TraitVectorInput>()
    }
    fn decode(&self, genome: &Value) -> Result<Box<dyn Any + Send + Sync>, String> {
        serde_json::from_value::<TraitVectorInput>(genome.clone())
            .map(|input| Box::new(input) as Box<dyn Any + Send + Sync>)
            .map_err(|error| error.to_string())
    }
    fn encode(&self, genome: &(dyn Any + Send + Sync)) -> Result<Value, String> {
        serde_json::to_value(
            genome
                .downcast_ref::<TraitVectorInput>()
                .ok_or("genome type contract mismatch")?,
        )
        .map_err(|error| error.to_string())
    }
    fn prepare_typed(
        &self,
        genome: &mut (dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<(), String> {
        let input = genome
            .downcast_mut::<TraitVectorInput>()
            .ok_or("genome type contract mismatch")?;
        let gain = parameters
            .get("expression_gain")
            .and_then(Value::as_f64)
            .ok_or("expression_gain must be numeric")?;
        if !gain.is_finite() {
            return Err("expression_gain must be finite".into());
        }
        for value in input.traits.values_mut() {
            *value *= gain;
        }
        Ok(())
    }
    fn validate_typed(&self, genome: &(dyn Any + Send + Sync)) -> Result<(), String> {
        let input = genome
            .downcast_ref::<TraitVectorInput>()
            .ok_or("genome type contract mismatch")?;
        if input.traits.is_empty() || input.traits.values().any(|value| !value.is_finite()) {
            return Err("traits must be nonempty and finite".into());
        }
        Ok(())
    }
    fn name(&self) -> &'static str {
        "genome_release_3"
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
    fn editable_fields(&self) -> &'static [&'static str] {
        &["armor", "mobility", "focus"]
    }
    fn validate(&self, g: &Value) -> Result<(), String> {
        let typed = self.decode(g)?;
        self.validate_typed(typed.as_ref())
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
pub struct AlleleArchivedFunction;
impl Interpretation for AlleleArchivedFunction {
    fn name(&self) -> &'static str {
        "allele_release_2"
    }
    fn input_type(&self) -> TypeId {
        TypeId::of::<AlleleInput>()
    }
    fn interpret_typed(
        &self,
        input: &(dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Creature, String> {
        AlleleCombat.interpret_typed(input, parameters)
    }
    fn archived(&self) -> bool {
        true
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        AlleleCombat.parameters()
    }
}
pub struct AlleleCombat;
impl Interpretation for AlleleCombat {
    fn interpret_typed(
        &self,
        genome: &(dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Creature, String> {
        let input = genome
            .downcast_ref::<AlleleInput>()
            .ok_or("function input type contract mismatch")?;
        let scale = parameters
            .get("scale")
            .and_then(Value::as_f64)
            .ok_or("scale must be numeric")?;
        let mean = |key: &str| input.0[key].iter().map(|v| f64::from(*v)).sum::<f64>() / 2.0;
        Ok(Creature {
            components: vec![
                component("vitality", "Vitality", mean("vitality") * scale, "points"),
                component("strength", "Strength", mean("strength") * scale, "points"),
            ],
        })
    }
    fn name(&self) -> &'static str {
        "allele_release_1"
    }
    fn input_type(&self) -> TypeId {
        TypeId::of::<AlleleInput>()
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[("scale", 1.0)]
    }
}
pub struct AlleleMobility;
impl Interpretation for AlleleMobility {
    fn interpret_typed(
        &self,
        genome: &(dyn Any + Send + Sync),
        _: &BTreeMap<String, Value>,
    ) -> Result<Creature, String> {
        let input = genome
            .downcast_ref::<AlleleInput>()
            .ok_or("function input type contract mismatch")?;
        Ok(Creature {
            components: vec![component(
                "strength",
                "Strength",
                input.0["strength"][0],
                "allele",
            )],
        })
    }
    fn name(&self) -> &'static str {
        "allele_dev_1"
    }
    fn input_type(&self) -> TypeId {
        TypeId::of::<AlleleInput>()
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Dev
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[]
    }
}
pub struct VectorCombat;
impl Interpretation for VectorCombat {
    fn interpret_typed(
        &self,
        genome: &(dyn Any + Send + Sync),
        parameters: &BTreeMap<String, Value>,
    ) -> Result<Creature, String> {
        let input = genome
            .downcast_ref::<TraitVectorInput>()
            .ok_or("function input type contract mismatch")?;
        let gain = parameters
            .get("gain")
            .and_then(Value::as_f64)
            .ok_or("gain must be numeric")?;
        Ok(Creature {
            components: input
                .traits
                .iter()
                .map(|(key, value)| component(key, key, value * gain, "trait units"))
                .collect(),
        })
    }
    fn name(&self) -> &'static str {
        "vector_release_1"
    }
    fn input_type(&self) -> TypeId {
        TypeId::of::<TraitVectorInput>()
    }
    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Release
    }
    fn parameters(&self) -> &'static [(&'static str, f64)] {
        &[("gain", 1.0)]
    }
}
pub fn example_lab() -> Lab {
    Lab {
        models: vec![
            Box::new(AlleleModel),
            Box::new(AlleleModelV2),
            Box::new(TraitVectorModel),
            Box::new(AlleleDevModel),
            Box::new(IsolatedModel),
            Box::new(AlleleArchivedModel),
        ],
        interpretations: vec![
            Box::new(AlleleCombat),
            Box::new(AlleleArchivedFunction),
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
