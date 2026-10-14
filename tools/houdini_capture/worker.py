"""Executed by Houdini after waitforui. All HOM/Qt calls stay on its UI thread."""
import hashlib
import json
import os
import sys
from pathlib import Path
import time
import traceback

import hou
from PySide6 import QtCore, QtGui, QtWidgets

sys.path.insert(0, os.environ['LOKI_CAPTURE_WORKER_DIR'])
from artifacts import new_output, capture_outputs

ROOT = Path(os.environ['LOKI_CAPTURE_ROOT']).resolve()
STEPS = json.loads((ROOT / 'scenario.json').read_text(encoding='utf-8'))['steps']
RESULTS = []
INDEX = 0


def write_json(path, data):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, indent=2), encoding='utf-8')
    temporary.replace(path)


def output_path(name):
    return new_output(ROOT, name)


def rectangle(rect):
    return [rect.x(), rect.y(), rect.width(), rect.height()]


def pane(selector):
    tabs = hou.ui.curDesktop().paneTabs()
    matches = [t for t in tabs if t.name() == selector]
    if not matches:
        matches = [t for t in tabs if str(t.type()).split('.')[-1] == selector]
    if len(matches) != 1:
        raise ValueError(f'Pane {selector!r} matches {len(matches)} tabs; use an exact name from inspect')
    return matches[0]


def inventory():
    window = hou.qt.mainWindow()
    return {'version':hou.applicationVersionString(), 'license':str(hou.licenseCategory()),
        'desktop':hou.ui.curDesktop().name(), 'desktops':[d.name() for d in hou.ui.desktops()],
        'window_size':[window.width(),window.height()], 'device_pixel_ratio':window.devicePixelRatioF(),
        'window_origin': [window.mapToGlobal(QtCore.QPoint()).x(), window.mapToGlobal(QtCore.QPoint()).y()],
        'panes':[{'name':t.name(), 'type':str(t.type()).split('.')[-1],
                  'current':t.isCurrentTab(), 'screen_rect':rectangle(t.qtScreenGeometry())}
                 for t in hou.ui.curDesktop().paneTabs()]}


def capture(step):
    target = pane(step['pane']) if 'pane' in step else None
    if target and not target.isCurrentTab():
        raise ValueError('Activate the pane with a pane action before capturing it')
    window = target.qtParentWindow() if target else hou.qt.mainWindow()
    pixmap = window.grab()
    if pixmap.isNull():
        raise RuntimeError('Houdini returned an empty capture')
    region = QtCore.QRect(0, 0, window.width(), window.height())
    if target:
        screen_rect = target.qtScreenGeometry()
        region = QtCore.QRect(window.mapFromGlobal(screen_rect.topLeft()), screen_rect.size())
    if 'rect' in step:
        requested = QtCore.QRect(*step['rect'])
        if not QtCore.QRect(0, 0, region.width(), region.height()).contains(requested):
            raise ValueError('Requested crop lies outside the selected region')
        region = QtCore.QRect(region.topLeft() + requested.topLeft(), requested.size())
    if not QtCore.QRect(0, 0, window.width(), window.height()).contains(region):
        raise ValueError(f'Pane geometry is outside its window: {rectangle(region)}')
    ratio = pixmap.devicePixelRatioF()
    pixels = QtCore.QRect(round(region.x()*ratio), round(region.y()*ratio),
                         round(region.width()*ratio), round(region.height()*ratio))
    image = pixmap.copy(pixels)
    path, sidecar = capture_outputs(ROOT, step['file'])
    if not image.save(str(path), 'PNG'):
        raise RuntimeError('PNG save failed')
    metadata = {'file':str(path), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                'logical_rect':rectangle(region), 'pixel_size':[image.width(),image.height()],
                'device_pixel_ratio':ratio, 'backend':'QWidget.grab', 'state':inventory()}
    write_json(sidecar, metadata)
    return metadata


