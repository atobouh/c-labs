"""Repeatably compile the bundled pilot lab references, starters, and known wrong solutions.

This script is a course-authoring check for fixed repository sources. It is not a sandbox
for arbitrary learner programs.
"""
import argparse
import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LAB_ROOT = ROOT / 'content' / 'foundations' / 'arrays-memory' / 'labs'
LAB_IDS = ['lab-trace-sum', 'lab-repair-boundary', 'lab-count-api']
BASE_FLAGS = ['-std=c17', '-Wall', '-Wextra', '-Wpedantic']


def invoke(command, *, timeout, label, require_success=False):
    try:
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True,
                                timeout=timeout, check=False)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f'{label} exceeded {timeout:.1f}s') from exc
    if require_success and result.returncode != 0:
        raise RuntimeError(f'{label} failed ({result.returncode}):\n{result.stderr}')
    return result


def output_size(result):
    return len(result.stdout.encode('utf-8')) + len(result.stderr.encode('utf-8'))


def parse_results(result, allowed_ids, output_limit, label):
    if output_size(result) > output_limit:
        raise RuntimeError(f'{label} exceeded the {output_limit}-byte output limit')
    if result.stderr:
        raise RuntimeError(f'{label} wrote unexpected stderr:\n{result.stderr}')
    parsed = {}
    try:
        for line in result.stdout.splitlines():
            row = json.loads(line)
            check_id = row['check_id']
            if check_id in parsed or check_id not in allowed_ids or type(row.get('passed')) is not bool:
                raise ValueError(f'invalid/duplicate check result: {row!r}')
            parsed[check_id] = row['passed']
    except (json.JSONDecodeError, KeyError, ValueError) as exc:
        raise RuntimeError(f'{label} did not emit valid test JSON Lines: {exc}') from exc
    if set(parsed) != set(allowed_ids):
        raise RuntimeError(f'{label} returned check IDs {sorted(parsed)}, expected {sorted(allowed_ids)}')
    expected_exit = 0 if all(parsed.values()) else 1
    if result.returncode != expected_exit:
        raise RuntimeError(f'{label} exit code {result.returncode} disagrees with its structured results')
    return parsed


def package_file(package, relative, label):
    path = Path(relative)
    if path.is_absolute() or '\\' in relative or ':' in relative:
        raise RuntimeError(f'{label}: path must be portable and package-relative: {relative}')
    resolved = (package / path).resolve()
    if not resolved.is_relative_to(package.resolve()) or not resolved.is_file():
        raise RuntimeError(f'{label}: missing or escaping package path: {relative}')
    return resolved


def validate_manifest(package, manifest, reviewed_source_ids):
    if manifest.get('schema_version') != 1 or manifest.get('kind') != 'lab':
        raise RuntimeError(f"{manifest.get('id')}: unsupported schema or kind")
    for field in ('brief_path', 'starter_files', 'learner_source_paths', 'solution_files',
                  'test_source_paths', 'fixture_paths'):
        values = [manifest[field]] if field == 'brief_path' else manifest[field]
        if not values:
            raise RuntimeError(f"{manifest['id']}: {field} cannot be empty")
        for relative in values:
            package_file(package, relative, manifest['id'])
    if set(manifest['starter_files']) & set(manifest['solution_files']):
        raise RuntimeError(f"{manifest['id']}: learner starter leaks the instructor solution")

    profile = manifest['profile']
    if profile.get('standard') != 'c17' or not profile.get('compiler_family') or not profile.get('compiler_version_requirement'):
        raise RuntimeError(f"{manifest['id']}: incomplete C17 compiler profile")
    if not all(type(profile.get(key)) is int and profile[key] > 0
               for key in ('compile_timeout_ms', 'run_timeout_ms', 'output_limit_bytes')):
        raise RuntimeError(f"{manifest['id']}: bounds must be positive finite integers")
    if not isinstance(profile.get('compiler_args'), list) or not all(
            isinstance(arg, str) and arg and not any(ch.isspace() for ch in arg)
            for arg in profile['compiler_args']):
        raise RuntimeError(f"{manifest['id']}: compiler arguments must be structured tokens")

    checks = manifest['checks']
    check_ids = [check['id'] for check in checks]
    if len(check_ids) != len(set(check_ids)) or 'compile-c17' not in check_ids:
        raise RuntimeError(f"{manifest['id']}: check IDs must be unique and include compile-c17")
    assertion_ids = {check['id'] for check in checks if check['kind'] == 'assertion-harness'}
    for check in checks:
        if check['kind'] not in {'compile', 'exit-code', 'stdout', 'assertion-harness', 'sanitizer'}:
            raise RuntimeError(f"{manifest['id']}: unknown check kind {check['kind']}")
        if check.get('required', True) is False and check['kind'] != 'sanitizer':
            raise RuntimeError(f"{manifest['id']}: only tool-specific checks may be optional")
    if not set(manifest.get('starter_must_fail', [])) <= assertion_ids:
        raise RuntimeError(f"{manifest['id']}: starter failure references an unknown check")
    for mutation in manifest['mutations']:
        package_file(package, mutation['path'], manifest['id'])
        if not mutation.get('must_fail') or not set(mutation['must_fail']) <= assertion_ids:
            raise RuntimeError(f"{manifest['id']}: mutant must map to existing assertion checks")
    sanitizer = manifest.get('optional_sanitizer_profile')
    if sanitizer:
        package_file(package, sanitizer['negative_test_source'], manifest['id'])
        package_file(package, sanitizer['intentional_mutant'], manifest['id'])
    for citation in manifest.get('citations', []):
        if citation.get('source_id') not in reviewed_source_ids or not citation.get('locator') or not citation.get('claim'):
            raise RuntimeError(f"{manifest['id']}: citation is missing or points to an unreviewed source")


