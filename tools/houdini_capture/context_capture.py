"""Process-local context-menu probes. Never inject desktop input or move the cursor."""
import hashlib
import json

import hou
from shiboken6 import getCppPointer
from PySide6 import QtCore, QtGui, QtWidgets
from artifacts import new_output


def probe(root, step, tab=None):
    window = tab.qtParentWindow() if tab else hou.qt.mainWindow()
    if window is None:
        folder = new_output(root, 'contexts/' + step['name'])
        folder.mkdir(parents=True, exist_ok=False)
        info = {'name': step['name'], 'status': 'unavailable', 'popups': [],
                'reason': 'Houdini did not expose a Qt parent window for this pane.'}
        (folder / 'result.json').write_text(json.dumps(info, indent=2), encoding='utf-8')
        return info
    if tab:
        rect = tab.qtScreenGeometry()
        point = rect.topLeft() + QtCore.QPoint(*step['point'])
        local = window.mapFromGlobal(point)
    else:
        local = QtCore.QPoint(*step['point'])
        point = window.mapToGlobal(local)
    widget = window.childAt(local) or window
    target_point = widget.mapFromGlobal(point)
    before = {getCppPointer(w)[0] for w in QtWidgets.QApplication.topLevelWidgets() if w.isVisible()}
    folder = new_output(root, 'contexts/' + step['name'])
    folder.mkdir(parents=True, exist_ok=False)
    info = {'name': step['name'], 'point': [point.x(), point.y()],
            'target_class': widget.metaObject().className(), 'method': step.get('method', 'mouse'),
            'popups': []}

    def finish():
        candidates = []
        try:
            candidates = [w for w in QtWidgets.QApplication.topLevelWidgets()
                          if w.isVisible() and w != window and getCppPointer(w)[0] not in before]
            for i, popup in enumerate(candidates):
                path = folder / f'popup-{i:02d}.png'
                pixmap = popup.grab()
                if pixmap.isNull():
                    continue
                if not pixmap.save(str(path), 'PNG'):
                    raise RuntimeError('Popup image save failed')
                actions = [{'text': a.text(), 'enabled': a.isEnabled(),
                            'separator': a.isSeparator(), 'submenu': a.menu() is not None}
                           for a in popup.actions()]
                info['popups'].append({'file': path.name, 'class': popup.metaObject().className(),
                    'title': popup.windowTitle(), 'size': [pixmap.width(), pixmap.height()],
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'actions': actions})
            window.grab().save(str(folder / 'workspace.png'), 'PNG')
            info['status'] = 'popup-captured' if info['popups'] else 'no-popup'
        except Exception as exc:
            info['status'] = 'error'
            info['error'] = str(exc)
        finally:
            for popup in candidates:
                # Dismiss without choosing a command. Some Houdini menus use nested loops.
                try:
                    for kind in [QtCore.QEvent.Type.KeyPress, QtCore.QEvent.Type.KeyRelease]:
                        QtWidgets.QApplication.sendEvent(popup, QtGui.QKeyEvent(
                            kind, QtCore.Qt.Key.Key_Escape, QtCore.Qt.KeyboardModifier.NoModifier))
                    popup.close()
                except RuntimeError:
                    pass  # Houdini may destroy a popup while processing Escape.
            (folder / 'result.json').write_text(json.dumps(info, indent=2), encoding='utf-8')

    def hover():
        popups = [w for w in QtWidgets.QApplication.topLevelWidgets()
                  if w.isVisible() and getCppPointer(w)[0] not in before and w.width() > 50 and w.height() > 50]
        if not popups:
            return
        popup = popups[-1]
        pos = QtCore.QPoint(*step['hover'])
        child = popup.childAt(pos) or popup
        global_pos = popup.mapToGlobal(pos)
        QtWidgets.QApplication.sendEvent(child, QtGui.QMouseEvent(
            QtCore.QEvent.Type.MouseMove, QtCore.QPointF(child.mapFromGlobal(global_pos)), QtCore.QPointF(global_pos),
            QtCore.Qt.MouseButton.NoButton, QtCore.Qt.MouseButton.NoButton, QtCore.Qt.KeyboardModifier.NoModifier))
    if 'hover' in step:
        QtCore.QTimer.singleShot(300, hover)
    QtCore.QTimer.singleShot(1000 if 'hover' in step else 600, finish)
    if step.get('method') == 'context':
        event = QtGui.QContextMenuEvent(QtGui.QContextMenuEvent.Reason.Mouse, target_point, point)
        QtWidgets.QApplication.sendEvent(widget, event)
    else:
        QtWidgets.QApplication.sendEvent(widget, QtGui.QMouseEvent(
            QtCore.QEvent.Type.MouseMove, QtCore.QPointF(target_point), QtCore.QPointF(point),
            QtCore.Qt.MouseButton.NoButton, QtCore.Qt.MouseButton.NoButton, QtCore.Qt.KeyboardModifier.NoModifier))
        for event_type, buttons in [(QtCore.QEvent.Type.MouseButtonPress, QtCore.Qt.MouseButton.RightButton),
                                    (QtCore.QEvent.Type.MouseButtonRelease, QtCore.Qt.MouseButton.NoButton)]:
            event = QtGui.QMouseEvent(event_type, QtCore.QPointF(target_point), QtCore.QPointF(point),
                QtCore.Qt.MouseButton.RightButton, buttons, QtCore.Qt.KeyboardModifier.NoModifier)
            QtWidgets.QApplication.sendEvent(widget, event)
    return {'name': step['name'], 'target_class': info['target_class']}
