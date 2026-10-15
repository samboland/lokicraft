"""Small disposable scenes for UI reference capture, never user project edits."""
import hou


def run(step):
    mode = step['mode']
    viewer = hou.ui.paneTabOfType(hou.paneTabType.SceneViewer)
    if mode == 'setup':
        geo = hou.node('/obj').createNode('geo', 'capture_reference')
        box = geo.createNode('box', 'box1')
        transform = geo.createNode('xform', 'transform1')
        transform.setInput(0, box)
        output = geo.createNode('null', 'OUT')
        output.setInput(0, transform)
        for node, pos in [(box, (0, 3)), (transform, (0, 0)), (output, (0, -3))]:
            node.setPosition(hou.Vector2(pos))
        box.setDisplayFlag(True)
        box.setCurrent(True, clear_all_selected=True)
        viewer.setPwd(geo)
        hou.ui.paneTabOfType(hou.paneTabType.NetworkEditor).setPwd(geo)
        for parm in [transform.parm('tx'), transform.parm('ry')]:
            for frame, value in [(1, 0), (24, 2), (48, -1), (72, 0)]:
                key = hou.Keyframe()
                key.setFrame(frame)
                key.setValue(value)
                parm.setKeyframe(key)
            parm.setScope(True)
        hou.setFrame(1)
        return {'nodes': [n.path() for n in geo.children()], 'animated': ['tx', 'ry']}
    geo = hou.node('/obj/capture_reference')
    box = geo.node('box1')
    if mode == 'geometry_selection':
        viewer.setIsCurrentTab()
        viewer.setPwd(geo)
        box.setCurrent(True, clear_all_selected=True)
        box.setDisplayFlag(True)
        viewer.setSelectionMode(hou.selectionMode.Geometry)
        kind = getattr(hou.geometryType, step['kind'])
        viewer.setPickGeometryType(kind)
        viewer.setCurrentState('select')
        geometry = box.geometry()
        items = {'Points': geometry.points()[:3], 'Primitives': geometry.prims()[:2],
                 'Edges': geometry.prims()[0].edges()[:2]}[step['kind']]
        selection = hou.Selection(items)
        viewer.setCurrentGeometrySelection(kind, (box,), (selection,))
        return {'kind': step['kind'], 'selection': selection.selectionString(geometry)}
    if mode == 'nodes_selection':
        for i, node in enumerate(geo.children()):
            node.setSelected(True, clear_all_selected=i == 0)
        return {'selected': [n.path() for n in hou.selectedNodes()]}
    if mode == 'bundle':
        bundle = hou.addNodeBundle('Capture_Objects')
        bundle.addNode(geo)
        bundle.setSelected(True, clear_all_selected=True)
        return {'bundle': bundle.name(), 'nodes': [n.path() for n in bundle.nodes()]}
    if mode == 'renderer':
        parent = hou.node('/out')
        types = parent.childTypeCategory().nodeTypes()
        kind = next(n for n in ['karma', 'ifd', 'usdrender'] if n in types)
        node = parent.createNode(kind, 'capture_renderer')
        node.setCurrent(True, clear_all_selected=True)
        hou.ui.paneTabOfType(hou.paneTabType.Parm).setCurrentNode(node)
        hou.ui.paneTabOfType(hou.paneTabType.NetworkEditor).setPwd(parent)
        return {'node': node.path(), 'type': node.type().name(), 'render_started': False}
    if mode == 'tool':
        node = geo.node('tool_'+step['type']) or geo.createNode(step['type'], 'tool_'+step['type'])
        node.setInput(0, box)
        node.setDisplayFlag(True)
        node.setCurrent(True, clear_all_selected=True)
        viewer.setIsCurrentTab()
        viewer.setPwd(geo)
        state = node.type().defaultState()
        viewer.setCurrentNode(node)
        viewer.enterCurrentNodeState()
        return {'node': node.path(), 'requested_state': state, 'active_state': viewer.currentState()}
    if mode == 'animation':
        node = geo.node('transform1')
        node.setCurrent(True, clear_all_selected=True)
        editor = hou.ui.paneTabOfType(hou.paneTabType.ChannelEditor)
        editor.setIsCurrentTab()
        for name in ['tx', 'ry']:
            node.parm(name).setScope(True)
        return {'keyframes': {name: len(node.parm(name).keyframes()) for name in ['tx', 'ry']}}
    if mode == 'keyframe_selection':
        editor = hou.ui.paneTabOfType(hou.paneTabType.ChannelEditor)
        selections = []
        for name in ['tx', 'ry']:
            parm = geo.node('transform1').parm(name)
            selection = hou.ChannelGraphSelection(parm.path(), [0] * len(parm.keyframes()))
            selection.selectValues(tuple(range(len(parm.keyframes()))))
            selection.selectTimeMarks(tuple(range(len(parm.keyframes()))))
            selections.append(selection)
        editor.graph().setSelection(selections)
        selected = editor.graph().selectedKeyframes()
        if not selected:
            raise RuntimeError('No keyframes selected by ChannelGraph.setSelection')
        return {'selected_keyframes': {p.path(): len(keys) for p, keys in selected.items()}}
    if mode == 'frame_network':
        editor = hou.ui.paneTabOfType(hou.paneTabType.NetworkEditor)
        rects = [editor.itemRect(node) for node in editor.pwd().children()]
        if not rects:
            raise RuntimeError('Network has no nodes to frame')
        editor.setVisibleBounds(hou.BoundingRect(
            min(r.min()[0] for r in rects)-1, min(r.min()[1] for r in rects)-1,
            max(r.max()[0] for r in rects)+1, max(r.max()[1] for r in rects)+1))
        return {'framed_nodes': len(rects)}
    if mode == 'network':
        kind = step['kind']
        if kind == 'materials':
            parent = hou.node('/mat')
            types = parent.childTypeCategory().nodeTypes()
            type_name = next(n for n in ['principledshader::2.0', 'principledshader', 'materialbuilder'] if n in types)
            parent.createNode(type_name, 'reference_material')
        elif kind == 'simulation':
            parent = hou.node('/obj').createNode('dopnet', 'reference_simulation')
            body = parent.createNode('rbdobject', 'rigid_body')
            solver = parent.createNode('rigidbodysolver', 'solver')
            solver.setInput(0, body)
            parent.createNode('gravity', 'gravity_force')
        elif kind == 'solaris':
            parent = hou.node('/stage')
            types = parent.childTypeCategory().nodeTypes()
            for name in ['sphere', 'distantlight', 'camera']:
                if name in types:
                    parent.createNode(name, 'reference_'+name)
        elif kind == 'compositing':
            parent = hou.node('/img').createNode('img', 'reference_comp')
            parent.createNode('color', 'reference_color')
        else:
            raise ValueError(kind)
        parent.layoutChildren()
        editor = hou.ui.paneTabOfType(hou.paneTabType.NetworkEditor)
        editor.setIsCurrentTab()
        editor.setPwd(parent)
        children = parent.children()
        if children:
            children[0].setCurrent(True, clear_all_selected=True)
            positions = [node.position() for node in children]
            editor.setVisibleBounds(hou.BoundingRect(
                min(p[0] for p in positions)-2, min(p[1] for p in positions)-2,
                max(p[0] for p in positions)+3, max(p[1] for p in positions)+2))
        return {'parent': parent.path(), 'nodes': [{'path': n.path(), 'type': n.type().name()} for n in children]}
    raise ValueError(mode)