def compile_candidate(compiler, target_args, package, candidate_paths, manifest, executable, extra_flags=()):
    profile = manifest['profile']
    test_paths = [package / item for item in manifest['test_source_paths']]
    include_dirs = [package / 'include', package / 'fixtures']
    command = [compiler, *profile['compiler_args'], *target_args, *extra_flags]
    for include_dir in include_dirs:
        command.extend(['-I', str(include_dir)])
    command.extend(str(package / item) for item in candidate_paths)
    command.extend(str(path) for path in test_paths)
    command.extend(['-o', str(executable)])
    result = invoke(command, timeout=profile['compile_timeout_ms'] / 1000,
                    label=f"{manifest['id']} compile")
    if result.returncode != 0:
        return result
    if output_size(result) > profile['output_limit_bytes'] or 'warning:' in result.stderr.lower():
        raise RuntimeError(f"{manifest['id']} compiled with warnings or exceeded the output limit:\n{result.stderr}")
    return result


def test_candidate(compiler, target_args, package, manifest, candidate_paths, temp, label):
    executable = temp / f'{label}.exe'
    compilation = compile_candidate(compiler, target_args, package, candidate_paths,
                                    manifest, executable)
    if compilation.returncode != 0:
        raise RuntimeError(f'{label} failed to compile:\n{compilation.stderr}')
    run_result = invoke([str(executable)], timeout=manifest['profile']['run_timeout_ms'] / 1000,
                        label=f'{label} run')
    assertion_ids = [check['id'] for check in manifest['checks']
                     if check['kind'] == 'assertion-harness']
    return parse_results(run_result, assertion_ids, manifest['profile']['output_limit_bytes'], label)


def validate_bad_candidate(label, results, must_fail):
    failures = {check_id for check_id, passed in results.items() if not passed}
    missing = set(must_fail) - failures
    if missing:
        raise RuntimeError(f'{label} was not rejected by required checks: {sorted(missing)}')
    print(f'PASS {label}: rejected by {", ".join(sorted(set(must_fail)))}')


