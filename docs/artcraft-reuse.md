# Artcraft reuse assessment

Inspected on 2026-10-15. Choose shared crates before copying editor code.
The initial Rust shell now uses the selected components. See [shell.md](shell.md).
The assessment below records the pre-build audit; GPU verification is documented there.

## Decision

Use `craft-ui` as the first docking and control candidate for the egui shell.
Use `craft-settings` and `craft-atomic` for preferences and file replacement.
Keep `craft-quit` available for native close integration, especially macOS.
Pin these dependencies to the reviewed commit, not a moving branch.

| Component | Reuse decision | Boundary or missing work |
| --- | --- | --- |
| `craft-ui::docking` | Reuse layout transactions, tabs, splits, floating groups, and workspace serialization | Floating groups stay inside one viewport. Native detached windows remain app work. |
| `craft-ui` controls | Reuse icon-button behavior, tab fitting, toolbar overflow, resize handles, and widget palettes | Houdini colors, spacing, panel contents, icons, and command dispatch remain local. |
| `craft-settings` | Reuse settings storage, migration, backup, recovery, and isolated test mode | Define LokiCraft's schema and application ID. This is not a HIP document format. |
| `craft-atomic` | Reuse atomic document and settings replacement | Supply the document serializer and autosave policy. A failed-write test passed; crash durability was not tested. |
| `craft-quit` | Reuse platform close handling when needed | Save prompts and unsaved-document decisions stay in LokiCraft. macOS behavior was not run here. |
| EffectCraft snapshot example | Adapt the egui_kittest/wgpu harness and scripted intermediate captures | It depends on EffectcraftApp and its command protocol. Reuse the approach, not the whole application. |
| EffectCraft time/keyframe crates | Reference or adapt later | Uses After Effects interpolation semantics and composition-specific value types. Not Houdini channel compatibility. |
| CADCraft geometry | Evaluate later for curves, NURBS, intersections, and tessellation | Its exported kernel is predominantly drafting geometry. This does not supply a polygon mesh topology or SOP engine. |
| CADCraft constraints/undo | Reference only | Solver and history depend on CADCraft's drawing model. Importing them would also import that model. |
| PhotoCraft automation | Reference its command/transport separation and snapshot workflow | Its automation crate depends on PhotoCraft sessions and engine commands. Avoid importing the image editor. |

Do not import a complete Artcraft editor as LokiCraft's foundation. The inspected
shared library already separates reusable UI from editor-specific engines.
Keep the procedural graph, mesh attributes/topology, evaluation/cache system,
3D viewport renderer, picking/manipulators, and HIP import behind LokiCraft APIs.
The inspected sources do not provide a Houdini-compatible implementation of those systems.

## Source pins and evidence

| Repository | Reviewed commit | Evidence |
| --- | --- | --- |
| craft-libs | `8f5a2831614e632f91ad367f7af2a6e5b61b9529` | [Workspace](https://github.com/storytold/craft-libs/blob/8f5a2831614e632f91ad367f7af2a6e5b61b9529/Cargo.toml), [docking model](https://github.com/storytold/craft-libs/blob/8f5a2831614e632f91ad367f7af2a6e5b61b9529/crates/ui/src/docking/model.rs), [docking boundaries](https://github.com/storytold/craft-libs/blob/8f5a2831614e632f91ad367f7af2a6e5b61b9529/docs/ui-docking.md) |
| EffectCraft | `2c08f675c22fc9fbf9a300a60deadee9944ce3c4` | [Snapshot harness](https://github.com/storytold/effectcraft/blob/2c08f675c22fc9fbf9a300a60deadee9944ce3c4/crates/ui-egui/examples/snapshot.rs), [keyframes](https://github.com/storytold/effectcraft/blob/2c08f675c22fc9fbf9a300a60deadee9944ce3c4/crates/keyframe/src/lib.rs) |
| CADCraft | `fc4e98315a8d771b5ab832f0ade0b5e7b106df7f` | [Geometry](https://github.com/storytold/cadcraft/blob/fc4e98315a8d771b5ab832f0ade0b5e7b106df7f/crates/geom/src/lib.rs), [constraints](https://github.com/storytold/cadcraft/blob/fc4e98315a8d771b5ab832f0ade0b5e7b106df7f/crates/constraints/src/lib.rs) |
| PhotoCraft | `17c85e41751f63027345c323b19058c56640da83` | [Automation architecture](https://github.com/storytold/photocraft/blob/17c85e41751f63027345c323b19058c56640da83/book/src/architecture/automation.md), [snapshot](https://github.com/storytold/photocraft/blob/17c85e41751f63027345c323b19058c56640da83/crates/ui-egui/examples/snapshot.rs) |

All four reviewed workspace manifests declare `MIT OR Apache-2.0`.
Their license files and NOTICE files were read. Select Apache-2.0 for reused code
and preserve applicable license, attribution, and change notices.
App NOTICE files exclude Artcraft branding from the code license. PhotoCraft's
icons and fonts carry separate licenses. `craft-ui` itself bundles no artwork.
CADCraft's DWG dependency is MPL-2.0; it is outside the selected shared crates.
This is a source inventory, not a complete transitive dependency audit.

## Compatibility verification

The selected craft-libs revision requires Rust 1.95 and egui 0.36.
The host has Rust/Cargo 1.98.1. The isolated compatibility check resolved egui
0.36.2 and compiled `craft-ui`, `craft-settings`, `craft-atomic`, and `craft-quit`.
Three tests passed: docking moves and serialization with invalid-move rollback,
settings in memory without profile writes, and preservation after an injected
save failure. No native window was opened.

EffectCraft's inspected manifest uses wgpu 30.0.1 with egui/eframe 0.36.
That is a source-backed candidate pairing, not a verified LokiCraft renderer.
The check did not render GPU output, exercise native multi-window docking,
test macOS quit, validate Houdini semantics, or build the complete editors.

Next implementation should use these shared components in a small shell and
verify the Houdini layout through offscreen screenshots before adding geometry.

Reproduce the bounded check from the repository root:

```powershell
& "$env:USERPROFILE/.cargo/bin/cargo.exe" test --manifest-path tools/artcraft_reuse_check/Cargo.toml --locked
```

The check pins craft-libs by Git revision and retains its resolved Cargo.lock.
It is separate from the future application workspace.