def execute(step):
    action = step['action']
    if action == 'capture_shelves':
        shelves = sorted(hou.shelves.shelves().values(), key=lambda s: s.name())
        data = []
        expanded = []
        for index, shelf in enumerate(shelves):
            tools = list(shelf.tools())
            pages = [tools[i:i+12] for i in range(0, len(tools), 12)] or [[]]
            item = {'name': shelf.name(), 'label': shelf.label(), 'pages': [],
                    'tools': [{'name': t.name(), 'label': t.label(), 'icon': t.icon()} for t in tools]}
            for page_index, page in enumerate(pages):
                name = f'shelves/{index:03d}-{shelf.name()}-{page_index+1:02d}.png'
                item['pages'].append({'file': name, 'tools': [t.name() for t in page]})
                expanded.extend([
                    {'action': 'show_shelf', 'label': shelf.label(), 'tools': [t.name() for t in page], 'settle_ms': 700},
                    {'action': 'capture', 'file': name, 'rect': [0, 0, hou.qt.mainWindow().width(), 180], 'settle_ms': 100}])
            data.append(item)
        write_json(output_path('shelves.json'), {'shelves': data, 'page_size': 12,
            'presentation': 'Original tools in original order, in temporary full-width shelves; no tool scripts invoked.'})
        STEPS[INDEX+1:INDEX+1] = expanded
        return {'shelves': len(data), 'pages': len(expanded)//2}
    if action == 'show_shelf':
        # All definitions are owned by this run; installed shelf definitions stay untouched.
        owned = str(ROOT / 'capture-shelves.shelf')
        shelf = hou.shelves.shelves().get('loki_capture_page')
        if shelf is None:
            shelf = hou.shelves.newShelf(file_path=owned, name='loki_capture_page', label=step['label'])
        shelf.setLabel(step['label'])
        shelf.setTools([hou.shelves.tools()[name] for name in step['tools']])
        shelf_set = hou.shelves.shelfSets().get('loki_capture_set')
        if shelf_set is None:
            shelf_set = hou.shelves.newShelfSet(file_path=owned, name='loki_capture_set', label='Capture')
        shelf_set.setShelves([shelf])
        for current in hou.ui.curDesktop().shelfDock().shelfSets():
            hou.hscript('shelfdock remove ' + current.name())
        result, error = hou.hscript('shelfdock -o 1 -l icon_and_text add loki_capture_set')
        if error:
            raise RuntimeError(error)
        hou.ui.triggerUpdate()
        return {'label': step['label'], 'tools': step['tools']}
    if action == 'inspect':
        data = inventory()
        write_json(output_path(step.get('file','inventory.json')), data)
        return data
    if action == 'capture':
        return capture(step)
    if action == 'desktop':
        matches = [d for d in hou.ui.desktops() if d.name() == step['name']]
        if len(matches) != 1:
            raise ValueError('Unknown desktop')
        matches[0].setAsCurrent()
    elif action == 'pane':
        tab = pane(step['pane'])
        if 'type' in step:
            tab = tab.setType(getattr(hou.paneTabType, step['type']))
        tab.setIsCurrentTab()
        if 'path' in step:
            node = hou.node(step['path'])
            if node is None:
                raise ValueError('Unknown node path')
            if tab.type() == hou.paneTabType.Parm:
                tab.setCurrentNode(node)
            else:
                tab.setPwd(node)
        return {'pane':tab.name(), 'type':str(tab.type())}
    elif action == 'create_node':
        parent = hou.node(step['parent'])
        if parent is None or parent.node(step['name']) is not None:
            raise ValueError('Parent missing or node name already exists')
        node = parent.createNode(step['type'], step['name'])
        for index, path in enumerate(step.get('inputs', [])):
            source = hou.node(path)
            if source is None:
                raise ValueError(f'Unknown input {path}')
            node.setInput(index, source)
        parent.layoutChildren()
        if step.get('display'):
            node.setDisplayFlag(True)
        return {'path':node.path(), 'type':node.type().name()}
    elif action == 'select_node':
        node = hou.node(step['path'])
        if node is None:
            raise ValueError('Unknown node path')
        node.setCurrent(True, clear_all_selected=True)
        for tab in hou.ui.curDesktop().paneTabs():
            if tab.type() == hou.paneTabType.Parm:
                tab.setCurrentNode(node)
    elif action == 'set_parameters':
        node = hou.node(step['path'])
        if node is None:
            raise ValueError('Unknown node path')
        node.setParms(step['values'])
    elif action == 'load_project':
        hou.hipFile.load(str(Path(step['path']).resolve(strict=True)), suppress_save_prompt=True)
    elif action == 'export_parameters':
        node = hou.node(step['path'])
        if node is None:
            raise ValueError('Unknown node path')
        data = []
        for parm in node.parmTuples():
            template = parm.parmTemplate()
            item = {'name':parm.name(), 'label':template.label(), 'type':str(template.type()),
                    'components':template.numComponents(), 'hidden':template.isHidden()}
            try:
                item['value'] = list(parm.eval())
                json.dumps(item['value'])
            except Exception:
                item.pop('value', None)
                item['value_unavailable'] = True
            data.append(item)
        write_json(output_path(step['file']), {'node':node.path(), 'parameters':data})
        return {'parameters':len(data)}
    else:
        raise ValueError(f'Unknown action {action}')
    hou.ui.triggerUpdate()
    return {'action':action}


def tick():
    global INDEX
    if INDEX == len(STEPS):
        write_json(ROOT / 'result.json', {'ok':True, 'steps':RESULTS, 'version':hou.applicationVersionString()})
        return
    step = STEPS[INDEX]
    write_json(ROOT / 'progress.json', {'index':INDEX, 'step':step, 'time':time.time()})
    try:
        result = execute(step)
        RESULTS.append({'index':INDEX, 'action':step['action'], 'result':result})
        INDEX += 1
        QtCore.QTimer.singleShot(step.get('settle_ms', 1000), tick)
    except Exception:
        write_json(ROOT / 'result.json', {'ok':False, 'failed_index':INDEX,
            'step':step, 'error':traceback.format_exc(), 'completed':RESULTS})


write_json(ROOT / 'startup.json', {'version':hou.applicationVersionString(), 'pid':os.getpid()})
QtCore.QTimer.singleShot(7000, tick)
