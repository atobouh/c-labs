//! Versioned learner state and trusted, bundled-package grading.
use crate::{
    catalog,
    runner::{self, CheckResult, Compiler, JobControl, JobResult, SourceFile},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Instant,
};

pub type Learner = Arc<Mutex<Result<Connection, String>>>;
pub struct Package {
    pub id: String,
    pub manifest: Value,
    pub files: BTreeMap<String, String>,
    pub hash: String,
    pub revision: u32,
}

fn strings(value: &Value) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or("Expected a package path list.")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("Expected a package path.".into())
        })
        .collect()
}
pub fn example(connection: &Connection, root: &Path, id: &str) -> Result<Vec<SourceFile>, String> {
    let item = catalog::item(connection, id)?.ok_or("Unknown lesson.")?;
    if item.kind != "lesson" {
        return Err("Examples belong to lesson records.".into());
    }
    let evidence: Value = serde_json::from_str(&read(root, "foundation/evidence.json")?)
        .map_err(|e| e.to_string())?;
    let record = evidence["records"]
        .as_array()
        .ok_or("Missing lesson evidence.")?
        .iter()
        .find(|r| r["item_id"] == id)
        .ok_or("Missing lesson evidence.")?;
    let mut files = Vec::new();
    for path in strings(&item.payload["example_paths"])? {
        let text = read(root, &path)?;
        if record["supporting_artifact_sha256"][&path]
            != format!("{:x}", Sha256::digest(text.as_bytes()))
        {
            return Err("Lesson example fingerprint mismatch.".into());
        }
        files.push(SourceFile {
            path: Path::new(&path)
                .file_name()
                .ok_or("Invalid example path.")?
                .to_string_lossy()
                .into_owned(),
            text,
        });
    }
    runner::validate_files(&files)?;
    Ok(files)
}
fn read(root: &Path, relative: &str) -> Result<String, String> {
    let resolved = root
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !resolved.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
        return Err("Content path escapes the bundle.".into());
    }
    std::fs::read_to_string(resolved).map_err(|e| e.to_string())
}
pub fn pilot(connection: &Connection, root: &Path) -> Result<Value, String> {
    let model: Value =
        serde_json::from_str(&read(root, "foundation/pilot.json")?).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for entry in model["items"].as_array().ok_or("Missing pilot records.")? {
        if entry["kind"] == "lexicon" {
            continue;
        }
        let item = catalog::item(
            connection,
            entry["id"].as_str().ok_or("Missing record ID.")?,
        )?
        .ok_or("Missing catalog record.")?;
        items.push(serde_json::to_value(item).map_err(|e| e.to_string())?);
    }
    let content_hash: String = connection
        .query_row(
            "SELECT package_sha256 FROM catalog_meta WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(
        json!({"items":items,"objectives":model["objectives"],"prerequisite_skills":model["prerequisite_skills"],"content_hash":content_hash}),
    )
}
pub fn load_package(connection: &Connection, root: &Path, id: &str) -> Result<Package, String> {
    let item = catalog::item(connection, id)?.ok_or("Unknown learning package.")?;
    if !matches!(item.kind.as_str(), "lab" | "project") {
        return Err("This record is not an executable package.".into());
    }
    let evidence: Value = serde_json::from_str(&read(root, "foundation/evidence.json")?)
        .map_err(|e| e.to_string())?;
    let record = evidence["records"]
        .as_array()
        .ok_or("Missing evidence.")?
        .iter()
        .find(|v| v["item_id"] == id)
        .ok_or("Package has no evidence.")?;
    let mut files = BTreeMap::new();
    let directory = Path::new(&item.artifact)
        .parent()
        .ok_or("Invalid package path.")?;
    for (path, expected) in record["package_files"]
        .as_object()
        .ok_or("Missing package fingerprints.")?
    {
        let text = read(root, path)?;
        if format!("{:x}", Sha256::digest(text.as_bytes())) != expected.as_str().unwrap_or("") {
            return Err(format!("Bundled evidence mismatch: {path}"));
        }
        let relative = Path::new(path)
            .strip_prefix(directory)
            .map_err(|_| "Package file escapes its directory.")?
            .to_string_lossy()
            .replace('\\', "/");
        files.insert(relative, text);
    }
    let manifest: Value = serde_json::from_str(
        files
            .get(
                Path::new(&item.artifact)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .ok_or("Missing manifest bytes.")?,
    )
    .map_err(|e| e.to_string())?;
    let (revision, hash): (u32, String) = connection
        .query_row(
            "SELECT content_revision,package_sha256 FROM catalog_meta WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(Package {
        id: id.into(),
        manifest,
        files,
        hash,
        revision,
    })
}
pub fn state(connection: &Connection) -> Result<Value, String> {
    let mut values = serde_json::Map::new();
    let mut statement = connection
        .prepare("SELECT id,value FROM learning_state")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (id, text) = row.map_err(|e| e.to_string())?;
        values.insert(id, serde_json::from_str(&text).map_err(|e| e.to_string())?);
    }
    let mut attempts = Vec::new();
    let mut statement = connection
        .prepare("SELECT result_json,created_at FROM attempts ORDER BY rowid DESC LIMIT 100")
        .map_err(|e| e.to_string())?;
    for row in statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
    {
        let (text, time) = row.map_err(|e| e.to_string())?;
        let mut value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        value["created_at"] = json!(time);
        attempts.push(value);
    }
    let mut gates = Vec::new();
    let mut statement = connection
        .prepare("SELECT chapter_id,attempt_id,content_hash FROM chapter_gates")
        .map_err(|e| e.to_string())?;
    for row in statement.query_map([],|r|Ok(json!({"chapter_id":r.get::<_,String>(0)?,"attempt_id":r.get::<_,String>(1)?,"content_hash":r.get::<_,String>(2)?}))).map_err(|e|e.to_string())? {gates.push(row.map_err(|e|e.to_string())?);}
    Ok(json!({"values":values,"attempts":attempts,"gates":gates}))
}
fn put(connection: &Connection, id: &str, value: &Value) -> Result<(), String> {
    connection.execute("INSERT INTO learning_state(id,value) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value,updated_at=CURRENT_TIMESTAMP",params![id,value.to_string()]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn save_progress(connection: &Connection, id: &str, value: &Value) -> Result<(), String> {
    if value.to_string().len() > 8192
        || !(id == "preferences"
            || id == "resume"
            || id.starts_with("lesson-")
            || id.starts_with("animation-")
            || id.starts_with("hint-"))
    {
        return Err("Unsupported or oversized learner preference.".into());
    }
    put(connection, id, value)
}
pub fn validate_submission(package: &Package, files: &[SourceFile]) -> Result<(), String> {
    runner::validate_files(files)?;
    let declared: BTreeSet<_> = strings(&package.manifest["learner_source_paths"])?
        .into_iter()
        .collect();
    let submitted: BTreeSet<_> = files.iter().map(|f| f.path.clone()).collect();
    if declared != submitted {
        return Err(
            "Submit exactly the declared learner files; supplied headers and tests are read-only."
                .into(),
        );
    }
    Ok(())
}
pub fn save_submission(
    connection: &Connection,
    package: &Package,
    files: &[SourceFile],
) -> Result<(), String> {
    validate_submission(package, files)?;
    put(
        connection,
        &format!("draft-{}", package.id),
        &json!({"files":files,"submission_hash":runner::submission_hash(files)}),
    )
}
pub fn package_view(connection: &Connection, package: &Package) -> Result<Value, String> {
    let starter: Vec<_> = strings(&package.manifest["starter_files"])?
        .iter()
        .map(|path| json!({"path":path,"text":package.files.get(path)}))
        .collect();
    let headers: Vec<_> = package
        .files
        .iter()
        .filter(|(path, _)| path.starts_with("include/"))
        .map(|(path, text)| json!({"path":path,"text":text}))
        .collect();
    let mut manifest = package.manifest.clone();
    for key in [
        "solution_files",
        "mutations",
        "test_source_paths",
        "reference_only_boundary_test",
    ] {
        manifest.as_object_mut().unwrap().remove(key);
    }
    let brief = package
        .files
        .get(package.manifest["brief_path"].as_str().unwrap_or(""))
        .ok_or("Missing package brief.")?;
    Ok(
        json!({"id":package.id,"manifest":manifest,"brief":brief,"api":package.files.get("api.md"),"starter":starter,"headers":headers,"content_hash":package.hash,"content_revision":package.revision,"learner":state(connection)?}),
    )
}
pub fn exercise(
    connection: &mut Connection,
    record: &catalog::CatalogItemDetail,
    answer: Value,
    hint_level: u32,
) -> Result<Value, String> {
    if record.kind != "exercise" || answer.to_string().len() > 1024 || hint_level > 3 {
        return Err("Invalid exercise submission.".into());
    }
    let model = &record.payload;
    let correct = answer == model["answer"]["value"];
    let fields = if answer.is_object() {
        answer.clone()
    } else {
        json!({"value":answer})
    };
    let category = if correct {
        "correct".into()
    } else {
        model["error_categories"]
            .as_array()
            .ok_or("Missing feedback rules.")?
            .iter()
            .find(|rule| {
                rule["when"]["otherwise"] == true
                    || rule["when"]["all"].as_array().is_some_and(|terms| {
                        terms.iter().all(|term| {
                            let actual = &fields[term["path"].as_str().unwrap_or("")];
                            match term["op"].as_str() {
                                Some("eq") => actual == &term["value"],
                                Some("neq") => actual != &term["value"],
                                _ => false,
                            }
                        })
                    })
            })
            .and_then(|r| r["id"].as_str())
            .unwrap_or("other")
            .to_owned()
    };
    let feedback = if correct {
        model["feedback"]["correct"].clone()
    } else {
        model["feedback"]["incorrect"][&category].clone()
    };
    let result = json!({"correct":correct,"category":category,"feedback":feedback,"answer":answer,"hint_level":hint_level});
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO exercise_attempts(record_id,answer_json,correct,category,hint_level) VALUES(?1,?2,?3,?4,?5)",params![record.id,answer.to_string(),correct,category,hint_level]).map_err(|e|e.to_string())?;
    put(&tx, &record.id, &result)?;
    if !correct {
        put(
            &tx,
            &format!("review-{}", record.id),
            &json!({"id":record.id,"title":record.title,"prompt":model["prompt"],"feedback":feedback,"category":category,"active":true}),
        )?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}
pub fn review(connection: &Connection, id: &str) -> Result<(), String> {
    let key = format!("review-{id}");
    let text: Option<String> = connection
        .query_row(
            "SELECT value FROM learning_state WHERE id=?1",
            [&key],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let mut value: Value =
        serde_json::from_str(&text.ok_or("No review card exists for this item.")?)
            .map_err(|e| e.to_string())?;
    value["active"] = json!(false);
    put(connection, &key, &value)
}
pub fn persist_attempt(
    connection: &mut Connection,
    package: &Package,
    files: &[SourceFile],
    result: &JobResult,
    self_review: bool,
) -> Result<(), String> {
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    let changed=tx.execute("INSERT INTO attempts(id,package_id,submission_hash,content_hash,rubric_version,result_json,files_json) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET result_json=excluded.result_json WHERE json_extract(attempts.result_json,'$.status')='running'",params![result.job_id,package.id,result.submission_hash,package.hash,result.rubric_version.unwrap_or(1),serde_json::to_string(result).map_err(|e|e.to_string())?,serde_json::to_string(files).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    if changed != 1 {
        return Err("Completed attempts are immutable.".into());
    }
    let draft: Option<String> = tx
        .query_row(
            "SELECT value FROM learning_state WHERE id=?1",
            [format!("draft-{}", package.id)],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let latest: Option<String> = tx
        .query_row(
            "SELECT value FROM learning_state WHERE id=?1",
            [format!("job-{}", package.id)],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let current = draft
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .is_some_and(|d| d["submission_hash"] == result.submission_hash)
        && latest
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .is_some_and(|v| v == result.job_id);
    if current
        && self_review
        && result.status == "passed"
        && result.package_id.as_deref() == Some("project-sensor")
        && result.content_hash.as_deref() == Some(&package.hash)
        && result.content_revision == Some(package.revision)
        && result.rubric_version
            == package.manifest["rubric_version"]
                .as_u64()
                .map(|v| v as u32)
    {
        tx.execute("INSERT INTO chapter_gates(chapter_id,attempt_id,content_hash) VALUES(?1,?2,?3) ON CONFLICT(chapter_id) DO UPDATE SET attempt_id=excluded.attempt_id,content_hash=excluded.content_hash",params![package.manifest["unlock_chapter_id"].as_str().ok_or("Missing unlock target.")?,result.job_id,package.hash]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}
pub fn start_grade(
    runner: Arc<runner::Runner>,
    learner: Learner,
    package: Package,
    files: Vec<SourceFile>,
    self_review: bool,
) -> Result<String, String> {
    validate_submission(&package, &files)?;
    let compiler = runner.toolchain();
    if !compiler.available {
        return Err(compiler.message);
    }
    runner.start(move |id, control, root| {
        {
            let mut connection = learner.lock().map_err(|e| e.to_string())?;
            let connection=connection.as_mut().map_err(|e|e.clone())?;
            let transaction=connection.transaction().map_err(|e|e.to_string())?;
            put(
                &transaction,
                &format!("job-{}", package.id),
                &json!(id),
            )?;
            let mut pending=runner::error_result(&id,"running","Grading began; an interrupted attempt cannot unlock a chapter.",0);
            pending.package_id=Some(package.id.clone());pending.submission_hash=runner::submission_hash(&files);pending.content_hash=Some(package.hash.clone());pending.content_revision=Some(package.revision);pending.rubric_version=Some(package.manifest["rubric_version"].as_u64().unwrap_or(1) as u32);pending.toolchain=compiler.identity.clone();
            transaction.execute("INSERT INTO attempts(id,package_id,submission_hash,content_hash,rubric_version,result_json,files_json) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,package.id,pending.submission_hash,package.hash,pending.rubric_version,serde_json::to_string(&pending).map_err(|e|e.to_string())?,serde_json::to_string(&files).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
            transaction.commit().map_err(|e|e.to_string())?;
        }
        let started = Instant::now();
        let mut result = match grade(&id, &control, &root, &compiler, &package, &files) {
            Ok(r) => r,
            Err(e) => runner::error_result(&id, "internal-error", &e, 0),
        };
        result.package_id = Some(package.id.clone());
        result.submission_hash = runner::submission_hash(&files);
        result.content_hash = Some(package.hash.clone());
        result.content_revision = Some(package.revision);
        result.rubric_version =
            Some(package.manifest["rubric_version"].as_u64().unwrap_or(1) as u32);
        result.toolchain = compiler.identity.clone();
        result.elapsed_ms = started.elapsed().as_millis() as u64;
        if control.cancelled.load(Ordering::Relaxed) {
            result.status = "cancelled".into();
            result.message = "Cancelled; no chapter unlock was granted.".into();
        }
        let mut connection = learner.lock().map_err(|e| e.to_string())?;
        persist_attempt(
            connection.as_mut().map_err(|e| e.clone())?,
            &package,
            &files,
            &result,
            self_review,
        )?;
        Ok(result)
    })
}

fn compile(
    root: &Path,
    compiler: &Compiler,
    control: &JobControl,
    sources: &[String],
    output: &Path,
) -> Result<runner::ProcessOutput, String> {
    let mut command = runner::compiler_command(compiler, root, "C17");
    command.args(["-Werror", "-Iinclude", "-Ifixtures"]);
    command.args(sources);
    command.arg("-o").arg(output);
    runner::run_process(&mut command, control, "compiling", 10_000, 65_536)
}
fn check(value: &Value, passed: bool, message: &str) -> CheckResult {
    CheckResult {
        id: value["id"].as_str().unwrap_or("").into(),
        description: value["description"].as_str().unwrap_or("").into(),
        status: if passed { "passed" } else { "failed" }.into(),
        message: message.into(),
    }
}
fn grade(
    id: &str,
    control: &JobControl,
    root: &Path,
    compiler: &Compiler,
    package: &Package,
    files: &[SourceFile],
) -> Result<JobResult, String> {
    // Copy only supplied support/test files. Instructor solutions never enter this workspace.
    for (path, text) in &package.files {
        if path.starts_with("include/")
            || path.starts_with("fixtures/")
            || path.starts_with("tests/")
        {
            let target = root.join(path);
            std::fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(target, text).map_err(|e| e.to_string())?;
        }
    }
    runner::write_files(root, files)?;
    let project = package.manifest["kind"] == "project";
    let mut sources: Vec<String> = files
        .iter()
        .filter(|f| f.path.ends_with(".c"))
        .map(|f| f.path.clone())
        .collect();
    let learner_sources = sources.clone();
    let tests = strings(&package.manifest["test_source_paths"])?;
    sources.extend(
        tests
            .into_iter()
            .filter(|p| !project || p == "tests/test_driver.c"),
    );
    let output = root.join("graded.exe");
    let compiled = compile(root, compiler, control, &sources, &output)?;
    let mut result = runner::error_result(id, "failed", "Some required checks failed.", 0);
    if compiled.status != "passed" {
        result.status = if compiled.status == "failed" {
            "compile-error".into()
        } else {
            compiled.status
        };
        result.message = "Compilation did not complete successfully.".into();
        result.stdout = compiled.stdout;
        result.stderr = compiled.stderr;
        result.exit_code = compiled.exit_code;
        result.truncated = compiled.truncated;
        return Ok(result);
    }
    let checks = package.manifest["checks"]
        .as_array()
        .ok_or("Missing checks.")?;
    let mut command = Command::new(&output);
    command.current_dir(root);
    let executed = runner::run_process(&mut command, control, "running checks", 2_000, 65_536)?;
    result.exit_code = executed.exit_code;
    result.stdout = executed.stdout.clone();
    result.stderr = executed.stderr.clone();
    result.truncated = executed.truncated;
    if !matches!(executed.status.as_str(), "passed" | "failed") {
        result.status = executed.status;
        result.message = "Grading run stopped before completing its checks.".into();
        return Ok(result);
    }
    let mut assertions = BTreeMap::new();
    let mut report_ids = Vec::new();
    if !project && !executed.stderr.is_empty() {
        return Err("Lab wrote unexpected stderr.".into());
    }
    for line in (if project {
        &executed.stderr
    } else {
        &executed.stdout
    })
    .lines()
    {
        let event: Value =
            serde_json::from_str(line).map_err(|_| "Invalid structured grader output.")?;
        if event.as_object().is_some_and(|o| o.len() == 2)
            && event["check_id"].is_string()
            && event["passed"].is_boolean()
        {
            if assertions
                .insert(
                    event["check_id"].as_str().unwrap().to_owned(),
                    event["passed"].as_bool().unwrap(),
                )
                .is_some()
            {
                return Err("Duplicate grader assertion.".into());
            }
        } else if project
            && event.as_object().is_some_and(|o| o.len() == 1)
            && event["report_id"].is_string()
        {
            report_ids.push(event["report_id"].as_str().unwrap().to_owned());
        } else {
            return Err("Unknown structured grader event.".into());
        }
    }
    let declared: BTreeSet<String> = checks
        .iter()
        .filter(|c| c["kind"] == "assertion-harness")
        .map(|c| c["id"].as_str().unwrap().into())
        .collect();
    if assertions.keys().cloned().collect::<BTreeSet<_>>() != declared
        || executed.exit_code
            != Some(if assertions.values().all(|v| *v) {
                0
            } else {
                1
            })
    {
        return Err("Grader assertions or exit status disagree with the package contract.".into());
    }
    result.checks.push(check(
        &checks[0],
        true,
        "C17 build completed without warnings.",
    ));
    for value in checks.iter().filter(|c| c["kind"] == "assertion-harness") {
        result.checks.push(check(
            value,
            assertions[value["id"].as_str().unwrap()],
            "Real harness assertion.",
        ));
    }
    if project {
        let reports: Vec<_> = checks
            .iter()
            .filter(|c| c["kind"] == "stdout" && c["scope"] != "adapter")
            .collect();
        let ids: Vec<_> = reports
            .iter()
            .map(|c| c["id"].as_str().unwrap().to_owned())
            .collect();
        if ids != report_ids {
            return Err("Report identifiers/order disagree with the package.".into());
        }
        let lines: Vec<_> = executed.stdout.lines().collect();
        for (i, value) in reports.iter().enumerate() {
            result.checks.push(check(
                value,
                lines.len() == reports.len()
                    && executed.stdout.ends_with('\n')
                    && lines
                        .get(i)
                        .is_some_and(|s| compare_report(s, &value["expected"])),
                "Exact report schema, order, types, and numeric values.",
            ));
        }
        let mut adapter_sources = learner_sources;
        adapter_sources.extend(strings(&package.manifest["provided_adapter_paths"])?);
        let adapter = root.join("adapter.exe");
        let built = compile(root, compiler, control, &adapter_sources, &adapter)?;
        let expected = checks
            .iter()
            .find(|c| c["id"] == package.manifest["adapter_check_id"])
            .ok_or("Missing adapter check.")?;
        if built.status == "passed" {
            let run = runner::run_process(
                Command::new(&adapter).current_dir(root),
                control,
                "checking adapter",
                2_000,
                65_536,
            )?;
            result.checks.push(check(
                expected,
                run.status == "passed"
                    && run.stderr.is_empty()
                    && run.stdout.ends_with('\n')
                    && compare_report(run.stdout.trim(), &expected["expected"]),
                "Independently compiled supplied adapter.",
            ));
            if !matches!(run.status.as_str(), "passed" | "failed") {
                result.status = run.status;
                return Ok(result);
            }
        } else {
            result
                .checks
                .push(check(expected, false, "Adapter compilation failed."));
            result.stderr = built.stderr;
            if built.status != "failed" {
                result.status = built.status;
                return Ok(result);
            }
        }
    }
    for value in checks.iter().filter(|c| c["required"] == false) {
        let mut c = check(
            value,
            false,
            "Optional profile has not been verified on this toolchain.",
        );
        c.status = "unavailable".into();
        result.checks.push(c);
    }
    if result
        .checks
        .iter()
        .all(|c| matches!(c.status.as_str(), "passed" | "unavailable"))
    {
        result.status = "passed".into();
        result.message =
            "All required automated checks passed. Explanation remains learner self-review.".into();
    }
    runner::bound_text(&mut result.stdout, &mut result.stderr, 65_536);
    Ok(result)
}

// Preserve wire key order and reject duplicates; serde_json::Value alone sorts keys.
struct OrderedReport(Vec<(String, Value)>);
impl<'de> Deserialize<'de> for OrderedReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = OrderedReport;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a report object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = map.next_entry()? {
                    pairs.push(pair);
                }
                Ok(OrderedReport(pairs))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
fn compare_report(text: &str, expected: &Value) -> bool {
    let Ok(report) = serde_json::from_str::<OrderedReport>(text) else {
        return false;
    };
    let keys = ["status", "count", "sum", "minimum", "maximum", "mean"];
    report.0.len() == keys.len()
        && report.0.iter().zip(keys).all(|((key, value), name)| {
            if key != name {
                return false;
            }
            let want = &expected[name];
            if name == "mean" && !want.is_null() {
                value.as_f64().zip(want.as_f64()).is_some_and(|(a, b)| {
                    a.is_finite()
                        && (a - b).abs() <= expected["mean_tolerance"].as_f64().unwrap_or(0.0)
                })
            } else if matches!(name, "count" | "sum" | "minimum" | "maximum") && !want.is_null() {
                value.is_i64() && value == want
            } else {
                value == want
            }
        })
}

#[cfg(test)]
mod tests;
