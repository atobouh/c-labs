"""Check planning references and evidence completeness, not pedagogical correctness."""
import argparse
import hashlib
import json
import re
from collections import Counter
from datetime import date
from pathlib import Path


def validate(root, release=False):
    root = root.resolve()
    errors = []

    def require(condition, message):
        if not condition:
            errors.append(message)

    def read(name):
        data = json.loads((root / 'foundation' / name).read_text(encoding='utf-8'))
        require(data.get('schema_version') == 1, name + ': unsupported schema_version')
        return data

    def identities(records, label):
        ids = [r.get('id') for r in records]
        require(all(isinstance(i, str) and re.fullmatch(r'[a-z][a-z0-9-]*', i) for i in ids),
                label + ': invalid ID')
        require(len(set(ids)) == len(ids), label + ': duplicate ID')
        return set(ids)

    try:
        curriculum = read('curriculum.json')
        pilot = read('pilot.json')
        sources = read('sources.json')['sources']
        evidence = read('evidence.json')['records']
        courses = curriculum['courses']
        identities(courses, 'courses')
        require(curriculum.get('baseline') == 'c17', 'baseline must be c17')
        chapters = [chapter for course in courses for chapter in course['chapters']]
        chapter_ids = identities(chapters, 'chapters')
        for course in courses:
            ordered = [c['order'] for c in course['chapters']]
            require(ordered == list(range(1, len(ordered) + 1)),
                    course['id'] + ': chapter order must be consecutive')
        graph = {c['id']: c['prerequisite_ids'] for c in chapters}
        for chapter in chapters:
            require(bool(chapter.get('title')) and bool(chapter.get('project_title')),
                    chapter['id'] + ': missing title/project')
            require(set(chapter['prerequisite_ids']) <= chapter_ids,
                    chapter['id'] + ': unknown prerequisite')
        visiting, visited = set(), set()

        def visit(key):
            if key in visiting:
                require(False, 'curriculum: prerequisite cycle at ' + key)
                return
            if key in visited or key not in graph:
                return
            visiting.add(key)
            for dependency in graph[key]:
                visit(dependency)
            visiting.remove(key)
            visited.add(key)

        for key in graph:
            visit(key)
        require(pilot['chapter_id'] in chapter_ids, 'pilot: unknown chapter')
        require(set(pilot['prerequisite_ids']) <= chapter_ids, 'pilot: unknown prerequisite')
        pilot_chapter = next((c for c in chapters if c['id'] == pilot['chapter_id']), None)
        if pilot_chapter is not None:
            require(pilot['prerequisite_ids'] == pilot_chapter['prerequisite_ids'],
                    'pilot: prerequisites must match curriculum chapter')
        skills = pilot.get('prerequisite_skills', [])
        require(bool(skills) and all(isinstance(s, str) and s.strip() for s in skills),
                'pilot: prerequisite skills must be explicit')
        require(type(pilot['content_revision']) is int and pilot['content_revision'] > 0,
                'pilot: content_revision must be positive integer')
        objectives = identities(pilot['objectives'], 'objectives')
        require(bool(objectives), 'pilot: objectives required')
        items = pilot['items']
        item_ids = identities(items, 'items')
        counts = Counter(item['kind'] for item in items)
        require(counts == Counter(lesson=4, exercise=3, animation=2, lab=3, project=1, lexicon=1),
                'pilot: expected 4 lessons, 3 exercises, 2 animations, 3 labs, 1 project, 1 lexicon')
        paths = []
        supporting_paths = {}
        for item in items:
            ident = item['id']
            require(item['status'] in {'planned', 'draft', 'verified'}, ident + ': invalid status')
            refs = item['objective_ids']
            require(bool(refs) and set(refs) <= objectives, ident + ': invalid objective references')
            require(bool(item['checks']) and all(isinstance(c, str) and c for c in item['checks']),
                    ident + ': checks required')
            raw = item['artifact']
            path = Path(raw)
            require(not path.is_absolute() and '\\' not in raw and ':' not in raw,
                    ident + ': use portable relative artifact path')
            resolved = (root / path).resolve()
            require(resolved.is_relative_to(root.resolve()), ident + ': artifact escapes project')
            paths.append(str(resolved))
            support = item.get('supporting_artifacts', [])
            require(isinstance(support, list) and len(support) == len(set(support)),
                    ident + ': supporting artifacts must be a unique path list')
            resolved_support = []
            for support_raw in support:
                support_path = Path(support_raw)
                require(not support_path.is_absolute() and '\\' not in support_raw and ':' not in support_raw,
                        ident + ': use portable relative supporting artifact path')
                support_file = (root / support_path).resolve()
                require(support_file.is_relative_to(root.resolve()),
                        ident + ': supporting artifact escapes project')
                resolved_support.append(support_file)
                if release or item['status'] == 'verified':
                    require(support_file.is_file(), ident + ': supporting artifact missing')
            supporting_paths[ident] = resolved_support
            if release or item['status'] == 'verified':
                require(resolved.is_file(), ident + ': artifact missing')
                require(item['status'] == 'verified', ident + ': not verified')
        require(len(set(paths)) == len(paths), 'items: duplicate artifact paths')
        for objective in objectives:
            covered = {i['kind'] for i in items if objective in i['objective_ids']}
            require('lesson' in covered and 'project' in covered and
                    bool(covered & {'exercise', 'lab'}), objective + ': incomplete coverage')
        coverage = pilot.get('coverage', [])
        coverage_ids = [row.get('objective_id') for row in coverage]
        require(len(coverage_ids) == len(set(coverage_ids)) and set(coverage_ids) == objectives,
                'pilot: coverage must map every objective exactly once')
        item_by_id = {item['id']: item for item in items}
        for row in coverage:
            objective = row.get('objective_id')
            lesson = item_by_id.get(row.get('lesson_id'))
            project = item_by_id.get(row.get('project_id'))
            practices = [item_by_id.get(i) for i in row.get('practice_ids', [])]
            require(lesson is not None and lesson['kind'] == 'lesson' and objective in lesson['objective_ids'],
                    str(objective) + ': coverage lesson mismatch')
            require(project is not None and project['kind'] == 'project' and objective in project['objective_ids'],
                    str(objective) + ': coverage project mismatch')
            require(bool(practices) and all(
                item is not None and item['kind'] in {'exercise', 'animation', 'lab'}
                and objective in item['objective_ids'] for item in practices),
                str(objective) + ': coverage practice mismatch')
        identities(sources, 'sources')
        for source in sources:
            ident = source['id']
            require(source['status'] in {'unreviewed', 'reviewed'}, ident + ': invalid source status')
            require(source['redistribution'] in {'link-only', 'original-reference', 'bundle-with-notice'},
                    ident + ': invalid redistribution decision')
            require(isinstance(source.get('url'), str) and source['url'].startswith('https://'),
                    ident + ': HTTPS source URL required')
            if source['status'] == 'reviewed':
                date.fromisoformat(source['accessed_on'])
                require(bool(source.get('version')) and bool(source.get('license_evidence_url')),
                        ident + ': missing version/license evidence')
        if release:
            for needed in ['wg14-c17', 'gcc-manual']:
                require(any(s['id'] == needed and s['status'] == 'reviewed' for s in sources),
                        needed + ': pilot release requires reviewed primary reference')
        evidence_ids = [record['item_id'] for record in evidence]
        require(len(set(evidence_ids)) == len(evidence_ids), 'evidence: duplicate item record')
        require(set(evidence_ids) <= item_ids, 'evidence: unknown item')
        by_id = {record['item_id']: record for record in evidence}
        for item in items:
            if not release and item['status'] != 'verified':
                continue
            ident = item['id']
            record = by_id.get(ident)
            require(record is not None, ident + ': evidence missing')
            if record is None:
                continue
            require(record.get('reviewed') is True, ident + ': evidence needs review')
            results = record['results']
            require(bool(results), ident + ': evidence results empty')
            require(set(item['checks']) <= {r['check'] for r in results},
                    ident + ': required check evidence missing')
            for result in results:
                require(result.get('passed') is True and bool(result.get('command_or_method'))
                        and bool(result.get('observed')), ident + ': failed/incomplete check evidence')
            expected_support = {p.relative_to(root).as_posix() for p in supporting_paths.get(ident, [])}
            support_hashes = record.get('supporting_artifact_sha256', {})
            require(set(support_hashes) == expected_support,
                    ident + ': supporting artifact evidence missing or unexpected')
            for relative, digest in support_hashes.items():
                support_file = (root / relative).resolve()
                if support_file.is_relative_to(root.resolve()) and support_file.is_file():
                    actual = hashlib.sha256(support_file.read_bytes()).hexdigest()
                    require(digest == actual, ident + ': stale supporting artifact evidence')
            artifact = (root / item['artifact']).resolve()
            if artifact.is_relative_to(root.resolve()) and artifact.is_file():
                digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
                require(record.get('artifact_sha256') == digest, ident + ': stale artifact evidence')
                if item['kind'] in {'lab', 'project'}:
                    package_hashes = {
                        p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in artifact.parent.rglob('*') if p.is_file()
                    }
                    require(record.get('package_files') == package_hashes,
                            ident + ': missing/stale package evidence')
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as exc:
        errors.append('Malformed or missing foundation data: ' + str(exc))
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true', help='Require artifacts and reviewed evidence')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    errors = validate(root, args.release)
    if errors:
        for error in errors:
            print('FAIL: ' + error)
        return 1
    print('PASS: ' + ('release structure/evidence' if args.release else 'draft planning structure'))
    print('This check does not establish factual accuracy, license rights, or successful execution.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
