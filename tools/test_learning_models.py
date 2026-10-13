"""Contract tests for the Arrays & Memory diagram exercises and animation models."""
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / 'content' / 'foundations' / 'arrays-memory'
MISSING = object()


def read_json(relative):
    return json.loads((CONTENT / relative).read_text(encoding='utf-8'))


def resolve(answer_value, path):
    if path == 'value' and not isinstance(answer_value, dict):
        return answer_value
    if isinstance(answer_value, dict):
        return answer_value.get(path, MISSING)
    return MISSING


def classify(exercise, answer):
    if answer.get('type') != exercise['answer']['type']:
        return 'other'
    for category in exercise['error_categories']:
        condition = category['when']
        if condition.get('otherwise') is True:
            return category['id']
        def matches(rule):
            actual = resolve(answer.get('value'), rule['path'])
            if actual is MISSING:
                return False
            return actual == rule['value'] if rule['op'] == 'eq' else actual != rule['value']

        if all(matches(rule) for rule in condition['all']):
            return category['id']
    raise AssertionError('exercise has no fallback category')


def is_correct(exercise, answer):
    expected = exercise['answer']
    return answer.get('type') == expected['type'] and answer.get('value') == expected['value']


class ExerciseModelTests(unittest.TestCase):
    def setUp(self):
        self.models = {
            'offset': read_json('exercises/offset.json'),
            'last': read_json('exercises/last-element.json'),
            'one_past': read_json('exercises/one-past.json'),
        }

    def test_typed_answers_and_model_values(self):
        offset = self.models['offset']
        self.assertEqual(offset['answer']['value'], {'index': 3, 'value': 21})
        self.assertEqual(offset['model']['values'][3], 21)
        last = self.models['last']
        self.assertEqual(last['answer'], {'type': 'index', 'value': 5})
        self.assertEqual(last['model']['valid_index_range'], [0, 5])
        one_past = self.models['one_past']
        self.assertEqual(one_past['answer']['value'], {
            'index': 6, 'can_form': True, 'can_dereference': False})

    def test_grading_requires_the_complete_typed_answer(self):
        for model in self.models.values():
            self.assertTrue(is_correct(model, model['answer']))
        self.assertFalse(is_correct(self.models['offset'], {
            'type': 'array_element', 'value': {'index': 3, 'value': 18}}))
        self.assertFalse(is_correct(self.models['last'], {'type': 'array_element', 'value': 5}))

    def test_misconceptions_select_declared_feedback(self):
        cases = [
            ('offset', {'type': 'array_element', 'value': {'index': 2, 'value': 9}}, 'one_based_index'),
            ('offset', {'type': 'array_element', 'value': {'index': 3, 'value': 3}}, 'offset_as_value'),
            ('offset', {'type': 'array_element', 'value': {'index': 3, 'value': 22}}, 'wrong_element_value'),
            ('offset', {'type': 'array_element', 'value': {'index': 0, 'value': 12}}, 'other'),
            ('last', {'type': 'index', 'value': 6}, 'count_as_index'),
            ('last', {'type': 'index', 'value': 4}, 'one_too_early'),
            ('last', {'type': 'array_element', 'value': 5}, 'other'),
            ('one_past', {'type': 'one_past_classification', 'value': {
                'index': 6, 'can_form': False, 'can_dereference': False}}, 'cannot_form'),
            ('one_past', {'type': 'one_past_classification', 'value': {
                'index': 6, 'can_form': True, 'can_dereference': True}}, 'can_read_one_past'),
            ('one_past', {'type': 'one_past_classification', 'value': {
                'index': 5, 'can_form': True, 'can_dereference': False}}, 'wrong_boundary'),
        ]
        for name, response, expected in cases:
            with self.subTest(name=name, response=response):
                model = self.models[name]
                self.assertEqual(classify(model, response), expected)
                self.assertIn(expected, model['feedback']['incorrect'])

    def test_hints_are_progressive_and_citations_resolve(self):
        sources = json.loads((ROOT / 'foundation' / 'sources.json').read_text(encoding='utf-8'))
        reviewed = {s['id'] for s in sources['sources'] if s['status'] == 'reviewed'}
        for model in self.models.values():
            self.assertEqual([h['level'] for h in model['hints']], [1, 2, 3])
            self.assertTrue(all(c['source_id'] in reviewed and c['locator'] and c['claim']
                                for c in model['citations']))
            self.assertEqual(set(model['feedback']['incorrect']),
                             {category['id'] for category in model['error_categories']})
            self.assertTrue(model['error_categories'][-1]['when'].get('otherwise'))
            for category in model['error_categories'][:-1]:
                for rule in category['when']['all']:
                    self.assertIn(rule['op'], {'eq', 'neq'})
                    self.assertTrue(rule['path'])


