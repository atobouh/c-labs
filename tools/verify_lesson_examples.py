"""Compile and check the complete C17 worked examples for the pilot lessons."""
import argparse
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / 'content' / 'foundations' / 'arrays-memory' / 'lessons' / 'examples'
CASES = [
    ('objects-side-by-side', ['objects-side-by-side.c'],
     'index | value\n0 | 12\n1 | 18\n2 | 9\n3 | 21\n4 | 15\n5 | 24\n'),
    ('index-offset', ['index-offset.c'],
     'index=0 value=12 total=12\nindex=1 value=18 total=30\n'
     'index=2 value=9 total=39\nindex=3 value=21 total=60\n'
     'index=4 value=15 total=75\nindex=5 value=24 total=99\nsum=99\n'),
    ('valid-range', ['valid-range.c'],
     'first index=0 value=12\nlast index=2 value=9\n'
     'one-past offset=3; no value is read\nempty range: no readable index\n'),
    ('carry-count', ['carry-count/main.c', 'carry-count/sum_values.c'],
     'sum=99 (count=6)\n'),
]
FLAGS = ['-std=c17', '-Wall', '-Wextra', '-Wpedantic']


def run(command, timeout, label):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True,
                            timeout=timeout, check=False)
    if result.returncode != 0:
        raise RuntimeError(f'{label} failed ({result.returncode}):\n{result.stderr}')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default='gcc', help='GCC or Clang executable (default: gcc)')
    args = parser.parse_args()
    version = run([args.compiler, '--version'], 2, 'compiler identity').stdout.splitlines()[0]
    target = run([args.compiler, '-dumpmachine'], 2, 'compiler target').stdout.strip()
    print(f'Compiler: {version}')
    flags = [*FLAGS]
    if 'mingw' in target.lower():
        # Use MinGW-w64's ISO-style stdio so C99/C17 %zu and %td formats are supported.
        flags.append('-D__USE_MINGW_ANSI_STDIO=1')
        print('Target profile: MinGW-w64 ISO-style stdio enabled')
    with tempfile.TemporaryDirectory(prefix='c-lab-lesson-examples-') as scratch:
        for name, source_names, expected in CASES:
            sources = [EXAMPLES / source for source in source_names]
            executable = Path(scratch) / (name + '.exe')
            include_dir = EXAMPLES / 'carry-count'
            command = [args.compiler, *flags, '-I', str(include_dir),
                       *(str(source) for source in sources), '-o', str(executable)]
            compile_result = run(command, 10, f'{name} compile')
            execution = run([str(executable)], 2, f'{name} run')
            if execution.stdout != expected:
                raise RuntimeError(f'{name} output mismatch:\n{execution.stdout!r}')
            if execution.stderr:
                raise RuntimeError(f'{name} wrote unexpected stderr:\n{execution.stderr}')
            if 'warning:' in compile_result.stderr.lower():
                raise RuntimeError(f'{name} compiled with a warning:\n{compile_result.stderr}')
            print(f'PASS {name}: compiled cleanly and matched its expected output')
    print('PASS all four lesson examples (temporary build files removed)')


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as exc:
        raise SystemExit(str(exc))
