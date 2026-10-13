"""Mutation checks for the structural validator; all data lives in temporary fixtures."""
import json
import hashlib
import shutil
import tempfile
import unittest
from pathlib import Path

from check_foundation import validate


class FoundationChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        shutil.copytree(Path(__file__).resolve().parents[1] / 'foundation', self.root / 'foundation')
        self.change('pilot.json', lambda d: [item.update(status='planned') for item in d['items']])
        self.change('evidence.json', lambda d: d.update(records=[]))

    def tearDown(self):
        self.temp.cleanup()

    def change(self, name, mutate):
        path = self.root / 'foundation' / name
        data = json.loads(path.read_text(encoding='utf-8'))
        mutate(data)
        path.write_text(json.dumps(data), encoding='utf-8')

    def test_draft_baseline(self):
        self.assertEqual(validate(self.root), [])

    def test_planned_content_cannot_release(self):
        self.assertTrue(any('not verified' in e for e in validate(self.root, True)))

    def test_duplicate_identity(self):
        self.change('pilot.json', lambda d: d['items'][1].update(id=d['items'][0]['id']))
        self.assertTrue(any('duplicate ID' in e for e in validate(self.root)))

    def test_unknown_objective(self):
        self.change('pilot.json', lambda d: d['items'][0].update(objective_ids=['invented']))
        self.assertTrue(any('objective references' in e for e in validate(self.root)))

    def test_escaping_artifact(self):
        self.change('pilot.json', lambda d: d['items'][0].update(artifact='../outside.md'))
        self.assertTrue(any('escapes project' in e for e in validate(self.root)))

    def test_prerequisite_cycle(self):
        self.change('curriculum.json', lambda d: d['courses'][0]['chapters'][0].update(
            prerequisite_ids=[d['courses'][0]['chapters'][1]['id']]))
        self.assertTrue(any('prerequisite cycle' in e for e in validate(self.root)))

    def test_missing_objective_coverage(self):
        self.change('pilot.json', lambda d: d['objectives'].append(
            {'id': 'obj-untaught', 'description': 'Deliberate fixture gap'}))
        self.assertTrue(any('incomplete coverage' in e for e in validate(self.root)))

    def test_coverage_map_rejects_wrong_practice(self):
        self.change('pilot.json', lambda d: d['coverage'][0].update(practice_ids=['lab-count-api']))
        self.assertTrue(any('coverage practice mismatch' in e for e in validate(self.root)))

    def test_pilot_prerequisites_match_curriculum(self):
        self.change('pilot.json', lambda d: d.update(prerequisite_ids=['foundations-build-run']))
        self.assertTrue(any('prerequisites must match' in e for e in validate(self.root)))

    def test_fake_verified_status_rejected(self):
        self.change('pilot.json', lambda d: d['items'][0].update(status='verified'))
        errors = validate(self.root)
        self.assertTrue(any('artifact missing' in e for e in errors))
        self.assertTrue(any('evidence missing' in e for e in errors))

    def test_stale_evidence(self):
        artifact = self.root / 'content' / 'fixture.md'
        artifact.parent.mkdir()
        artifact.write_text('Fixture only; not course content.', encoding='utf-8')
        self.change('pilot.json', lambda d: d['items'][0].update(
            status='verified', artifact='content/fixture.md'))
        self.change('evidence.json', lambda d: d['records'].append({
            'item_id': 'lesson-objects-side-by-side', 'artifact_sha256': '0' * 64,
            'reviewed': True, 'results': [{'check': check, 'command_or_method': 'test fixture',
                                         'observed': 'fixture only', 'passed': True}
                                        for check in ['citation-review', 'teaching-review',
                                                      'example-verification']]}))
        self.assertTrue(any('stale artifact evidence' in e for e in validate(self.root)))

    def test_package_change_invalidates_evidence(self):
        package = self.root / 'content' / 'lab-fixture'
        package.mkdir(parents=True)
        manifest = package / 'lab.json'
        manifest.write_text('{}', encoding='utf-8')
        source = package / 'solution.c'
        source.write_text('/* structural test fixture */', encoding='utf-8')
        hashes = {p.relative_to(self.root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in package.iterdir()}
        self.change('pilot.json', lambda d: next(i for i in d['items'] if i['id'] == 'lab-trace-sum').update(
            status='verified', artifact='content/lab-fixture/lab.json'))
        self.change('evidence.json', lambda d: d['records'].append({
            'item_id': 'lab-trace-sum', 'artifact_sha256': hashes['content/lab-fixture/lab.json'],
            'package_files': hashes, 'reviewed': True,
            'results': [{'check': c, 'command_or_method': 'structural fixture',
                         'observed': 'not actual execution evidence', 'passed': True}
                        for c in ['compile-run-fixtures', 'wrong-solution-tests', 'package-review']]}))
        self.assertEqual(validate(self.root), [])
        source.write_text('/* changed fixture */', encoding='utf-8')
        self.assertTrue(any('stale package evidence' in e for e in validate(self.root)))

    def test_lesson_supporting_artifact_change_invalidates_evidence(self):
        body = self.root / 'content' / 'lesson.md'
        metadata = self.root / 'content' / 'lesson.json'
        body.parent.mkdir()
        body.write_text('Lesson body fixture.', encoding='utf-8')
        metadata.write_text('{"schema_version":1}', encoding='utf-8')
        hashes = {path.relative_to(self.root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                  for path in [metadata]}
        self.change('pilot.json', lambda d: next(i for i in d['items']
            if i['id'] == 'lesson-objects-side-by-side').update(
                status='verified', artifact='content/lesson.md', supporting_artifacts=['content/lesson.json']))
        self.change('evidence.json', lambda d: d['records'].append({
            'item_id': 'lesson-objects-side-by-side',
            'artifact_sha256': hashlib.sha256(body.read_bytes()).hexdigest(),
            'supporting_artifact_sha256': hashes, 'reviewed': True,
            'results': [{'check': c, 'command_or_method': 'fixture', 'observed': 'fixture', 'passed': True}
                        for c in ['citation-review', 'teaching-review', 'example-verification']]}))
        self.assertEqual(validate(self.root), [])
        metadata.write_text('{"schema_version":2}', encoding='utf-8')
        self.assertTrue(any('stale supporting artifact evidence' in e for e in validate(self.root)))


if __name__ == '__main__':
    unittest.main()
