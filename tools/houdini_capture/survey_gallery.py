"""Build a stage-by-stage UI capture index without hiding failures."""
import argparse
import hashlib
import html
import json
from pathlib import Path


def build(root):
    root = Path(root)
    stages = []
    sections = []
    quality_file = root / 'review.json'
    quality = json.loads(quality_file.read_text()) if quality_file.exists() else {}
    for run in sorted(p for p in root.iterdir() if p.is_dir() and (p/'result.json').exists()):
        result = json.loads((run/'result.json').read_text())
        stage = {'name': run.name, 'run_completed': result['ok'], 'run_error': result.get('error'),
                 'probes': [], 'state_errors': [], 'screenshots': []}
        cards = []
        for path in sorted((run/'states').glob('*.json')):
            data = json.loads(path.read_text())
            if not data.get('ok'):
                stage['state_errors'].append({'name': path.stem, 'error': data.get('error')})
        for path in sorted(run.glob('*.png')):
            sidecar = path.with_suffix('.png.json')
            if sidecar.exists():
                meta = json.loads(sidecar.read_text())
                if hashlib.sha256(path.read_bytes()).hexdigest() != meta['sha256']:
                    raise ValueError('Hash mismatch: ' + str(path))
            rel = path.relative_to(root).as_posix()
            stage['screenshots'].append(rel)
            cards.append(f'<article><h3>{html.escape(path.stem)}</h3><a href="{rel}"><img loading="lazy" src="{rel}"></a></article>')
        for path in sorted((run/'contexts').glob('*/result.json')):
            data = json.loads(path.read_text())
            stage['probes'].append({'name': data['name'], 'status': data['status'], 'images': len(data['popups']),
                                   'help_error': data.get('help_error', False), 'scroll': data.get('scroll'),
                                   'capture_gaps': data.get('capture_gaps', []),
                                   'error': data.get('trigger_error') or data.get('error') or data.get('scroll_error')})
            images = []
            for popup in data['popups']:
                picture = path.parent/popup['file']
                if hashlib.sha256(picture.read_bytes()).hexdigest() != popup['sha256']:
                    raise ValueError('Hash mismatch: ' + str(picture))
                rel = picture.relative_to(root).as_posix()
                backend = html.escape(popup.get('backend', 'QWidget.grab'))
                images.append(f'<figure><a href="{rel}"><img loading="lazy" src="{rel}"></a><figcaption>{backend}</figcaption></figure>')
            if images:
                cards.append(f'<article><h3>{html.escape(data["name"])}</h3><div class="images">{"".join(images)}</div></article>')
        stage['review'] = quality.get(run.name, 'Not reviewed')
        scenario_path = run/'scenario.json'
        if scenario_path.exists():
            recorded = {probe['name'] for probe in stage['probes']}
            for step in json.loads(scenario_path.read_text())['steps']:
                if step['action'] == 'context_menu' and step['name'] not in recorded:
                    stage['probes'].append({'name': step['name'], 'status': 'missing-result', 'images': 0})
        stages.append(stage)
        empty = [p['name'] for p in stage['probes'] if p['status'] != 'popup-captured']
        failure = f'<details><summary>Run stopped with an error</summary><pre>{html.escape(str(stage["run_error"]))}</pre></details>' if not stage['run_completed'] else ''
        sections.append(f'<section><h2>{html.escape(run.name)}</h2><p>{html.escape(stage["review"])}</p>{failure}<p>Probe gaps: {html.escape(", ".join(empty)) or "none"}. Scene setup errors: {len(stage["state_errors"])}.</p><div class="cards">{"".join(cards)}</div></section>')
    summary = {'stages': stages, 'popup_images': sum(p['images'] for s in stages for p in s['probes']),
               'reference_images': sum(len(s['screenshots']) for s in stages),
               'probes': sum(len(s['probes']) for s in stages),
               'no_popup': sum(p['status']=='no-popup' for s in stages for p in s['probes']),
               'missing_results': sum(p['status']=='missing-result' for s in stages for p in s['probes']),
               'state_errors': sum(len(s['state_errors']) for s in stages)}
    (root/'coverage.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
    document = '''<!doctype html><meta charset="utf-8"><title>Houdini UI capture survey</title><style>
body{background:#202124;color:#ddd;font:16px system-ui;margin:24px}input{padding:12px;width:380px}.cards{display:flex;flex-wrap:wrap;gap:20px;align-items:flex-start}article{max-width:100%;border:1px solid #444;padding:12px}h3{font-size:15px}.images{display:flex;flex-wrap:wrap;gap:8px}img{max-width:640px;max-height:600px;object-fit:contain;object-position:top left}section{margin:40px 0}p{max-width:1000px}</style><h1>Houdini UI capture survey</h1><p>Sequential reference captures. Open an image for full resolution. Review notes distinguish usable references from rendering failures. Counts are images, not unique UI features.</p><input aria-label="Filter captures" placeholder="Filter captures">'''
    document += ''.join(sections)
    document += '<script>document.querySelector("input").addEventListener("input",e=>{for(const a of document.querySelectorAll("article"))a.hidden=!a.textContent.toLowerCase().includes(e.target.value.toLowerCase())})</script>'
    (root/'index.html').write_text(document,encoding='utf-8')
    return {k:v for k,v in summary.items() if k != 'stages'}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root')
    print(json.dumps(build(parser.parse_args().root),indent=2))
