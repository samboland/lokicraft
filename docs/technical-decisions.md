# Initial technical decisions

- Product UI: egui.
- Graphics abstraction: wgpu.
- Initial scope: procedural modeling with navigable placeholders for later workspaces.
- Compatibility goal: Houdini project import, beginning with a supported subset.
- UI requirements come from imported metadata and captured Houdini states.
- Capture tooling may use Python and Houdini APIs; the application remains Rust.
- Nuke is a separate project.

The shell pins egui/eframe 0.36.2 and craft-ui at the audited revision.
Cargo.lock resolves wgpu 30.0.1. The offscreen wgpu integration renders successfully.
A node editor dependency remains undecided.
Do not make the node editor library's graph the canonical project format.
Keep UI, project data, geometry evaluation, and rendering separate.

The Rust shell and capture harness are implemented. See [shell.md](shell.md).
The shell does not evaluate geometry or establish independent HIP parsing.

## Artcraft references

Artcraft apps are an additional reference for reusable UI and application patterns.
Sam identifies their license as Apache-2.0; verify each source repository's license
and notices before copying code or assets. Prefer suitable existing components
and automated imports over manual recreation.

Sam reports no Artcraft 3D editor yet. Plan for substantial original 3D editor
work while checking which shared application components can be reused.

The source audit and tested reuse choices are in [artcraft-reuse.md](artcraft-reuse.md). Prefer the pinned craft-ui, craft-settings, and craft-atomic components before creating equivalents.

## Blender source port

Sam selected a Rust port of Blender UI logic and viewport handling as the implementation
basis, beyond using it as a reference. Preserve the Houdini shell layout. Port source
algorithms with pinned provenance and explicit coverage. Use Rust adapters for egui,
wgpu and the future procedural model. Track remaining Blender behavior in
[blender-port/coverage.md](blender-port/coverage.md); do not equate a navigation subset
with a complete UI or viewport port.
