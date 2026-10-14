# Initial technical decisions

- Product UI: egui.
- Graphics abstraction: wgpu.
- Initial scope: procedural modeling with navigable placeholders for later workspaces.
- Compatibility goal: Houdini project import, beginning with a supported subset.
- UI requirements come from imported metadata and captured Houdini states.
- Capture tooling may use Python and Houdini APIs; the application remains Rust.
- Nuke is a separate project.

Exact egui, wgpu, docking, and node editor versions remain unpinned until the
first Rust integration build verifies that they share compatible dependencies.
Do not make the node editor library's graph the canonical project format.
Keep UI, project data, geometry evaluation, and rendering separate.

The capture harness is the first implementation. It does not implement the
Rust application or establish independent HIP parsing.