class AnimationModelTests(unittest.TestCase):
    def setUp(self):
        self.traversal = read_json('animations/traversal.json')
        self.boundary = read_json('animations/boundary.json')
        sources = json.loads((ROOT / 'foundation' / 'sources.json').read_text(encoding='utf-8'))
        self.reviewed_sources = {source['id'] for source in sources['sources']
                                 if source['status'] == 'reviewed'}

    def assert_well_formed(self, model):
        states = model['states']
        ids = [state['id'] for state in states]
        line_count = len(model['code'].splitlines())
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(ids[0], model['initial_state_id'])
        self.assertEqual(model['reset_state_id'], model['initial_state_id'])
        self.assertEqual(set(model['controls']), {'step_forward', 'step_back', 'pause', 'reset'})
        self.assertEqual(set(model['keyboard_actions']), {'step_forward', 'step_back', 'pause', 'reset'})
        self.assertFalse(model['auto_play'])
        self.assertTrue(model['pause_behavior'])
        self.assertTrue(model['reduced_motion_summary'].strip())
        self.assertTrue(model['accessible_summary'])
        self.assertTrue(all(c['source_id'] in self.reviewed_sources and c['locator'] and c['claim']
                            for c in model['citations']))
        for state in states:
            self.assertTrue(state['caption'].strip())
            self.assertIn(state['evaluation_status'], {'ready', 'complete', 'invalid-operation'})
            self.assertIsInstance(state['selected_indices'], list)
            line = state['source_line']
            self.assertTrue(line is None or 1 <= line <= line_count)
        # The ordered state list is the model's reversible timeline; reset selects index zero.
        cursor = 0
        for _ in states[1:]:
            cursor = min(cursor + 1, len(states) - 1)
        self.assertEqual(cursor, len(states) - 1)
        for _ in states[1:]:
            cursor = max(cursor - 1, 0)
        self.assertEqual(cursor, 0)

    def test_traversal_states_trace_every_sample_once(self):
        model = self.traversal
        self.assert_well_formed(model)
        values = [12, 18, 9, 21, 15, 24]
        total = 0
        states = model['states']
        self.assertEqual(states[0]['variables']['total'], 0)
        for expected_index, state in enumerate(states[1:-1]):
            variables = state['variables']
            self.assertEqual(state['selected_indices'], [expected_index])
            self.assertEqual(variables['i'], expected_index)
            self.assertEqual(variables['selected_value'], values[expected_index])
            total += values[expected_index]
            self.assertEqual(variables['total'], total)
            self.assertIn('true', variables['condition'])
        self.assertEqual(total, 99)
        terminal = states[-1]
        self.assertEqual(terminal['evaluation_status'], 'complete')
        self.assertEqual(terminal['selected_indices'], [])
        self.assertEqual(terminal['variables']['i'], 6)
        self.assertEqual(terminal['variables']['total'], 99)
        self.assertIn('false', terminal['variables']['condition'])

    def test_boundary_model_stops_before_invalid_evaluation(self):
        model = self.boundary
        self.assert_well_formed(model)
        self.assertIn('never execute', model['execution_policy'])
        states = model['states']
        self.assertEqual([s['source_line'] for s in states], [7, 8, 9, 10])
        self.assertEqual(states[1]['selected_indices'], [2])
        self.assertEqual(states[1]['variables']['selected_value'], 8)
        self.assertEqual(states[2]['variables']['pointer_state'], 'one-past')
        self.assertEqual(states[-1]['evaluation_status'], 'invalid-operation')
        self.assertEqual(states[-1]['variables']['proposed_operation'], '*end')
        self.assertEqual(states[-1]['variables']['result'], 'not evaluated')
        self.assertNotIn('observed_value', states[-1]['variables'])
        self.assertIn('no value', states[-1]['caption'])
        self.assertEqual(states[-1], model['states'][-1])


if __name__ == '__main__':
    unittest.main()
