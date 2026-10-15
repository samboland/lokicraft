"""Process-local context-menu probes. Never inject desktop input or move the cursor."""
import hashlib
import json

import hou
from shiboken6 import getCppPointer
from PySide6 import QtCore, QtGui, QtWidgets, QtQuick, QtOpenGLWidgets
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
    if 'window_point' in step:
        local = QtCore.QPoint(*step['window_point'])
        point = window.mapToGlobal(local)
    elif tab:
        rect = tab.qtScreenGeometry()
        if 'node' in step or 'wire' in step:
            paths = [step['node']] if 'node' in step else step['wire']
            positions = [tab.posToScreen(tab.itemRect(hou.node(path)).center()) for path in paths]
            x = sum(p[0] for p in positions) / len(positions)
            y = sum(p[1] for p in positions) / len(positions)
            point = rect.bottomLeft() + QtCore.QPoint(round(x), -round(y))
        else:
            point = rect.topLeft() + QtCore.QPoint(*step['point'])
        local = window.mapFromGlobal(point)
    else:
        local = QtCore.QPoint(*step['point'])
        point = window.mapToGlobal(local)
    widget = window.childAt(local) or window
    target_point = widget.mapFromGlobal(point)
    before = {getCppPointer(w)[0] for w in QtWidgets.QApplication.topLevelWidgets() if w.isVisible()}
    quick_before = {getCppPointer(w)[0] for w in QtGui.QGuiApplication.allWindows()
                    if isinstance(w, QtQuick.QQuickWindow) and w.isVisible()}
    folder = new_output(root, 'contexts/' + step['name'])
    folder.mkdir(parents=True, exist_ok=False)
    info = {'name': step['name'], 'point': [point.x(), point.y()],
            'window_point': [local.x(), local.y()],
            'target_class': widget.metaObject().className(), 'method': step.get('method', 'mouse'),
            'popups': []}

    def finish():
        candidates = []
        quick_candidates = []
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
                if step.get('framebuffer_probe'):
                    surfaces = popup.findChildren(QtOpenGLWidgets.QOpenGLWidget)
                    if isinstance(popup, QtOpenGLWidgets.QOpenGLWidget):
                        surfaces.insert(0, popup)
                    for j, surface in enumerate(surfaces):
                        if not surface.isVisible():
                            continue
                        raw = surface.grabFramebuffer()
                        if raw.isNull():
                            continue
                        raw_path = folder / f'framebuffer-{i:02d}-{j:02d}.png'
                        if not raw.save(str(raw_path), 'PNG'):
                            raise RuntimeError('OpenGL framebuffer save failed')
                        info['popups'].append({'file': raw_path.name, 'class': surface.metaObject().className(),
                            'title': popup.windowTitle(), 'size': [raw.width(), raw.height()],
                            'backend': 'QOpenGLWidget.grabFramebuffer', 'actions': [],
                            'sha256': hashlib.sha256(raw_path.read_bytes()).hexdigest()})
            quick_candidates = [w for w in QtGui.QGuiApplication.allWindows()
                                if isinstance(w, QtQuick.QQuickWindow) and w.isVisible()
                                and getCppPointer(w)[0] not in quick_before]
            for popup in quick_candidates:
                path = folder / f'quick-{len(info["popups"]):02d}.png'
                picture = popup.grabWindow()
                if picture.isNull():
                    info.setdefault('capture_gaps', []).append({'title': popup.title(), 'reason': 'QQuickWindow.grabWindow returned null'})
                    continue
                if not picture.save(str(path), 'PNG'):
                    raise RuntimeError('Qt Quick image save failed')
                info['popups'].append({'file': path.name, 'class': popup.metaObject().className(),
                    'title': popup.title(), 'size': [picture.width(), picture.height()],
                    'backend': 'QQuickWindow.grabWindow',
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'actions': []})
            window.grab().save(str(folder / 'workspace.png'), 'PNG')
            info['status'] = 'error' if 'trigger_error' in info else ('popup-captured' if info['popups'] else 'no-popup')
        except Exception as exc:
            info['status'] = 'error'
            info['error'] = str(exc)
        finally:
            for popup in quick_candidates:
                try:
                    popup.close()
                except RuntimeError:
                    pass
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
    visited = set()
    def followup(item):
        popups = [w for w in QtWidgets.QApplication.topLevelWidgets()
                  if w.isVisible() and getCppPointer(w)[0] not in before and w.width() > 50 and w.height() > 50]
        fresh = [w for w in popups if getCppPointer(w)[0] not in visited]
        if not fresh:
            return
        popup = fresh[-1]
        visited.add(getCppPointer(popup)[0])
        pos = QtCore.QPoint(*item['point'])
        child = popup.childAt(pos) or popup
        global_pos = popup.mapToGlobal(pos)
        local_pos = child.mapFromGlobal(global_pos)
        QtWidgets.QApplication.sendEvent(child, QtGui.QMouseEvent(
            QtCore.QEvent.Type.MouseMove, QtCore.QPointF(local_pos), QtCore.QPointF(global_pos),
            QtCore.Qt.MouseButton.NoButton, QtCore.Qt.MouseButton.NoButton, QtCore.Qt.KeyboardModifier.NoModifier))
        if item.get('kind') == 'click':
            for kind, buttons in [(QtCore.QEvent.Type.MouseButtonPress, QtCore.Qt.MouseButton.LeftButton),
                                  (QtCore.QEvent.Type.MouseButtonRelease, QtCore.Qt.MouseButton.NoButton)]:
                QtWidgets.QApplication.sendEvent(child, QtGui.QMouseEvent(kind,
                    QtCore.QPointF(local_pos), QtCore.QPointF(global_pos), QtCore.Qt.MouseButton.LeftButton,
                    buttons, QtCore.Qt.KeyboardModifier.NoModifier))
    def scroll_quick():
        try:
            for popup in QtGui.QGuiApplication.allWindows():
                if not isinstance(popup, QtQuick.QQuickView) or not popup.isVisible():
                    continue
                if getCppPointer(popup)[0] in quick_before:
                    continue
                root_item = popup.rootObject()
                items = root_item.findChildren(QtQuick.QQuickItem)
                candidates = [item for item in items if item.isVisible()
                              and item.width() > popup.width() * 0.45
                              and isinstance(item.property('contentY'), (float, int))
                              and isinstance(item.property('contentHeight'), (float, int))]
                if not candidates:
                    continue
                item = max(candidates, key=lambda child: child.width() * child.height())
                origin = item.property('originY') or 0
                maximum = max(0, item.property('contentHeight') - item.height())
                requested = step.get('scroll_y', 0)
                actual = min(max(0, requested), maximum)
                item.setProperty('contentY', origin + actual)
                info['scroll'] = {'requested_y': requested, 'y': actual,
                                  'maximum': maximum, 'viewport_height': item.height(),
                                  'content_height': item.property('contentHeight')}
        except Exception as exc:
            info['scroll_error'] = str(exc)
    if 'hover' in step:
        QtCore.QTimer.singleShot(300, hover)
    for i, item in enumerate(step.get('followups', [])):
        QtCore.QTimer.singleShot(400 + 700*i, lambda item=item: followup(item))
    finish_delay = 1000 + 700*len(step['followups']) if 'followups' in step else (2000 if step.get('method') in ['hover', 'tooltip'] else (1000 if 'hover' in step else 600))
    finish_delay = max(finish_delay, 2300 if 'dialog' in step else 0, step.get('capture_after_ms', 0))
    QtCore.QTimer.singleShot(finish_delay, finish)
    try:
        if step.get('dialog') == 'preferences':
            from hutil.qt.prefs.mainprefs import showMainPrefsWindow
            prefs = showMainPrefsWindow(step.get('page'))
            if 'page_name' in step:
                prefs.setPageName(step['page_name'])
            QtCore.QTimer.singleShot(1200, scroll_quick)
        elif step.get('dialog') == 'color-theme':
            from hutil.qt.pluto import themeeditor
            themeeditor.showThemeEditorWindow()
            if 'theme_tab' in step:
                themeeditor.theme_editor_window.sharedState().currentTab = step['theme_tab']
            QtCore.QTimer.singleShot(1200, scroll_quick)
        elif step.get('dialog') == 'help-tooltip':
            from hutil.qt.tooltip import getView
            view = getView()
            view.setFrom(step['help_url'])
            info['help_error'] = view.sharedState().isError
            # Supply an actual isolated-screen point, without reading or moving the OS cursor.
            anchor = view.screen().availableGeometry().center()
            view.display(pos=anchor, target=QtCore.QRect(anchor, QtCore.QSize(32, 32)))
        elif step.get('method') == 'context':
            event = QtGui.QContextMenuEvent(QtGui.QContextMenuEvent.Reason.Mouse, target_point, point)
            QtWidgets.QApplication.sendEvent(widget, event)
        elif step.get('method') in ['hover', 'tooltip']:
            QtWidgets.QApplication.sendEvent(widget, QtGui.QMouseEvent(
                QtCore.QEvent.Type.MouseMove, QtCore.QPointF(target_point), QtCore.QPointF(point),
                QtCore.Qt.MouseButton.NoButton, QtCore.Qt.MouseButton.NoButton, QtCore.Qt.KeyboardModifier.NoModifier))
            if step['method'] == 'tooltip':
                QtWidgets.QApplication.sendEvent(widget, QtGui.QHelpEvent(QtCore.QEvent.Type.ToolTip, target_point, point))
        else:
            button = QtCore.Qt.MouseButton.LeftButton if step.get('button') == 'left' else QtCore.Qt.MouseButton.RightButton
            QtWidgets.QApplication.sendEvent(widget, QtGui.QMouseEvent(
                QtCore.QEvent.Type.MouseMove, QtCore.QPointF(target_point), QtCore.QPointF(point),
                QtCore.Qt.MouseButton.NoButton, QtCore.Qt.MouseButton.NoButton, QtCore.Qt.KeyboardModifier.NoModifier))
            for event_type, buttons in [(QtCore.QEvent.Type.MouseButtonPress, button),
                                        (QtCore.QEvent.Type.MouseButtonRelease, QtCore.Qt.MouseButton.NoButton)]:
                event = QtGui.QMouseEvent(event_type, QtCore.QPointF(target_point), QtCore.QPointF(point),
                    button, buttons, QtCore.Qt.KeyboardModifier.NoModifier)
                QtWidgets.QApplication.sendEvent(widget, event)
    except Exception as exc:
        info['trigger_error'] = str(exc)
    return {'name': step['name'], 'target_class': info['target_class']}
