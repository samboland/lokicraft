# LokiCraft shell

Run from the repository root:

```powershell
& "$env:USERPROFILE/.cargo/bin/cargo.exe" run --locked
```

The shell uses eframe/egui 0.36.2 and wgpu 30.0.1. Artcraft's pinned `craft-ui`
renders the dock layout. `craft-settings` persists workspace layouts and the
selected shelf, with `craft-atomic` handling replacement underneath.

- Drag panel tabs to dock, reorder, or float them inside the window.
- Use Windows to reopen panels or reset the current layout.
- Choose a desktop beside the main menu. Each desktop keeps its own layout.
- Choose a shelf tool to create a node placeholder. Select or drag it in Network Editor.
- Edit its name and transform fields in Parameters.
- Press Tab or click Add node to search the imported tool catalog.
- Middle-drag the Scene View to orbit. Shift-middle pans; Ctrl-middle zooms.
- Scroll to zoom. Escape cancels a drag. Home resets the view.
- Choose Turntable or Trackball. View contains six axis views. Orthographic toggles projection.
- Play or scrub the timeline from frame 1 to 240.

The seven desktop presets approximate captured panel arrangements. They do not
implement the specialized editors. The perspective grid is a UI preview drawn
through egui; it is not a mesh renderer. Node records and parameter values are
session-only. There is no project save, HIP import, geometry evaluation, scripting,
rendering, or simulation yet. Unsupported commands are disabled or labeled.

Workspace settings use the `lokicraft` application ID. Use the in-memory snapshot harness for isolated runs.
`LOKICRAFT_CONFIG_DIR` can select a separate settings directory.
The harness never opens a native window or moves the desktop cursor:

```powershell
& "$env:USERPROFILE/.cargo/bin/cargo.exe" run --locked --example snapshot -- .captures/shell
& "$env:USERPROFILE/.cargo/bin/cargo.exe" test --locked --all-targets
```

Snapshots cover Build, modeling placeholders, Animate, a floating panel,
800x600, and a 2x display scale. The harness uses egui_kittest's wgpu renderer.
Its structure follows the EffectCraft reference listed in artcraft-reuse.md.

`assets/shelves.json` includes names and labels from the captured catalog.
Regenerate it with `python tools/import_shelves.py PATH/TO/shelves.json`.
No SideFX icons or executable tool scripts are included. The initial icons are
local vector drawings. The UI uses the captured Build proportions and panel hierarchy.
The Artcraft Apache-2.0 license and NOTICE are retained in `third-party/`.

Source boundaries: `model` owns shell node records; `workspace` owns layout
presets; `panels` owns panel contents; `Shell` owns UI state and settings.
A future procedural graph must remain independent of dock state and egui memory.

Blender source ports and remaining work are tracked in [blender-port/coverage.md](blender-port/coverage.md).
