# Blender source port coverage

Target: port Blender's UI logic and viewport handling into Rust, while retaining
LokiCraft's Houdini-oriented layout. This is a staged source port, not a claim
that the current shell contains Blender's full UI or renderer.

Pinned source: `blender/blender` at `608ad2b3d6bbca0f5d4938048b21744b36d4ebf5`.
`sources.json` contains downloaded-file hashes. `inventory.json` lists 2,782 files
in selected editor, window-manager, draw, GPU, UI-script and keymap trees.
That inventory is a scope map, not completed coverage or an exhaustive dependency graph.

## First port: navigation

| Blender source function | Rust implementation | Coverage |
| --- | --- | --- |
| `calctrackballvec` | `trackball_vector` | Sphere/hyperbola mapping and aspect correction; logical float coordinates replace integer window coordinates. |
| `viewrotate_apply`, trackball branch | `trackball_rotation` | Angle, sensitivity, wrap, cross product and quaternion multiplication. Scene callbacks excluded. |
| `viewrotate_apply`, turntable branch | `turntable_rotation` | Horizon blend, gimbal recovery, local/global rotations and normalization. Sensitivity currently uses the shell's 0.007 radians per logical pixel. |
| `view3d_orbit_apply_dyn_ofs` | `orbit_offset` | Math ported and tested. Not connected to selection until a geometry scene exists. |
| `viewmove_apply`, unlocked branch | `View::pan` | Offset update ported; pixel-to-world conversion uses the local camera adapter. Camera/offset locks excluded. |
| `view_zoom_to_window_xy_3d` | `View::zoom_factor` | Distance and cursor-offset formulas. Host distance bounds are 0.01 to 1,000,000. |
| `view_zoom_apply_step` | `View::zoom_steps` | 1.2 distance step. Host adapter accepts fractional wheel input. |
| `viewzoom_scale_value`, vertical dolly | `dolly_factor` | Ratio formula and incremental zoom anchored at the previous cursor position. Host bounds replace the scene-dependent range. |
| `ViewOpsData` backup/restore and modal handlers | `Navigation` | Begin/update/confirm/cancel and modifier switching. Camera state, autokey, scene callbacks and cursor wrapping excluded. |

The egui adapter uses middle-drag for rotation, Shift-middle for pan, Ctrl-middle
for zoom, and Escape for cancel. Focus loss also cancels. No cursor capture,
warping, or desktop input is used. Projection and six axis views use a local
camera adapter. Keys 1/3/7/5 are available while hovering; egui does not distinguish
these keys from numpad keys here. The Houdini Y-up basis maps to Blender Z-up
as `(x, y, z) -> (x, -z, y)` at the viewport boundary.

## Verification

`tools/blender_reference/trackball.cpp` embeds the unchanged pinned Blender
`calctrackballvec` with minimal host helpers. Its compiled output supplies eight
numerical fixtures. Rust tests compare those fixtures, cursor-anchor invariance,
pan displacement, axis projection, near clipping, pole stability, wheel stepping,
dynamic pivot behavior and modal cancel through egui pointer events.

`examples/navigation_snapshot.rs` renders perspective, top, front and a real
trackball drag through offscreen wgpu. This does not establish visual parity
with a running Blender instance. The current grid still uses egui lines.

## Remaining port

1. Remaining navigation: axis snap, camera view/locks, smooth transitions, zoom styles,
   selection/depth pivots, fly/walk, NDOF, touchpad conventions and full keymap behavior.
2. Selection and manipulators: object/component picking, box/lasso, gizmos,
   transform operators, snapping, modal feedback and undo integration.
3. Draw infrastructure: scene synchronization, GPU resources, depth/ID buffers,
   overlays, solid shading, clipping, color management and wgpu shader translation.
4. General UI logic: region lifecycle, event dispatch, operator polling,
   property/RNA adapters, widget behaviors, menus, popovers, shortcuts and preferences.
5. Editor-specific behavior: outliner, properties, node editor, timeline and tools.
   These depend on a Rust scene model; egui placeholders do not count as ports.

Retain function-level provenance and test evidence as coverage expands. Do not
import Blender's C++ scene model into the canonical Rust project format.
