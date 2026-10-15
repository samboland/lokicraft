import json
from pathlib import Path
import tempfile
import unittest

from artifacts import new_output, capture_outputs
from capture import read_scenario


class HarnessTests(unittest.TestCase):
    def test_capture_does_not_overwrite_parameter_export(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            export = root / 'box.json'
            export.write_text('{"parameters":[1]}')
            image, metadata = capture_outputs(root, 'box.png')
            self.assertEqual(metadata.name, 'box.png.json')
            metadata.write_text('{"screenshot":true}')
            self.assertEqual(json.loads(export.read_text()), {'parameters':[1]})

    def test_capture_rejects_existing_sidecar(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'box.png.json').write_text('existing')
            with self.assertRaises(FileExistsError):
                capture_outputs(root, 'box.png')
            self.assertFalse((root / 'box.png').exists())

    def test_output_cannot_escape_run(self):
        with tempfile.TemporaryDirectory() as directory:
            for path in ('../escape.png', '.'):
                with self.assertRaises(ValueError):
                    new_output(directory, path)

    def test_unknown_actions_and_unbounded_waits_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            scenario = Path(directory) / 'scenario.json'
            for step in ({'action':'execute'}, {'action':'capture','settle_ms':999999},
                         {'action':'context_menu','settle_ms':100},
                         {'action':'context_menu','hover':[100,30]}):
                scenario.write_text(json.dumps({'steps':[step]}))
                with self.assertRaises(ValueError):
                    read_scenario(scenario)

    def test_popup_capture_finishes_before_next_step(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'scenario.json'
            for extra, boundary in [({'method':'hover'}, 2000),
                                    ({'method':'tooltip'}, 2000),
                                    ({'followups':[{}, {}]}, 2400),
                                    ({'capture_after_ms':4500}, 4500)]:
                step = {'action':'context_menu', **extra, 'settle_ms':boundary}
                path.write_text(json.dumps({'steps':[step]}))
                with self.assertRaises(ValueError):
                    read_scenario(path)
                step['settle_ms'] += 1
                path.write_text(json.dumps({'steps':[step]}))
                self.assertEqual(read_scenario(path)['steps'][0], step)


if __name__ == '__main__':
    unittest.main()
