"""Compile the sensor project, check its adapter/reports, and reject bundled wrong solutions.

This authoring verifier runs only the fixed sources shipped in this repository. It is not
a sandbox for arbitrary learner submissions.
"""
import argparse
import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / 'content' / 'foundations' / 'arrays-memory' / 'project'


def invoke(command, timeout, label):
    try:
        return subprocess.run(command, cwd=ROOT, text=True, capture_output=True,
                              timeout=timeout, check=False)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f'{label} exceeded {timeout:.1f}s') from exc


def size_bytes(result):
    return len(result.stdout.encode('utf-8')) + len(result.stderr.encode('utf-8'))


def package_file(relative, label):
    path = Path(relative)
    resolved = (PACKAGE / path).resolve()
    if path.is_absolute() or '\\' in relative or ':' in relative:
        raise RuntimeError(f'{label}: path must be portable and package-relative: {relative}')
    if not resolved.is_relative_to(PACKAGE.resolve()) or not resolved.is_file():
        raise RuntimeError(f'{label}: missing or escaping project path: {relative}')
    return resolved


def validate_manifest(manifest, reviewed_sources):
    if manifest.get('schema_version') != 1 or manifest.get('kind') != 'project':
        raise RuntimeError('project.json has an unsupported schema or kind')
    for field in ('brief_path', 'starter_files', 'learner_source_paths', 'solution_files',
                  'test_source_paths', 'fixture_paths', 'provided_adapter_paths'):
        values = [manifest[field]] if field == 'brief_path' else manifest[field]
        if not values:
            raise RuntimeError(f'project.json: {field} cannot be empty')
        for relative in values:
            package_file(relative, field)
    package_file(manifest['reference_only_boundary_test'], 'reference-only boundary test')
    if set(manifest['starter_files']) & set(manifest['solution_files']):
        raise RuntimeError('project starter leaks an instructor solution')

    profile = manifest['profile']
    if profile.get('standard') != 'c17' or not profile.get('compiler_family') or not profile.get('compiler_version_requirement'):
        raise RuntimeError('project compiler profile is incomplete')
    if not all(type(profile.get(key)) is int and profile[key] > 0
               for key in ('compile_timeout_ms', 'run_timeout_ms', 'output_limit_bytes')):
        raise RuntimeError('project compiler/run bounds must be positive finite integers')
    if not isinstance(profile.get('compiler_args'), list) or not all(
            isinstance(arg, str) and arg and not any(ch.isspace() for ch in arg)
            for arg in profile['compiler_args']):
        raise RuntimeError('project compiler arguments must be structured tokens')
    for adjustment in profile.get('target_adjustments', []):
        if not adjustment.get('target') or not isinstance(adjustment.get('args'), list):
            raise RuntimeError('project target adjustment is malformed')

    checks = manifest['checks']
    ids = [check['id'] for check in checks]
    if len(ids) != len(set(ids)) or 'compile-c17' not in ids:
        raise RuntimeError('project checks need unique IDs and compile-c17')
    check_ids = set(ids)
    for check in checks:
        if check['kind'] not in {'compile', 'assertion-harness', 'stdout'}:
            raise RuntimeError(f"unknown project check kind: {check['kind']}")
        if check['kind'] == 'stdout' and check.get('normalization') != 'json-object':
            raise RuntimeError(f"{check['id']}: stdout checks must declare JSON normalization")
    outputs = [check['id'] for check in checks
               if check['kind'] == 'stdout' and check.get('scope') != 'adapter']
    if not outputs or manifest['adapter_check_id'] not in check_ids:
        raise RuntimeError('project report outputs or supplied adapter check are missing')

    objective_ids = set(manifest['objective_ids'])
    for row in manifest['rubric']:
        if row['review_mode'] not in {'automated', 'self-review'}:
            raise RuntimeError(f"{row['id']}: unknown rubric review mode")
        if not set(row['objective_ids']) <= objective_ids or not set(row['check_ids']) <= check_ids:
            raise RuntimeError(f"{row['id']}: rubric has dangling objective/check references")
        if row['review_mode'] == 'automated' and not row['check_ids']:
            raise RuntimeError(f"{row['id']}: automated row must reference checks")
    rubric_objectives = {objective for row in manifest['rubric'] for objective in row['objective_ids']}
    if rubric_objectives != objective_ids:
        raise RuntimeError('rubric does not cover every project objective')

    candidate_ids = check_ids
    if not set(manifest['starter_must_fail']) <= candidate_ids:
        raise RuntimeError('starter failure list references unknown checks')
    for mutation in manifest['mutations']:
        for relative in mutation['learner_sources']:
            package_file(relative, mutation['id'])
        if not mutation.get('must_fail') or not set(mutation['must_fail']) <= candidate_ids:
            raise RuntimeError(f"{mutation['id']}: mutation has dangling required failures")
    for citation in manifest.get('citations', []):
        if citation.get('source_id') not in reviewed_sources or not citation.get('locator') or not citation.get('claim'):
            raise RuntimeError('project citation is missing or points to an unreviewed source')


