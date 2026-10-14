use super::*;
use crate::storage;
use std::path::PathBuf;

fn catalog_fixture() -> (Connection, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();
    let mut catalog = Connection::open_in_memory().unwrap();
    catalog::sync(&mut catalog, &root).unwrap();
    (catalog, root)
}
fn learner() -> Connection {
    let mut c = Connection::open_in_memory().unwrap();
    storage::migrate_learner(&mut c).unwrap();
    c
}
fn files(package: &Package, paths: &[String]) -> Vec<SourceFile> {
    strings(&package.manifest["learner_source_paths"])
        .unwrap()
        .iter()
        .zip(paths)
        .map(|(path, from)| SourceFile {
            path: path.clone(),
            text: package.files[from].clone(),
        })
        .collect()
}

#[test]
fn typed_exercise_feedback_hints_and_review_survive_reopen() {
    let (catalog, root) = catalog_fixture();
    let dir = std::env::temp_dir().join(format!("clab-learning-state-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join(format!(
        "{}.sqlite",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut connection = storage::open(&db).unwrap();
    storage::migrate_learner(&mut connection).unwrap();
    let item = catalog::item(&catalog, "exercise-last").unwrap().unwrap();
    let wrong = exercise(&mut connection, &item, json!(6), 2).unwrap();
    assert_eq!(wrong["category"], "count_as_index");
    assert_eq!(wrong["correct"], false);
    assert_eq!(
        exercise(&mut connection, &item, json!("5"), 0).unwrap()["correct"],
        false
    );
    assert_eq!(
        exercise(&mut connection, &item, json!(5), 0).unwrap()["correct"],
        true
    );
    let boundary = catalog::item(&catalog, "exercise-one-past")
        .unwrap()
        .unwrap();
    assert_eq!(
        exercise(
            &mut connection,
            &boundary,
            json!({"index":6,"can_form":true,"can_dereference":true}),
            1
        )
        .unwrap()["correct"],
        false
    );
    save_progress(&connection, "hint-exercise-last", &json!(2)).unwrap();
    drop(connection);
    let connection = storage::open(&db).unwrap();
    let snapshot = state(&connection).unwrap();
    assert_eq!(snapshot["values"]["hint-exercise-last"], 2);
    assert_eq!(snapshot["values"]["review-exercise-last"]["active"], true);
    review(&connection, "exercise-last").unwrap();
    assert_eq!(
        state(&connection).unwrap()["values"]["review-exercise-last"]["active"],
        false
    );
    assert_eq!(
        pilot(&catalog, &root).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        13
    );
}

#[test]
fn reports_require_exact_keys_types_order_and_tolerance() {
    let expected = json!({"status":"ok","count":6,"sum":99,"minimum":9,"maximum":24,"mean":16.5,"mean_tolerance":0.000001});
    assert!(compare_report(
        r#"{"status":"ok","count":6,"sum":99,"minimum":9,"maximum":24,"mean":16.5000001}"#,
        &expected
    ));
    for wrong in [
        r#"{"count":6,"status":"ok","sum":99,"minimum":9,"maximum":24,"mean":16.5}"#,
        r#"{"status":"ok","count":6.0,"sum":99,"minimum":9,"maximum":24,"mean":16.5}"#,
        r#"{"status":"ok","count":6,"sum":99,"minimum":9,"maximum":24,"mean":16.51}"#,
        r#"{"status":"ok","count":6,"sum":99,"minimum":9,"maximum":24,"mean":16.5,"mean":16.5}"#,
    ] {
        assert!(!compare_report(wrong, &expected));
    }
}

#[test]
fn transaction_gate_rejects_stale_cancelled_failed_and_old_rubric() {
    let (catalog, root) = catalog_fixture();
    let package = load_package(&catalog, &root, "project-sensor").unwrap();
    let files = files(
        &package,
        &strings(&package.manifest["starter_files"]).unwrap(),
    );
    let mut connection = learner();
    save_submission(&connection, &package, &files).unwrap();
    for (i, status) in ["failed", "cancelled", "passed", "passed", "passed"]
        .iter()
        .enumerate()
    {
        let id = format!("attempt-{i}");
        put(&connection, "job-project-sensor", &json!(id)).unwrap();
        let mut result = runner::error_result(&id, status, "test", 0);
        result.submission_hash = runner::submission_hash(&files);
        result.package_id = Some(package.id.clone());
        result.content_revision = Some(package.revision);
        result.content_hash = Some(package.hash.clone());
        result.rubric_version = Some(1);
        if i == 2 {
            result.submission_hash = "old-draft".into();
        }
        if i == 3 {
            result.rubric_version = Some(0);
        }
        persist_attempt(&mut connection, &package, &files, &result, i != 4).unwrap();
        assert!(
            state(&connection).unwrap()["gates"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
    let mut result = runner::error_result("accepted", "passed", "test", 0);
    result.submission_hash = runner::submission_hash(&files);
    result.package_id = Some(package.id.clone());
    result.content_revision = Some(package.revision);
    result.content_hash = Some(package.hash.clone());
    result.rubric_version = Some(1);
    put(&connection, "job-project-sensor", &json!("accepted")).unwrap();
    persist_attempt(&mut connection, &package, &files, &result, true).unwrap();
    assert_eq!(
        state(&connection).unwrap()["gates"][0]["chapter_id"],
        "foundations-strings-input"
    );
    let mut changed = files.clone();
    changed[0].text.push_str("\n/* later draft */");
    save_submission(&connection, &package, &changed).unwrap();
    assert_eq!(
        state(&connection).unwrap()["attempts"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        state(&connection).unwrap()["gates"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // An interrupted transaction cannot leave half an attempt or an unlock.
    {
        let tx = connection.transaction().unwrap();
        tx.execute("INSERT INTO attempts SELECT 'interrupted',package_id,submission_hash,content_hash,rubric_version,result_json,files_json,created_at FROM attempts LIMIT 1",[]).unwrap();
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM attempts WHERE id='interrupted'",
                [],
                |r| r.get::<_, i32>(0)
            )
            .unwrap(),
        0
    );
}

#[cfg(windows)]
#[test]
fn real_bundled_grading_accepts_references_and_rejects_starters_and_mutants() {
    let _native_guard = runner::TEST_JOB_LOCK.lock().unwrap();
    let (catalog, root) = catalog_fixture();
    let base = std::env::temp_dir().join(format!(
        "clab-native-grading-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).unwrap();
    let runner = runner::Runner::new(base.join("runner"));
    let compiler = runner.toolchain();
    assert!(
        compiler.available,
        "Native grading tests require a local compiler: {}",
        compiler.message
    );
    for id in [
        "lab-trace-sum",
        "lab-repair-boundary",
        "lab-count-api",
        "project-sensor",
    ] {
        let package = load_package(&catalog, &root, id).unwrap();
        let mut candidates = vec![
            (
                "reference".to_owned(),
                strings(&package.manifest["solution_files"]).unwrap(),
                true,
            ),
            (
                "starter".into(),
                strings(&package.manifest["starter_files"]).unwrap(),
                false,
            ),
        ];
        for mutant in package.manifest["mutations"].as_array().unwrap() {
            let paths = if mutant["learner_sources"].is_array() {
                strings(&mutant["learner_sources"]).unwrap()
            } else {
                vec![mutant["path"].as_str().unwrap().into()]
            };
            candidates.push((mutant["id"].as_str().unwrap().into(), paths, false));
        }
        for (name, paths, expected) in candidates {
            let sources = files(&package, &paths);
            let work = base.join(id).join(&name);
            std::fs::create_dir_all(&work).unwrap();
            let result = grade(
                &name,
                &JobControl::default(),
                &work,
                &compiler,
                &package,
                &sources,
            )
            .unwrap();
            assert_eq!(
                result.status == "passed",
                expected,
                "{id}/{name}: {}\n{}",
                result.message,
                result.stderr
            );
            if expected {
                assert!(!result.checks.is_empty());
                if id == "project-sensor" && compiler.identity.to_lowercase().contains("mingw") {
                    let mut incomplete_metadata = compiler.clone();
                    incomplete_metadata.target.clear();
                    let work = base.join("mingw-identity-profile");
                    std::fs::create_dir_all(&work).unwrap();
                    let result = grade(
                        "identity-profile",
                        &JobControl::default(),
                        &work,
                        &incomplete_metadata,
                        &package,
                        &sources,
                    )
                    .unwrap();
                    assert_eq!(
                        result.status, "passed",
                        "MinGW identity must preserve the C17 printf profile: {}",
                        result.stderr
                    );
                }
            }
        }
    }
}
