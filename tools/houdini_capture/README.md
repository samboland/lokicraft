# Houdini UI capture harness

Runs Houdini on a separate Windows desktop that is never shown or switched to.
The controller uses Python's standard library. The worker uses Houdini's own
Python, HOM, and PySide6. No pip packages or background network service are needed.

This is unattended GUI capture, not Houdini's non-graphical `hython` mode.
A licensed Houdini installation and a logged-in Windows session are required.
The initial launcher explicitly selects Apprentice mode.

## Run

From the repository root, in PowerShell:

```powershell
python tools/houdini_capture/capture.py --houdini 'C:\Program Files\Side Effects Software\Houdini 22.0.466\bin\houdinifx.exe' --scenario tools/houdini_capture/scenarios/modeling.json --output .captures/modeling-001
```

Choose a new output directory for every run. Existing directories are rejected.
The default size is 1600x1000, with a 180-second timeout.
Use `--width`, `--height`, and `--timeout` to change those values.
The captured client size can differ from the requested startup geometry.
Every capture records the actual size and device pixel ratio.

The launcher creates a new Houdini process and a private preference directory.
It never attaches to the user's Houdini session. It never calls SwitchDesktop,
SendInput, SetCursorPos, or SetForegroundWindow. The launcher closes its own
Houdini process after completion, failure, timeout, or interruption.
Recordings include foreground window handles before and after the run.
These observations alone do not prove that every future run preserves focus.

## Explore

Copy the scenario and edit its `steps` array. Actions run on Houdini's UI thread.
Each action yields to the event loop before the next action starts. The default
settling delay is 1000 ms. Set `settle_ms` on an action for slower UI changes.

| Action | Fields | Effect |
| --- | --- | --- |
| `inspect` | optional `file` | Export desktops, pane names, types, bounds, and selected tabs |
| `desktop` | `name` | Switch to an installed Houdini desktop |
| `pane` | `pane`, optional `type`, `path` | Select a tab, change its type, or navigate to a node |
| `create_node` | `parent`, `type`, `name`, optional `inputs`, `display` | Create a reference node in the disposable session |
| `select_node` | `path` | Select a node and show its parameters |
| `set_parameters` | `path`, `values` | Set parameters by name |
| `export_parameters` | `path`, `file` | Export parameter names, labels, types, and available values |
| `load_project` | absolute `path` | Load a project into the disposable session |
| `capture` | `file`, optional `pane`, `rect` | Capture the whole window, a pane, or a region |

Pane selectors accept exact names from `inspect`, or a unique type such as
`Parm`, `NetworkEditor`, or `DetailsView`. Ambiguous selectors fail.
Activate a pane before capturing it. Crops use `[x, y, width, height]` in logical
pixels relative to the selected pane, or the whole window when no pane is given.
Invalid crops fail rather than silently clipping.

```json
{"action":"capture","pane":"Parm","rect":[0,0,400,200],"file":"parameter-header.png"}
```

Only load trusted projects. Houdini projects and node types may execute scripts
inside Houdini. An isolated desktop does not provide a security sandbox.
The harness does not save projects or alter the original project file.

## Evidence

- `scenario.json`: exact requested actions.
- `startup.json`: Houdini version and process ID.
- `progress.json`: last attempted step.
- `result.json`: completed steps or the failing step and traceback.
- `launch.json`: executable, private desktop, process, timing, and foreground observations.
- `*.png`: captures.
- `*.png.json`: capture hash, dimensions, region, and UI inventory.

Parameter data uses separate filenames, such as `box-parameters.json`.
Capture metadata uses `box-parameters.png.json` and cannot replace that export.
Run output stays local under the ignored `.captures` directory.

## Tests

```powershell
python -m unittest discover -s tools/houdini_capture -p test_harness.py -v
```

The unit tests cover output containment, evidence collisions, invalid actions,
and bounded settling delays. The modeling scenario is the integration test.
Inspect its screenshots before treating a run as a visual reference.

## Current limits

Verified against Houdini Apprentice 22.0.466 on this Windows host.
The modeling scenario exercises node selection, a parameter panel, a node graph,
and the geometry spreadsheet. Other desktops, imported projects, dialogs,
menus, mixed DPI, and other Houdini versions need separate verification.

The first QWidget capture backend showed dotted artifacts in some parameter
controls. The geometry spreadsheet rendered cleanly. Captures with these
artifacts are diagnostic evidence, not approved visual baselines.
Qt's offscreen platform did not reach the capture script within the probe timeout.

The harness uses fixed settling delays, not a proven render-completion signal.
Slow cooks may need a longer delay. Startup dialogs or unavailable licenses can
cause timeout. No authentication or license dialogs are automated.

## Verification on 2026-10-14

The final repository run, `.captures/modeling-002`, passed all 15 scenario steps.
It created a box, selected it, exported 14 parameter tuples, changed panes,
and captured full windows, individual panes, and a 500x200 logical-pixel region.
Four regression tests and Python compilation checks passed.
Parameter exports and screenshot sidecars remained separate.

The geometry spreadsheet and its region crop were inspected and rendered cleanly.
Parameter controls retained dotted artifacts, including after rebuilding the pane.
Those captures are not approved visual baselines. Native PrintWindow and Qt
screen capture returned blank content on the inactive desktop. QWidget capture
remains the implemented backend. Next work is capture fidelity for parameter
controls, followed by wider UI coverage and the egui/wgpu integration build.