def target_args_for(profile, target):
    result = []
    for adjustment in profile.get('target_adjustments', []):
        if adjustment['target'].lower() in target.lower():
            result.extend(adjustment['args'])
    return result


def compile_files(compiler, manifest, target_args, source_paths, executable, label, extra_args=()):
    profile = manifest['profile']
    command = [compiler, *profile['compiler_args'], *target_args, *extra_args,
               '-I', str(PACKAGE / 'include'), '-I', str(PACKAGE / 'fixtures')]
    command.extend(str(package_file(path, label)) for path in source_paths)
    command.extend(['-o', str(executable)])
    result = invoke(command, profile['compile_timeout_ms'] / 1000, f'{label} compile')
    if result.returncode != 0:
        raise RuntimeError(f'{label} failed to compile:\n{result.stderr}')
    if size_bytes(result) > profile['output_limit_bytes'] or 'warning:' in result.stderr.lower():
        raise RuntimeError(f'{label} compiled with warnings or exceeded the output limit:\n{result.stderr}')


def parse_events(stderr, label):
    check_results = {}
    report_ids = []
    try:
        for line in stderr.splitlines():
            event = json.loads(line)
            if set(event) == {'check_id', 'passed'}:
                if event['check_id'] in check_results or type(event['passed']) is not bool:
                    raise ValueError(f'duplicate or invalid result: {event!r}')
                check_results[event['check_id']] = event['passed']
            elif set(event) == {'report_id'}:
                report_ids.append(event['report_id'])
            else:
                raise ValueError(f'unknown test-driver event: {event!r}')
    except (json.JSONDecodeError, KeyError, ValueError) as exc:
        raise RuntimeError(f'{label} wrote invalid structured stderr: {exc}') from exc
    return check_results, report_ids


def compare_report(report, check):
    expected = check['expected']
    keys = ['status', 'count', 'sum', 'minimum', 'maximum', 'mean']
    if list(report.keys()) != keys:
        return False
    tolerance = expected.get('mean_tolerance')
    for key in keys:
        if key not in expected:
            continue
        actual = report[key]
        wanted = expected[key]
        if key == 'mean' and tolerance is not None and actual is not None:
            if type(actual) not in (int, float) or abs(actual - wanted) > tolerance:
                return False
        elif actual != wanted or type(actual) is not type(wanted):
            return False
    return True


def run_candidate(compiler, manifest, target_args, source_paths, temp, label):
    executable = temp / f'{label}.exe'
    compile_files(compiler, manifest, target_args,
                  [*source_paths, 'tests/test_driver.c'], executable, label)
    result = invoke([str(executable)], manifest['profile']['run_timeout_ms'] / 1000,
                    f'{label} run')
    if size_bytes(result) > manifest['profile']['output_limit_bytes']:
        raise RuntimeError(f'{label} exceeded the output limit')
    check_results, report_ids = parse_events(result.stderr, label)
    assertion_ids = {check['id'] for check in manifest['checks'] if check['kind'] == 'assertion-harness'}
    if set(check_results) != assertion_ids:
        raise RuntimeError(f'{label}: assertion IDs differ from project.json')
    stdout_checks = [check for check in manifest['checks']
                     if check['kind'] == 'stdout' and check.get('scope') != 'adapter']
    if report_ids != [check['id'] for check in stdout_checks]:
        raise RuntimeError(f'{label}: report IDs/order differ from stdout checks')
    try:
        reports = [json.loads(line) for line in result.stdout.splitlines()]
    except json.JSONDecodeError:
        reports = []
    output_is_well_formed = result.stdout.endswith('\n') and len(reports) == len(stdout_checks)
    for index, check in enumerate(stdout_checks):
        check_results[check['id']] = (
            output_is_well_formed
            and compare_report(reports[index], check)
        )
    expected_exit = 0 if all(check_results[check_id] for check_id in assertion_ids) else 1
    if result.returncode != expected_exit:
        raise RuntimeError(f'{label}: exit code disagrees with structured check results')
    return check_results


