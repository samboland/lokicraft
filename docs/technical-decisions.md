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

## Artcraft references

Artcraft apps are an additional reference for reusable UI and application patterns.
Sam identifies their license as Apache-2.0; verify each source repository's license
and notices before copying code or assets. Prefer suitable existing components
and automated imports over manual recreation.

Sam reports no Artcraft 3D editor yet. Plan for substantial original 3D editor
work while checking which shared application components can be reused.

The source audit and tested reuse choices are in [artcraft-reuse.md](artcraft-reuse.md). Prefer the pinned craft-ui, craft-settings, and craft-atomic components before creating equivalents.
