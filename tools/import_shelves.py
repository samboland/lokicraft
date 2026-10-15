"""Import display metadata only. Never copy or execute Houdini shelf scripts."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('source', type=Path)
parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets')
args = parser.parse_args()
data = json.loads(args.source.read_text(encoding='utf-8'))
shelves = [{'name': shelf['name'], 'label': shelf['label'],
            'tools': [{'name': tool['name'], 'label': tool['label']} for tool in shelf['tools']]}
           for shelf in data['shelves']]
if len({s['name'] for s in shelves}) != len(shelves):
    raise ValueError('Duplicate shelf identities')
args.output.mkdir(parents=True, exist_ok=True)
(args.output / 'shelves.json').write_text(json.dumps(shelves, indent=2), encoding='utf-8')
(args.output / 'provenance.json').write_text(json.dumps({
    'source': 'Houdini Apprentice 22.0.466 shelf metadata export',
    'sha256': hashlib.sha256(args.source.read_bytes()).hexdigest(),
    'fields': ['shelf name', 'shelf label', 'tool name', 'tool label'],
    'excluded': ['icons', 'scripts', 'help content'], 'shelves': len(shelves)
}, indent=2), encoding='utf-8')