def sanitizer_check(compiler, target_args, package, manifest, temp):
    profile = manifest.get('optional_sanitizer_profile')
    if not profile:
        return
    extra = profile['compiler_args']
    include_dir = package / 'include'
    reference = package / manifest['solution_files'][0]
    exact_test = package / profile['negative_test_source']
    safe_executable = temp / 'asan-reference.exe'
    command = [compiler, *manifest['profile']['compiler_args'], *target_args, *extra,
               '-I', str(include_dir), str(reference), str(exact_test), '-o', str(safe_executable)]
    probe = invoke(command, timeout=manifest['profile']['compile_timeout_ms'] / 1000,
                   label=f"{manifest['id']} AddressSanitizer profile")
    if probe.returncode != 0:
        lowered = probe.stderr.lower()
        unavailable_markers = ('cannot find -lasan', 'unrecognized command-line option',
                               'unsupported option', 'unknown argument', 'cannot find -lclang_rt')
        if any(marker in lowered for marker in unavailable_markers):
            lines = probe.stderr.strip().splitlines()
            reason = next((line.strip() for line in lines if any(marker in line.lower()
                          for marker in unavailable_markers)), 'unsupported profile')
            print(f'UNAVAILABLE {manifest["id"]} AddressSanitizer: {reason}')
            return
        raise RuntimeError(f'AddressSanitizer profile failed unexpectedly:\n{probe.stderr}')
    if 'warning:' in probe.stderr.lower():
        raise RuntimeError(f'AddressSanitizer reference compiled with a warning:\n{probe.stderr}')
    safe_run = invoke([str(safe_executable)], timeout=manifest['profile']['run_timeout_ms'] / 1000,
                      label=f"{manifest['id']} sanitized reference")
    if safe_run.returncode != 0:
        lower = safe_run.stderr.lower()
        if 'libasan' in lower or 'failed to initialize' in lower:
            print(f'UNAVAILABLE {manifest["id"]} AddressSanitizer runtime: {safe_run.stderr.strip()}')
            return
        raise RuntimeError(f'sanitized reference failed:\n{safe_run.stderr}')

    bad_executable = temp / 'asan-inclusive-end.exe'
    bad_compile = compile_candidate(compiler, target_args, package, [profile['intentional_mutant']],
                                    manifest, bad_executable, extra_flags=extra)
    if bad_compile.returncode != 0:
        raise RuntimeError(f'AddressSanitizer mutant failed to compile:\n{bad_compile.stderr}')
    bad_run = invoke([str(bad_executable)], timeout=manifest['profile']['run_timeout_ms'] / 1000,
                     label=f"{manifest['id']} sanitized boundary mutant")
    if bad_run.returncode == 0 or 'AddressSanitizer' not in bad_run.stderr:
        raise RuntimeError('AddressSanitizer did not report the exact-size boundary mutant')
    print('PASS lab-repair-boundary optional AddressSanitizer: detected the exact-size mutant')


def verify_lab(compiler, target, target_args, lab_id, temp_root, reviewed_source_ids):
    package = LAB_ROOT / lab_id.removeprefix('lab-')
    manifest_path = package / 'lab.json'
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    if manifest['id'] != lab_id:
        raise RuntimeError(f'{lab_id}: package identity mismatch')
    validate_manifest(package, manifest, reviewed_source_ids)
    with tempfile.TemporaryDirectory(prefix=f'{lab_id}-', dir=temp_root) as scratch:
        temp = Path(scratch)
        reference = test_candidate(compiler, target_args, package, manifest,
                                   manifest['solution_files'], temp, f'{lab_id}-reference')
        if not all(reference.values()):
            failed = sorted(key for key, value in reference.items() if not value)
            raise RuntimeError(f'{lab_id} reference failed checks: {failed}')
        print(f'PASS {lab_id} reference: {len(reference)} assertion checks passed')

        starter = test_candidate(compiler, target_args, package, manifest,
                                 manifest['learner_source_paths'], temp, f'{lab_id}-starter')
        validate_bad_candidate(f'{lab_id} starter', starter, manifest['starter_must_fail'])

        for mutation in manifest['mutations']:
            results = test_candidate(compiler, target_args, package, manifest,
                                     [mutation['path']], temp, f"{lab_id}-{mutation['id']}")
            validate_bad_candidate(f"{lab_id} mutant {mutation['id']}", results, mutation['must_fail'])

        sanitizer_check(compiler, target_args, package, manifest, temp)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default='gcc', help='GCC or Clang executable (default: gcc)')
    args = parser.parse_args()
    version = invoke([args.compiler, '--version'], timeout=2, label='compiler identity', require_success=True)
    target_result = invoke([args.compiler, '-dumpmachine'], timeout=2, label='compiler target')
    target = target_result.stdout.strip() if target_result.returncode == 0 else 'unknown'
    target_args = ['-D__USE_MINGW_ANSI_STDIO=1'] if 'mingw' in target.lower() else []
    print(f'Compiler: {version.stdout.splitlines()[0]}')
    print(f'Target: {target}')
    if target_args:
        print(f'Target adjustments: {target_args}')
    sources = json.loads((ROOT / 'foundation' / 'sources.json').read_text(encoding='utf-8'))
    reviewed_source_ids = {source['id'] for source in sources['sources'] if source['status'] == 'reviewed'}
    with tempfile.TemporaryDirectory(prefix='c-lab-verify-labs-') as scratch:
        for lab_id in LAB_IDS:
            verify_lab(args.compiler, target, target_args, lab_id, scratch, reviewed_source_ids)
    print('PASS all lab references and deterministic wrong-solution checks')
    print('This authoring script is not an execution sandbox for learner code.')


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, json.JSONDecodeError) as exc:
        raise SystemExit(str(exc))
