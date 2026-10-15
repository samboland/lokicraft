"""Package verified menu images and retain unsuccessful probe evidence."""
import argparse
import hashlib
import html
import json
from pathlib import Path
import shutil


def build(runs, output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    images = output / 'images'
    images.mkdir()
    unique = {}
    probes = []
    run_results = []
    requested = 0
    target_names = set()
    for run in map(Path, runs):
        evidence = output / 'evidence' / run.name
        evidence.mkdir(parents=True)
        for name in ['scenario.json', 'result.json', 'launch.json']:
            shutil.copy2(run / name, evidence / name)
        run_results.append({'run': run.name, 'ok': json.loads((run/'result.json').read_text())['ok']})
        targets = {s['name']: s for s in json.loads((run/'scenario.json').read_text())['steps'] if s['action']=='context_menu'}
        requested += len(targets)
        target_names.update(targets)
        for path in sorted((run/'contexts').glob('*/result.json')):
            data = json.loads(path.read_text())
            data['run'] = run.name
            data['requested_hover'] = targets.get(data['name'], {}).get('hover')
            probes.append(data)
            for popup in data['popups']:
                source = path.parent / popup['file']
                digest = hashlib.sha256(source.read_bytes()).hexdigest()
                if digest != popup['sha256']:
                    raise ValueError('Hash mismatch: ' + str(source))
                if digest not in unique:
                    filename = f'images/{len(unique):03d}-{data["name"]}.png'
                    shutil.copy2(source, output / filename)
                    unique[digest] = {'file': filename, 'size': popup['size'], 'sources': []}
                unique[digest]['sources'].append(run.name + '/' + data['name'])
    summary = {'runs': run_results, 'attempted_probes': len(probes),
        'scheduled_including_retries': requested, 'interrupted_run_probe_results': requested-len(probes),
        'unique_targets': len(target_names),
        'targets_without_result': sorted(target_names - {p['name'] for p in probes}),
        'probes_with_popups': sum(p['status']=='popup-captured' for p in probes),
        'unique_images': len(unique), 'no_popup': sum(p['status']=='no-popup' for p in probes),
        'errors': sum(p['status']=='error' for p in probes),
        'unavailable': sum(p['status']=='unavailable' for p in probes),
        'hover_without_second_popup': [p['run']+'/'+p['name'] for p in probes if p['requested_hover'] and len(p['popups'])<2],
        'limits': 'Representative context coverage, not an exhaustive inventory. No menu commands were executed. Exact PNG hashes deduplicate images, not semantic menu types.'}
    (output/'coverage.json').write_text(json.dumps(summary, indent=2))
    (output/'probes.json').write_text(json.dumps(probes, indent=2))
    (output/'images.json').write_text(json.dumps(list(unique.values()), indent=2))
    cards = []
    for item in unique.values():
        title = html.escape(', '.join(s.split('/')[-1] for s in item['sources']))
        cards.append(f'<section><h2>{title}</h2><a href="{item["file"]}"><img loading="lazy" src="{item["file"]}"></a></section>')
    document = '''<!doctype html><meta charset="utf-8"><title>Houdini context menus</title>
<style>body{background:#202124;color:#ddd;font:16px system-ui;margin:24px}input{padding:12px;width:360px}main{display:flex;flex-wrap:wrap;align-items:flex-start;gap:24px}section{max-width:700px}h2{font-size:15px;max-width:500px}img{max-width:100%;height:auto}p{max-width:900px}</style><h1>Houdini context menus</h1>'''
    document += f'<p>{len(unique)} distinct captured images from {len(probes)} probes. Filter by the pane or target name.</p>'
    document += '<p>Representative coverage. No menu commands were executed. Empty areas, unavailable targets, and unopened submenus remain gaps. See coverage.json and probes.json for failures.</p><input placeholder="Filter targets" aria-label="Filter targets"><main>'
    document += ''.join(cards) + '</main><script>document.querySelector("input").addEventListener("input",e=>{for(const s of document.querySelectorAll("section"))s.hidden=!s.textContent.toLowerCase().includes(e.target.value.toLowerCase())})</script>'
    (output/'index.html').write_text(document, encoding='utf-8')
    return summary


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True)
    parser.add_argument('runs', nargs='+')
    args = parser.parse_args()
    print(json.dumps(build(args.runs, args.output), indent=2))
