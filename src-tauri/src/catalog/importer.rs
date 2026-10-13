use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 1;
const PACKAGE_PREFIX: &str = "content/foundations/arrays-memory/";

#[derive(Debug, Deserialize)]
pub(super) struct Curriculum {
    pub schema_version: u32,
    pub baseline: String,
    pub courses: Vec<CourseRecord>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CourseRecord {
    pub id: String,
    pub title: String,
    pub chapters: Vec<ChapterRecord>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChapterRecord {
    pub id: String,
    pub order: u32,
    pub title: String,
    pub summary: String,
    pub project_title: String,
    pub status: String,
    #[serde(default)]
    pub prerequisite_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Pilot {
    schema_version: u32,
    content_revision: u32,
    chapter_id: String,
    #[serde(default)]
    prerequisite_ids: Vec<String>,
    objectives: Vec<Objective>,
    coverage: Vec<Coverage>,
    items: Vec<ManifestItem>,
}

#[derive(Debug, Deserialize)]
struct Objective {
    id: String,
    description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct ManifestItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    pub artifact: String,
    #[serde(default)]
    pub supporting_artifacts: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub checks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Coverage {
    objective_id: String,
    lesson_id: String,
    practice_ids: Vec<String>,
    project_id: String,
}

#[derive(Debug, Deserialize)]
struct EvidenceRoot {
    schema_version: u32,
    records: Vec<EvidenceRecord>,
}

#[derive(Debug, Deserialize)]
struct EvidenceRecord {
    item_id: String,
    artifact_sha256: String,
    reviewed: bool,
    results: Vec<EvidenceResult>,
    #[serde(default)]
    supporting_artifact_sha256: HashMap<String, String>,
    #[serde(default)]
    package_files: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct EvidenceResult {
    check: String,
    passed: bool,
}

#[derive(Debug, Deserialize)]
struct SourcesRoot {
    schema_version: u32,
    sources: Vec<SourceRecord>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct SourceRecord {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    pub url: String,
    pub redistribution: String,
    pub status: String,
    pub notes: String,
}

#[derive(Debug)]
pub(super) struct ImportedCourse {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub position: u32,
    pub chapters: Vec<ImportedChapter>,
}

#[derive(Debug)]
pub(super) struct ImportedChapter {
    pub id: String,
    pub position: u32,
    pub title: String,
    pub summary: String,
    pub project_title: String,
    pub outline_status: String,
    pub content_status: String,
}

#[derive(Debug)]
pub(super) struct ImportedItem {
    pub id: String,
    pub chapter_id: String,
    pub kind: String,
    pub title: String,
    pub artifact: String,
    pub status: String,
    pub body_text: String,
    pub payload_json: String,
    pub citations_json: String,
    pub search_text: String,
}

#[derive(Debug)]
pub(super) struct ImportedLexiconEntry {
    pub id: String,
    pub term: String,
    pub category: String,
    pub definition: String,
    pub example: String,
    pub misconception: String,
    pub related_ids_json: String,
    pub citations_json: String,
    pub search_text: String,
}

#[derive(Debug)]
pub(super) struct ImportedCatalog {
    pub revision: u32,
    pub package_sha256: String,
    pub courses: Vec<ImportedCourse>,
    pub items: Vec<ImportedItem>,
    pub lexicon: Vec<ImportedLexiconEntry>,
    pub sources: Vec<SourceRecord>,
}

pub(super) fn load(root: &Path) -> Result<ImportedCatalog, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("Course data directory is unavailable: {error}"))?;
    let curriculum: Curriculum = read_json(&root.join("foundation/curriculum.json"))?;
    let pilot: Pilot = read_json(&root.join("foundation/pilot.json"))?;
    let evidence: EvidenceRoot = read_json(&root.join("foundation/evidence.json"))?;
    let sources: SourcesRoot = read_json(&root.join("foundation/sources.json"))?;

    require_version("curriculum", curriculum.schema_version)?;
    require_version("pilot", pilot.schema_version)?;
    require_version("evidence", evidence.schema_version)?;
    require_version("sources", sources.schema_version)?;
    if curriculum.baseline != "c17" {
        return Err(format!(
            "Unsupported C baseline '{}'; this catalog expects c17.",
            curriculum.baseline
        ));
    }

    let package = root.join("content/foundations/arrays-memory");
    let package = package
        .canonicalize()
        .map_err(|error| format!("Arrays & memory content package is unavailable: {error}"))?;
    if !package.starts_with(&root) {
        return Err("The content package resolves outside the bundled course data.".into());
    }

    validate_curriculum(&curriculum)?;
    let chapter_lookup = chapter_lookup(&curriculum);
    if !chapter_lookup.contains_key(&pilot.chapter_id) {
        return Err(format!(
            "Pilot chapter '{}' is missing from curriculum.json.",
            pilot.chapter_id
        ));
    }
    for prerequisite in &pilot.prerequisite_ids {
        if !chapter_lookup.contains_key(prerequisite) {
            return Err(format!(
                "Pilot prerequisite '{prerequisite}' does not identify a curriculum chapter."
            ));
        }
    }

    let source_lookup = unique_by_id(&sources.sources, "source")?;
    let objective_lookup = unique_by_id(&pilot.objectives, "objective")?;
    for objective in &pilot.objectives {
        require_text(&objective.description, "objective description")?;
    }
    let item_lookup = unique_by_id(&pilot.items, "pilot item")?;
    let mut evidence_lookup = HashMap::new();
    for record in evidence.records {
        if evidence_lookup
            .insert(record.item_id.clone(), record)
            .is_some()
        {
            return Err("Duplicate item id in evidence.json.".into());
        }
    }

    let mut imported_items = Vec::with_capacity(pilot.items.len());
    let mut lexicon_entries = Vec::new();
    let mut content_paths = HashSet::new();
    let mut all_item_kinds = HashMap::new();

    for item in &pilot.items {
        validate_identifier(&item.id, "pilot item id")?;
        if !matches!(
            item.kind.as_str(),
            "lesson" | "exercise" | "animation" | "lab" | "project" | "lexicon"
        ) {
            return Err(format!(
                "Unknown content kind '{}' on '{}'.",
                item.kind, item.id
            ));
        }
        if item.status != "verified" && item.status != "draft" && item.status != "planned" {
            return Err(format!(
                "Unknown status '{}' on '{}'.",
                item.status, item.id
            ));
        }
        require_text(&item.title, "pilot item title")?;
        if item.objective_ids.is_empty() {
            return Err(format!("Pilot item '{}' has no objective IDs.", item.id));
        }
        for objective_id in &item.objective_ids {
            if !objective_lookup.contains_key(objective_id.as_str()) {
                return Err(format!(
                    "Pilot item '{}' references unknown objective '{}'.",
                    item.id, objective_id
                ));
            }
        }

        let artifact = project_file(&root, &package, &item.artifact)?;
        content_paths.insert(item.artifact.clone());
        let mut supporting_paths = HashSet::new();
        for supporting in &item.supporting_artifacts {
            if !supporting_paths.insert(supporting.clone()) {
                return Err(format!(
                    "Pilot item '{}' repeats supporting artifact '{}'.",
                    item.id, supporting
                ));
            }
            project_file(&root, &package, supporting)?;
            content_paths.insert(supporting.clone());
        }

        let (payload, body_text) = load_item_payload(item, &artifact, &root, &package)?;
        validate_record(item, &payload, &root, &package, &source_lookup)?;
        validate_evidence(item, evidence_lookup.get(&item.id), &root, &package)?;
        collect_payload_paths(&payload, item, &mut content_paths)?;
        all_item_kinds.insert(item.id.clone(), item.kind.clone());

        let payload_json = serde_json::to_string(&payload)
            .map_err(|error| format!("Could not encode '{}': {error}", item.id))?;
        let citations = citations_for(&payload, &source_lookup)?;
        let citations_json = serde_json::to_string(&citations)
            .map_err(|error| format!("Could not encode citations for '{}': {error}", item.id))?;
        let search_text = format!("{} {} {} {}", item.title, item.id, body_text, payload_json);

        if item.kind == "lexicon" {
            lexicon_entries = parse_lexicon(&payload, &source_lookup)?;
        }
        imported_items.push(ImportedItem {
            id: item.id.clone(),
            chapter_id: pilot.chapter_id.clone(),
            kind: item.kind.clone(),
            title: item.title.clone(),
            artifact: item.artifact.clone(),
            status: item.status.clone(),
            body_text,
            payload_json,
            citations_json,
            search_text,
        });
    }

    validate_coverage(
        &pilot.coverage,
        &objective_lookup,
        &item_lookup,
        &all_item_kinds,
    )?;
    validate_lexicon_references(&lexicon_entries)?;
    for item_id in evidence_lookup.keys() {
        if !item_lookup.contains_key(item_id.as_str()) {
            return Err(format!(
                "Evidence record '{item_id}' does not identify a pilot item."
            ));
        }
    }

    let mut courses = Vec::with_capacity(curriculum.courses.len());
    for (position, course) in curriculum.courses.iter().enumerate() {
        let mut chapters = Vec::with_capacity(course.chapters.len());
        for chapter in &course.chapters {
            chapters.push(ImportedChapter {
                id: chapter.id.clone(),
                position: chapter.order,
                title: chapter.title.clone(),
                summary: chapter.summary.clone(),
                project_title: chapter.project_title.clone(),
                outline_status: chapter.status.clone(),
                content_status: if chapter.id == pilot.chapter_id {
                    "pilot-catalog".into()
                } else {
                    "outline".into()
                },
            });
        }
        courses.push(ImportedCourse {
            id: course.id.clone(),
            title: course.title.clone(),
            summary: course_summary(&course.id).into(),
            position: u32::try_from(position + 1).expect("course count fits u32"),
            chapters,
        });
    }

    let package_sha256 = package_digest(&root, &package, &content_paths)?;
    Ok(ImportedCatalog {
        revision: pilot.content_revision,
        package_sha256,
        courses,
        items: imported_items,
        lexicon: lexicon_entries,
        sources: sources.sources,
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Invalid JSON in {}: {error}", path.display()))
}

fn require_version(name: &str, version: u32) -> Result<(), String> {
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(format!(
            "Unsupported {name} schema version {version}; supported version is {SCHEMA_VERSION}."
        ))
    }
}

fn require_text(value: &str, description: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{description} must not be empty."))
    } else {
        Ok(())
    }
}

fn validate_identifier(value: &str, description: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.starts_with('-')
        || value.ends_with('-')
    {
        return Err(format!("Invalid {description} '{value}'."));
    }
    Ok(())
}

fn unique_by_id<'a, T>(
    values: &'a [T],
    description: &str,
) -> Result<HashMap<&'a str, &'a T>, String>
where
    T: HasId,
{
    let mut result = HashMap::with_capacity(values.len());
    for value in values {
        validate_identifier(value.id(), description)?;
        if result.insert(value.id(), value).is_some() {
            return Err(format!("Duplicate {description} id '{}'.", value.id()));
        }
    }
    Ok(result)
}

trait HasId {
    fn id(&self) -> &str;
}

impl HasId for CourseRecord {
    fn id(&self) -> &str {
        &self.id
    }
}
impl HasId for ChapterRecord {
    fn id(&self) -> &str {
        &self.id
    }
}
impl HasId for Objective {
    fn id(&self) -> &str {
        &self.id
    }
}
impl HasId for ManifestItem {
    fn id(&self) -> &str {
        &self.id
    }
}
impl HasId for SourceRecord {
    fn id(&self) -> &str {
        &self.id
    }
}

fn validate_curriculum(curriculum: &Curriculum) -> Result<(), String> {
    unique_by_id(&curriculum.courses, "course")?;
    let mut chapters = HashMap::new();
    for (course_position, course) in curriculum.courses.iter().enumerate() {
        validate_identifier(&course.id, "course id")?;
        require_text(&course.title, "course title")?;
        if course.chapters.is_empty() {
            return Err(format!("Course '{}' has no chapters.", course.id));
        }
        let mut orders = HashSet::new();
        for chapter in &course.chapters {
            validate_identifier(&chapter.id, "chapter id")?;
            require_text(&chapter.title, "chapter title")?;
            require_text(&chapter.summary, "chapter summary")?;
            require_text(&chapter.project_title, "chapter project title")?;
            if chapter.order == 0 || !orders.insert(chapter.order) {
                return Err(format!(
                    "Course '{}' has a zero or duplicate chapter order.",
                    course.id
                ));
            }
            if chapters
                .insert(chapter.id.clone(), (course_position, chapter.order))
                .is_some()
            {
                return Err(format!("Duplicate chapter id '{}'.", chapter.id));
            }
        }
    }
    for (course_position, course) in curriculum.courses.iter().enumerate() {
        for chapter in &course.chapters {
            for prerequisite_id in &chapter.prerequisite_ids {
                let Some((prerequisite_course, prerequisite_order)) = chapters.get(prerequisite_id)
                else {
                    return Err(format!(
                        "Chapter '{}' references unknown prerequisite '{}'.",
                        chapter.id, prerequisite_id
                    ));
                };
                if *prerequisite_course > course_position
                    || (*prerequisite_course == course_position
                        && *prerequisite_order >= chapter.order)
                {
                    return Err(format!(
                        "Chapter '{}' must depend only on an earlier chapter in its course.",
                        chapter.id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn chapter_lookup(curriculum: &Curriculum) -> HashMap<String, (String, u32)> {
    let mut lookup = HashMap::new();
    for course in &curriculum.courses {
        for chapter in &course.chapters {
            lookup.insert(chapter.id.clone(), (course.id.clone(), chapter.order));
        }
    }
    lookup
}

fn load_item_payload(
    item: &ManifestItem,
    artifact: &Path,
    root: &Path,
    package: &Path,
) -> Result<(Value, String), String> {
    if item.kind == "lesson" {
        let metadata_path = item
            .supporting_artifacts
            .iter()
            .find(|path| path.ends_with(".json"))
            .ok_or_else(|| format!("Lesson '{}' has no metadata JSON companion.", item.id))?;
        let metadata_file = project_file(root, package, metadata_path)?;
        let payload: Value = read_json(&metadata_file)?;
        let body = fs::read_to_string(artifact)
            .map_err(|error| format!("Could not read lesson '{}': {error}", item.id))?;
        Ok((payload, body))
    } else {
        let payload: Value = read_json(artifact)?;
        let body = match item.kind.as_str() {
            "exercise" => value_text(&payload, "prompt")
                .unwrap_or_default()
                .to_owned(),
            "animation" => value_text(&payload, "accessible_summary")
                .unwrap_or_default()
                .to_owned(),
            "lab" | "project" => {
                let brief_path = value_text(&payload, "brief_path")
                    .ok_or_else(|| format!("{} '{}' has no brief_path.", item.kind, item.id))?;
                let brief = package_sibling_file(package, artifact, brief_path)?;
                fs::read_to_string(brief)
                    .map_err(|error| format!("Could not read brief for '{}': {error}", item.id))?
            }
            "lexicon" => value_array(&payload, "entries")
                .unwrap_or_default()
                .iter()
                .filter_map(|entry| {
                    Some(format!(
                        "{} — {}",
                        value_text(entry, "term")?,
                        value_text(entry, "definition")?
                    ))
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        };
        Ok((payload, body))
    }
}

fn validate_record(
    item: &ManifestItem,
    payload: &Value,
    root: &Path,
    package: &Path,
    sources: &HashMap<&str, &SourceRecord>,
) -> Result<(), String> {
    let version = value_u64(payload, "schema_version")
        .ok_or_else(|| format!("Record '{}' has no schema_version.", item.id))?;
    require_version(&format!("record '{}'", item.id), version as u32)?;
    if value_text(payload, "id") != Some(item.id.as_str())
        || value_text(payload, "kind") != Some(item.kind.as_str())
    {
        return Err(format!(
            "Manifest and record identity differ for '{}'.",
            item.id
        ));
    }
    let record_objectives = value_string_array(payload, "objective_ids")?;
    if record_objectives != item.objective_ids {
        return Err(format!(
            "Manifest and record objectives differ for '{}'.",
            item.id
        ));
    }
    if !payload
        .as_object()
        .is_some_and(|object| object.contains_key("citations"))
    {
        return Err(format!("Record '{}' has no citations field.", item.id));
    }
    validate_citations(payload.get("citations"), sources, &item.id)?;

    match item.kind.as_str() {
        "lesson" => {
            let body_path = value_text(payload, "body_path")
                .ok_or_else(|| format!("Lesson '{}' has no body_path.", item.id))?;
            require_text(body_path, "lesson body path")?;
            for field in ["prerequisite_ids", "example_paths", "checkpoint_ids"] {
                if payload.get(field).is_none() {
                    return Err(format!("Lesson '{}' has no {field}.", item.id));
                }
            }
        }
        "exercise" => validate_exercise(payload, &item.id)?,
        "animation" => validate_animation(payload, &item.id)?,
        "lab" | "project" => validate_lab(payload, item, root, package)?,
        "lexicon" => validate_lexicon_shape(payload, sources, &item.id)?,
        _ => unreachable!("kind checked by importer"),
    }
    let artifact = project_file(root, package, &item.artifact)?;
    validate_payload_paths(payload, root, package, &artifact)?;
    Ok(())
}

fn validate_exercise(payload: &Value, id: &str) -> Result<(), String> {
    for field in [
        "prompt",
        "feedback",
        "hints",
        "review_tags",
        "model",
        "answer",
        "error_categories",
    ] {
        if payload.get(field).is_none() {
            return Err(format!("Exercise '{id}' has no {field}."));
        }
    }
    let model = payload
        .get("model")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Exercise '{id}' model is not an object."))?;
    let count = if let Some(values) = model.get("values").and_then(Value::as_array) {
        values.len() as u64
    } else {
        model
            .get("element_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("Exercise '{id}' has no positive element count."))?
    };
    if count == 0 || count > 1_000_000 {
        return Err(format!("Exercise '{id}' has an invalid element count."));
    }
    if let Some(range) = model.get("valid_index_range").and_then(Value::as_array) {
        let valid = range.len() == 2
            && range[0].as_u64() == Some(0)
            && range[1].as_u64() == Some(count - 1);
        if !valid {
            return Err(format!("Exercise '{id}' has invalid array bounds."));
        }
    }
    let answer = payload
        .get("answer")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Exercise '{id}' answer is not an object."))?;
    if let Some(index) = answer
        .get("value")
        .and_then(|value| value.get("index"))
        .and_then(Value::as_u64)
    {
        if index > count
            || (index == count
                && answer.get("type").and_then(Value::as_str) != Some("one_past_classification"))
        {
            return Err(format!(
                "Exercise '{id}' answer index is outside its declared bound."
            ));
        }
    } else if answer.get("type").and_then(Value::as_str) == Some("index") {
        let index = answer
            .get("value")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("Exercise '{id}' index answer is not an unsigned integer."))?;
        if index >= count {
            return Err(format!(
                "Exercise '{id}' answer index is outside its declared bound."
            ));
        }
    }
    let categories = payload
        .get("error_categories")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Exercise '{id}' error_categories is not an array."))?;
    let mut category_ids = HashSet::new();
    let mut otherwise_count = 0;
    for category in categories {
        let category_id = value_text(category, "id")
            .ok_or_else(|| format!("Exercise '{id}' has an unnamed error category."))?;
        if !category_ids.insert(category_id) {
            return Err(format!(
                "Exercise '{id}' repeats error category '{category_id}'."
            ));
        }
        if category.pointer("/when/otherwise").and_then(Value::as_bool) == Some(true) {
            otherwise_count += 1;
        }
        if let Some(rules) = category.pointer("/when/all").and_then(Value::as_array) {
            for rule in rules {
                if !matches!(value_text(rule, "op"), Some("eq" | "neq"))
                    || value_text(rule, "path").is_none()
                    || rule.get("value").is_none()
                {
                    return Err(format!("Exercise '{id}' has an invalid typed error rule."));
                }
            }
        }
    }
    if categories.is_empty() || otherwise_count != 1 {
        return Err(format!(
            "Exercise '{id}' needs one final otherwise error category."
        ));
    }
    Ok(())
}

fn validate_animation(payload: &Value, id: &str) -> Result<(), String> {
    if value_text(payload, "model_type") != Some("conceptual") {
        return Err(format!("Animation '{id}' must declare a conceptual model."));
    }
    let code =
        value_text(payload, "code").ok_or_else(|| format!("Animation '{id}' has no code."))?;
    let line_count = code.lines().count();
    let states = payload
        .get("states")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Animation '{id}' states is not an array."))?;
    if states.is_empty() {
        return Err(format!("Animation '{id}' has no states."));
    }
    let mut state_ids = HashSet::new();
    for state in states {
        let state_id = value_text(state, "id")
            .ok_or_else(|| format!("Animation '{id}' has an unnamed state."))?;
        if !state_ids.insert(state_id) {
            return Err(format!("Animation '{id}' repeats state '{state_id}'."));
        }
        if value_text(state, "caption").is_none_or(|caption| caption.trim().is_empty()) {
            return Err(format!("Animation '{id}' has a blank state caption."));
        }
        if let Some(line) = state.get("source_line")
            && !line.is_null()
            && line
                .as_u64()
                .is_none_or(|line| line == 0 || line as usize > line_count)
        {
            return Err(format!("Animation '{id}' maps a state outside its code."));
        }
        if !matches!(
            value_text(state, "evaluation_status"),
            Some("ready" | "complete" | "invalid-operation")
        ) {
            return Err(format!(
                "Animation '{id}' has an unknown evaluation status."
            ));
        }
    }
    for field in ["initial_state_id", "reset_state_id"] {
        if value_text(payload, field).is_none_or(|state_id| !state_ids.contains(state_id)) {
            return Err(format!("Animation '{id}' has an invalid {field}."));
        }
    }
    if payload.get("keyboard_actions").is_none()
        || payload
            .get("reduced_motion_summary")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        return Err(format!(
            "Animation '{id}' is missing keyboard or static-summary information."
        ));
    }
    Ok(())
}

fn validate_lab(
    payload: &Value,
    item: &ManifestItem,
    root: &Path,
    package: &Path,
) -> Result<(), String> {
    let profile = payload
        .get("profile")
        .filter(|value| value.is_object())
        .ok_or_else(|| format!("{} '{}' has no compiler profile.", item.kind, item.id))?;
    if value_text(profile, "standard") != Some("c17") {
        return Err(format!("{} '{}' does not declare C17.", item.kind, item.id));
    }
    if profile
        .get("compiler_args")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        return Err(format!(
            "{} '{}' has no structured compiler arguments.",
            item.kind, item.id
        ));
    }
    let compile_timeout = value_u64(profile, "compile_timeout_ms").unwrap_or(0);
    let run_timeout = value_u64(profile, "run_timeout_ms").unwrap_or(0);
    let output_limit = value_u64(profile, "output_limit_bytes").unwrap_or(0);
    if !(1..=60_000).contains(&compile_timeout)
        || !(1..=60_000).contains(&run_timeout)
        || !(1..=1_048_576).contains(&output_limit)
    {
        return Err(format!(
            "{} '{}' declares invalid runtime bounds.",
            item.kind, item.id
        ));
    }

    for field in [
        "starter_files",
        "learner_source_paths",
        "solution_files",
        "test_source_paths",
        "fixture_paths",
        "checks",
        "hints",
    ] {
        if payload.get(field).and_then(Value::as_array).is_none() {
            return Err(format!(
                "{} '{}' has no array field '{field}'.",
                item.kind, item.id
            ));
        }
    }
    if payload.get("brief_path").and_then(Value::as_str).is_none() {
        return Err(format!("{} '{}' has no brief path.", item.kind, item.id));
    }
    let brief_path = value_text(payload, "brief_path").expect("checked above");
    let artifact = project_file(root, package, &item.artifact)?;
    package_sibling_file(package, &artifact, brief_path)?;
    let checks = payload
        .get("checks")
        .and_then(Value::as_array)
        .expect("checked above");
    let mut check_ids = HashSet::new();
    for check in checks {
        let check_id = value_text(check, "id")
            .ok_or_else(|| format!("{} '{}' has an unnamed check.", item.kind, item.id))?;
        if !check_ids.insert(check_id) {
            return Err(format!(
                "{} '{}' repeats check '{check_id}'.",
                item.kind, item.id
            ));
        }
    }
    if item.kind == "project" {
        let unlock = value_text(payload, "unlock_chapter_id")
            .ok_or_else(|| format!("Project '{}' has no unlock chapter.", item.id))?;
        if unlock != "foundations-strings-input" {
            return Err(format!(
                "Project '{}' points to an unexpected unlock chapter.",
                item.id
            ));
        }
        let rubric = payload
            .get("rubric")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("Project '{}' has no rubric.", item.id))?;
        if rubric.is_empty() {
            return Err(format!("Project '{}' has an empty rubric.", item.id));
        }
        for row in rubric {
            let objective_ids = value_string_array(row, "objective_ids")?;
            for objective_id in objective_ids {
                if !item.objective_ids.contains(&objective_id) {
                    return Err(format!(
                        "Project '{}' rubric refers to an unknown objective '{objective_id}'.",
                        item.id
                    ));
                }
            }
            if value_text(row, "review_mode")
                .is_none_or(|mode| !matches!(mode, "automated" | "self-review"))
            {
                return Err(format!("Project '{}' has an invalid review mode.", item.id));
            }
            if value_text(row, "review_mode") == Some("automated") {
                let ids = value_string_array(row, "check_ids")?;
                if ids.is_empty() || ids.iter().any(|id| !check_ids.contains(id.as_str())) {
                    return Err(format!(
                        "Project '{}' has a rubric row with missing checks.",
                        item.id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_lexicon_shape(
    payload: &Value,
    sources: &HashMap<&str, &SourceRecord>,
    id: &str,
) -> Result<(), String> {
    let entries = payload
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Lexicon '{id}' has no entries."))?;
    if entries.is_empty() {
        return Err(format!("Lexicon '{id}' is empty."));
    }
    let mut ids = HashSet::new();
    for entry in entries {
        let entry_id = value_text(entry, "id")
            .ok_or_else(|| format!("Lexicon '{id}' has an unnamed entry."))?;
        validate_identifier(entry_id, "lexicon entry id")?;
        if !ids.insert(entry_id) {
            return Err(format!("Lexicon '{id}' repeats entry '{entry_id}'."));
        }
        for field in ["term", "definition", "example", "misconception"] {
            require_text(
                value_text(entry, field).unwrap_or_default(),
                "lexicon field",
            )?;
        }
        validate_citations(entry.get("citations"), sources, entry_id)?;
    }
    Ok(())
}

fn validate_lexicon_references(entries: &[ImportedLexiconEntry]) -> Result<(), String> {
    let ids: HashSet<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
    for entry in entries {
        let related: Vec<String> = serde_json::from_str(&entry.related_ids_json)
            .map_err(|error| format!("Invalid related term list for '{}': {error}", entry.id))?;
        for related_id in related {
            if !ids.contains(related_id.as_str()) {
                return Err(format!(
                    "Lexicon entry '{}' references missing term '{}'.",
                    entry.id, related_id
                ));
            }
        }
    }
    Ok(())
}

fn parse_lexicon(
    payload: &Value,
    sources: &HashMap<&str, &SourceRecord>,
) -> Result<Vec<ImportedLexiconEntry>, String> {
    let entries = payload
        .get("entries")
        .and_then(Value::as_array)
        .expect("validated lexicon");
    let mut output = Vec::with_capacity(entries.len());
    for entry in entries {
        let id = value_text(entry, "id").unwrap_or_default().to_owned();
        let term = value_text(entry, "term").unwrap_or_default().to_owned();
        let definition = value_text(entry, "definition")
            .unwrap_or_default()
            .to_owned();
        let example = value_text(entry, "example").unwrap_or_default().to_owned();
        let misconception = value_text(entry, "misconception")
            .unwrap_or_default()
            .to_owned();
        let related_ids = value_string_array(entry, "related_ids")?;
        let citations = citations_for(entry, sources)?;
        let search_text = format!("{term} {definition} {example} {misconception}");
        output.push(ImportedLexiconEntry {
            id,
            term,
            category: "Memory".into(),
            definition,
            example,
            misconception,
            related_ids_json: serde_json::to_string(&related_ids)
                .map_err(|error| error.to_string())?,
            citations_json: serde_json::to_string(&citations).map_err(|error| error.to_string())?,
            search_text,
        });
    }
    Ok(output)
}

fn validate_coverage(
    coverage: &[Coverage],
    objectives: &HashMap<&str, &Objective>,
    items: &HashMap<&str, &ManifestItem>,
    kinds: &HashMap<String, String>,
) -> Result<(), String> {
    let mut covered = HashSet::new();
    for entry in coverage {
        if !objectives.contains_key(entry.objective_id.as_str())
            || !covered.insert(entry.objective_id.as_str())
        {
            return Err(format!(
                "Coverage has an unknown or duplicate objective '{}'.",
                entry.objective_id
            ));
        }
        require_kind(items, kinds, &entry.lesson_id, "lesson")?;
        if entry.practice_ids.is_empty() {
            return Err(format!(
                "Objective '{}' has no practice record.",
                entry.objective_id
            ));
        }
        for practice_id in &entry.practice_ids {
            if !items.contains_key(practice_id.as_str())
                || !matches!(
                    kinds.get(practice_id).map(String::as_str),
                    Some("exercise" | "animation" | "lab")
                )
            {
                return Err(format!(
                    "Coverage item '{practice_id}' must refer to an exercise, animation, or lab."
                ));
            }
        }
        require_kind(items, kinds, &entry.project_id, "project")?;
    }
    if covered.len() != objectives.len() {
        return Err("The coverage map does not include every pilot objective.".into());
    }
    Ok(())
}

fn require_kind(
    items: &HashMap<&str, &ManifestItem>,
    kinds: &HashMap<String, String>,
    id: &str,
    expected: &str,
) -> Result<(), String> {
    if !items.contains_key(id) || kinds.get(id).map(String::as_str) != Some(expected) {
        return Err(format!("Coverage item '{id}' must refer to a {expected}."));
    }
    Ok(())
}

fn validate_evidence(
    item: &ManifestItem,
    evidence: Option<&EvidenceRecord>,
    root: &Path,
    package: &Path,
) -> Result<(), String> {
    if item.status != "verified" {
        return Ok(());
    }
    let evidence =
        evidence.ok_or_else(|| format!("Verified item '{}' has no evidence record.", item.id))?;
    if !evidence.reviewed
        || evidence.results.is_empty()
        || evidence.results.iter().any(|result| !result.passed)
    {
        return Err(format!(
            "Verified item '{}' has incomplete or failing evidence.",
            item.id
        ));
    }
    let expected_checks: HashSet<&str> = item.checks.iter().map(String::as_str).collect();
    let observed_checks: HashSet<&str> = evidence
        .results
        .iter()
        .map(|result| result.check.as_str())
        .collect();
    if expected_checks.len() != item.checks.len()
        || observed_checks.len() != evidence.results.len()
        || expected_checks.is_empty()
        || !expected_checks.is_subset(&observed_checks)
    {
        return Err(format!(
            "Verified item '{}' has missing, duplicate, or unrecognized evidence checks.",
            item.id
        ));
    }
    let artifact = project_file(root, package, &item.artifact)?;
    if sha256_file(&artifact)? != evidence.artifact_sha256 {
        return Err(format!(
            "Verified item '{}' has a stale artifact hash.",
            item.id
        ));
    }

    let expected_support: HashSet<&str> = item
        .supporting_artifacts
        .iter()
        .map(String::as_str)
        .collect();
    let recorded_support: HashSet<&str> = evidence
        .supporting_artifact_sha256
        .keys()
        .map(String::as_str)
        .collect();
    if expected_support != recorded_support {
        return Err(format!(
            "Verified item '{}' has an incomplete supporting-artifact hash map.",
            item.id
        ));
    }
    for (path, expected) in &evidence.supporting_artifact_sha256 {
        let file = project_file(root, package, path)?;
        if sha256_file(&file)? != *expected {
            return Err(format!(
                "Verified item '{}' has a stale supporting-artifact hash for '{path}'.",
                item.id
            ));
        }
    }

    if matches!(item.kind.as_str(), "lab" | "project") {
        if evidence.package_files.is_empty() {
            return Err(format!(
                "Verified package '{}' has no package file hashes.",
                item.id
            ));
        }
        for (path, expected) in &evidence.package_files {
            let file = project_file(root, package, path)?;
            if sha256_file(&file)? != *expected {
                return Err(format!(
                    "Verified package '{}' has a stale hash for '{path}'.",
                    item.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_citations(
    citations: Option<&Value>,
    sources: &HashMap<&str, &SourceRecord>,
    owner: &str,
) -> Result<(), String> {
    let citations = citations
        .and_then(Value::as_array)
        .ok_or_else(|| format!("'{owner}' citations are not an array."))?;
    for citation in citations {
        let source_id = value_text(citation, "source_id")
            .ok_or_else(|| format!("'{owner}' has a citation without source_id."))?;
        let source = sources
            .get(source_id)
            .ok_or_else(|| format!("'{owner}' cites unknown source '{source_id}'."))?;
        if source.status != "reviewed" {
            return Err(format!("'{owner}' cites unreviewed source '{source_id}'."));
        }
        require_text(
            value_text(citation, "locator").unwrap_or_default(),
            "citation locator",
        )?;
        require_text(
            value_text(citation, "claim").unwrap_or_default(),
            "citation claim",
        )?;
    }
    Ok(())
}

fn citations_for(
    record: &Value,
    sources: &HashMap<&str, &SourceRecord>,
) -> Result<Vec<Value>, String> {
    let mut output = Vec::new();
    let direct = record
        .get("citations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for citation in direct {
        let source_id = value_text(citation, "source_id").unwrap_or_default();
        let source = sources
            .get(source_id)
            .ok_or_else(|| format!("Unknown citation source '{source_id}'."))?;
        let mut enriched = citation.as_object().cloned().unwrap_or_default();
        enriched.insert("source_title".into(), Value::String(source.title.clone()));
        enriched.insert("source_url".into(), Value::String(source.url.clone()));
        output.push(Value::Object(enriched));
    }
    Ok(output)
}

fn validate_payload_paths(
    payload: &Value,
    root: &Path,
    package: &Path,
    artifact: &Path,
) -> Result<(), String> {
    match payload {
        Value::Object(map) => {
            for (key, value) in map {
                if key.ends_with("_path") {
                    if let Some(path) = value.as_str() {
                        resolve_item_reference(root, package, artifact, path)?;
                    } else if !value.is_null() {
                        return Err(format!("Content path field '{key}' must be a string."));
                    }
                } else if key.ends_with("_paths") {
                    let paths = value
                        .as_array()
                        .ok_or_else(|| format!("Content paths field '{key}' must be an array."))?;
                    for path in paths {
                        let path = path.as_str().ok_or_else(|| {
                            format!("Content paths field '{key}' contains a non-string value.")
                        })?;
                        resolve_item_reference(root, package, artifact, path)?;
                    }
                } else {
                    validate_payload_paths(value, root, package, artifact)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_payload_paths(value, root, package, artifact)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_payload_paths(
    payload: &Value,
    item: &ManifestItem,
    paths: &mut HashSet<String>,
) -> Result<(), String> {
    match payload {
        Value::Object(map) => {
            for (key, value) in map {
                if key.ends_with("_path") {
                    if let Some(path) = value.as_str() {
                        paths.insert(item_reference(item, path)?);
                    }
                } else if key.ends_with("_paths") {
                    if let Some(values) = value.as_array() {
                        for value in values.iter().filter_map(Value::as_str) {
                            paths.insert(item_reference(item, value)?);
                        }
                    }
                } else {
                    collect_payload_paths(value, item, paths)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_payload_paths(value, item, paths)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn item_reference(item: &ManifestItem, path: &str) -> Result<String, String> {
    if path.starts_with(PACKAGE_PREFIX) {
        Ok(path.to_owned())
    } else {
        validate_relative_reference(path)?;
        let artifact = item
            .artifact
            .strip_prefix(PACKAGE_PREFIX)
            .ok_or_else(|| format!("Item '{}' is outside its content package.", item.id))?;
        let parent = Path::new(artifact)
            .parent()
            .ok_or_else(|| format!("Item '{}' has no package directory.", item.id))?;
        let relative = parent.join(path).to_string_lossy().replace('\\', "/");
        Ok(format!("{PACKAGE_PREFIX}{relative}"))
    }
}

fn resolve_item_reference(
    root: &Path,
    package: &Path,
    artifact: &Path,
    path: &str,
) -> Result<PathBuf, String> {
    if path.starts_with(PACKAGE_PREFIX) {
        project_file(root, package, path)
    } else {
        package_sibling_file(package, artifact, path)
    }
}

fn validate_relative_reference(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains('\\')
        || path.starts_with('/')
        || path.contains(':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("Unsafe content-relative path '{path}'."));
    }
    Ok(())
}

fn package_sibling_file(
    package: &Path,
    artifact: &Path,
    relative: &str,
) -> Result<PathBuf, String> {
    validate_relative_reference(relative)?;
    let parent = artifact
        .parent()
        .ok_or_else(|| "Content artifact has no parent directory.".to_owned())?;
    let canonical = parent
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("Referenced content path '{relative}' is unavailable: {error}"))?;
    if !canonical.starts_with(package) || !canonical.is_file() {
        return Err(format!(
            "Referenced content path '{relative}' escapes its package or is not a file."
        ));
    }
    Ok(canonical)
}

fn project_file(root: &Path, package: &Path, relative: &str) -> Result<PathBuf, String> {
    let Some(package_relative) = relative.strip_prefix(PACKAGE_PREFIX) else {
        return Err(format!(
            "Content path '{relative}' is outside the Arrays & memory package."
        ));
    };
    package_file(package, package_relative)
        .map_err(|error| format!("{error} (from {})", root.display()))
}

fn package_file(package: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative.starts_with('/')
        || relative.contains(':')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("Unsafe package-relative path '{relative}'."));
    }
    let candidate = package.join(relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("Referenced content path '{relative}' is unavailable: {error}"))?;
    if !canonical.starts_with(package) || !canonical.is_file() {
        return Err(format!(
            "Referenced content path '{relative}' escapes its package or is not a file."
        ));
    }
    Ok(canonical)
}

fn package_digest(
    root: &Path,
    package: &Path,
    project_paths: &HashSet<String>,
) -> Result<String, String> {
    let mut files = vec![
        "foundation/curriculum.json".to_owned(),
        "foundation/pilot.json".to_owned(),
        "foundation/evidence.json".to_owned(),
        "foundation/sources.json".to_owned(),
    ];
    files.extend(project_paths.iter().cloned());
    let mut sorted = files;
    sorted.sort();
    sorted.dedup();
    let mut digest = Sha256::new();
    for project_path in sorted {
        let file = if project_path.starts_with(PACKAGE_PREFIX) {
            project_file(root, package, &project_path)?
        } else {
            let safe = safe_root_relative(&project_path)?;
            let candidate = root.join(safe);
            let canonical = candidate.canonicalize().map_err(|error| {
                format!("Could not read catalog input '{project_path}': {error}")
            })?;
            if !canonical.starts_with(root) || !canonical.is_file() {
                return Err(format!(
                    "Catalog input '{project_path}' resolves outside the bundle."
                ));
            }
            canonical
        };
        let bytes = fs::read(&file)
            .map_err(|error| format!("Could not hash {}: {error}", file.display()))?;
        digest.update(project_path.as_bytes());
        digest.update([0]);
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn safe_root_relative(relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative.starts_with('/')
        || relative.contains(':')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("Unsafe catalog resource path '{relative}'."));
    }
    Ok(PathBuf::from(relative))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("Could not hash {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn value_text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn value_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn value_array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn value_string_array(value: &Value, key: &str) -> Result<Vec<String>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Field '{key}' must be an array."))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("Field '{key}' contains a non-string value."))
        })
        .collect()
}

fn course_summary(id: &str) -> &'static str {
    match id {
        "foundations" => "From the first executable to memory you can reason about.",
        "systems" => "Build useful tools around data, files, processes, and interfaces.",
        "reliable" => "Defend invariants, diagnose faults, and handle hostile input.",
        _ => "A local C learning path.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_reject_parent_and_absolute_forms() {
        let root = std::env::temp_dir();
        for path in [
            "../outside.c",
            "lessons/../../outside.c",
            "/tmp/outside.c",
            "C:/outside.c",
            "starter\\bad.c",
        ] {
            assert!(package_file(&root, path).is_err(), "accepted {path}");
        }
    }

    #[test]
    fn exercise_bounds_reject_an_invalid_last_index() {
        let mut record = serde_json::json!({
            "prompt":"Select an element.",
            "feedback":{},
            "hints":["Count the elements."],
            "review_tags":["bounds"],
            "model":{"element_count":6,"valid_index_range":[0,6]},
            "answer":{"type":"index","value":5},
            "error_categories":[{"id":"other","when":{"otherwise":true}}]
        });
        assert!(validate_exercise(&record, "test-exercise").is_err());
        record["model"]["valid_index_range"][1] = Value::from(5);
        assert!(validate_exercise(&record, "test-exercise").is_ok());
    }
}
