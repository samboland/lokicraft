"""Verify a completed shelf capture and write a searchable local gallery."""
import argparse
import hashlib
import html
import json
from pathlib import Path


def build(root):
    root = Path(root).resolve()
    result = json.loads((root / 'result.json').read_text())
    if not result.get('ok'):
        raise ValueError('Capture did not complete')
    inventory = json.loads((root / 'shelves.json').read_text())
    cards = []
    count = 0
    occurrences = 0
    unique = set()
    for shelf in inventory['shelves']:
        names = [tool['name'] for tool in shelf['tools']]
        observed = [name for page in shelf['pages'] for name in page['tools']]
        if names != observed:
            raise ValueError('Tool coverage or ordering mismatch: ' + shelf['name'])
        occurrences += len(names)
        unique.update(names)
        images = []
        for page in shelf['pages']:
            path = root / page['file']
            metadata = json.loads(path.with_suffix('.png.json').read_text())
            if hashlib.sha256(path.read_bytes()).hexdigest() != metadata['sha256']:
                raise ValueError('Capture hash mismatch: ' + str(path))
            count += 1
            images.append(f'<a href="{html.escape(page["file"])}"><img loading="lazy" src="{html.escape(page["file"])}"></a>')
        label = html.escape(shelf['label'])
        tools = html.escape(', '.join(t['label'] for t in shelf['tools'])) or 'Empty shelf'
        cards.append(f'<section><h2>{label} <small>{len(names)} tools</small></h2>{"".join(images)}<p>{tools}</p></section>')
    summary = {'shelves': len(inventory['shelves']), 'pages': count,
               'tool_occurrences': occurrences, 'unique_tools': len(unique),
               'hashes_verified': count, 'order_verified': True,
               'limits': 'Temporary full-width shelves, at most 12 tools per page. Tool scripts were not invoked. Inactive-desktop border artifacts remain.'}
    (root / 'coverage.json').write_text(json.dumps(summary, indent=2))
    document = '''<!doctype html><meta charset="utf-8"><title>Houdini shelf views</title>
<style>body{background:#202124;color:#ddd;font:16px system-ui;margin:24px}input{padding:12px;width:360px}section{margin:28px 0}img{width:100%;border:1px solid #555}small,p{color:#aaa}h2{font-size:20px}a{display:block}</style>
<h1>Houdini shelf views</h1>'''
    document += f'<p>{summary["shelves"]} shelves · {count} pages · {occurrences} tool entries · {len(unique)} unique tools</p>'
    document += '<p>Original tool order. Long shelves use multiple pages. Tool scripts were not invoked. Some borders contain capture artifacts.</p><input placeholder="Filter shelves or tools" aria-label="Filter shelves or tools">'
    document += ''.join(cards)
    document += '''<script>document.querySelector('input').addEventListener('input',e=>{for(const s of document.querySelectorAll('section'))s.hidden=!s.textContent.toLowerCase().includes(e.target.value.toLowerCase())})</script>'''
    (root / 'index.html').write_text(document, encoding='utf-8')
    return summary


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('run_directory')
    print(json.dumps(build(parser.parse_args().run_directory), indent=2))