def run_adapter(compiler, manifest, target_args, temp):
    source_paths = [*manifest['solution_files'], *manifest['provided_adapter_paths']]
    executable = temp / 'provided-adapter.exe'
    compile_files(compiler, manifest, target_args, source_paths, executable, 'provided adapter')
    result = invoke([str(executable)], manifest['profile']['run_timeout_ms'] / 1000,
                    'provided adapter run')
    if result.returncode != 0 or result.stderr or size_bytes(result) > manifest['profile']['output_limit_bytes']:
        raise RuntimeError(f'provided adapter failed:\n{result.stderr}')
    if not result.stdout.endswith('\n'):
        raise RuntimeError('provided adapter output is missing its final newline')
    check = next(check for check in manifest['checks'] if check['id'] == manifest['adapter_check_id'])
    report = json.loads(result.stdout)
    compare_report(report, check)
    print('PASS supplied sample adapter: compiled independently and emitted the expected JSON report')


def run_null_empty_reference(compiler, manifest, target_args, temp):
    executable = temp / 'reference-null-empty.exe'
    compile_files(compiler, manifest, target_args,
                  ['solutions/sensor_analysis.c', manifest['reference_only_boundary_test']],
                  executable, 'reference null/zero boundary test')
    result = invoke([str(executable)], manifest['profile']['run_timeout_ms'] / 1000,
                    'reference null/zero boundary run')
    if result.returncode != 0 or result.stdout or result.stderr:
        raise RuntimeError('reference null/zero boundary contract failed')
    print('PASS reference-only (NULL, 0) empty contract')


def verify_project(compiler):
    manifest = json.loads((PACKAGE / 'project.json').read_text(encoding='utf-8'))
    sources = json.loads((ROOT / 'foundation' / 'sources.json').read_text(encoding='utf-8'))
    reviewed = {source['id'] for source in sources['sources'] if source['status'] == 'reviewed'}
    validate_manifest(manifest, reviewed)
    version = invoke([compiler, '--version'], 2, 'compiler identity')
    if version.returncode != 0:
        raise RuntimeError(f'compiler identity failed: {version.stderr}')
    target_result = invoke([compiler, '-dumpmachine'], 2, 'compiler target')
    target = target_result.stdout.strip() if target_result.returncode == 0 else 'unknown'
    target_args = target_args_for(manifest['profile'], target)
    print(f'Compiler: {version.stdout.splitlines()[0]}')
    print(f'Target: {target}')
    if target_args:
        print(f'Target adjustments: {target_args}')
    with tempfile.TemporaryDirectory(prefix='c-lab-verify-project-') as scratch:
        temp = Path(scratch)
        reference = run_candidate(compiler, manifest, target_args,
                                  manifest['solution_files'], temp, 'project-reference')
        if not all(reference.values()):
            raise RuntimeError('project reference failed: ' + ', '.join(k for k, v in reference.items() if not v))
        print(f'PASS project reference: {len(reference)} analysis, report, and structured-output checks passed')

        starter = run_candidate(compiler, manifest, target_args,
                                manifest['learner_source_paths'], temp, 'project-starter')
        failures = {check_id for check_id, passed in starter.items() if not passed}
        missing = set(manifest['starter_must_fail']) - failures
        if missing:
            raise RuntimeError(f'project starter was not rejected by {sorted(missing)}')
        print(f'PASS project starter: rejected by {", ".join(sorted(set(manifest["starter_must_fail"])))}')

        for mutation in manifest['mutations']:
            results = run_candidate(compiler, manifest, target_args,
                                    mutation['learner_sources'], temp, f"project-{mutation['id']}")
            failed = {check_id for check_id, passed in results.items() if not passed}
            missing = set(mutation['must_fail']) - failed
            if missing:
                raise RuntimeError(f"project mutant {mutation['id']} escaped checks {sorted(missing)}")
            print(f"PASS mutant {mutation['id']}: rejected by {', '.join(sorted(set(mutation['must_fail'])))}")

        run_adapter(compiler, manifest, target_args, temp)
        run_null_empty_reference(compiler, manifest, target_args, temp)
    print('PASS project package and wrong-solution checks')
    print('This authoring verifier is not an execution sandbox for learner code.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default='gcc', help='GCC or Clang executable (default: gcc)')
    args = parser.parse_args()
    verify_project(args.compiler)


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, json.JSONDecodeError) as exc:
        raise SystemExit(str(exc))
