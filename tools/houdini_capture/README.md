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

## All shelf views

Run `scenarios/shelves.json` at 2560x1440 with a 600-second timeout.
The worker inventories every loaded shelf, including empty shelves, and expands
its queue dynamically. It displays original tools in original order in temporary
full-width shelves, with at most 12 tools per page. It does not execute tools.
Definitions are written only inside the capture run's directory.
The 180-pixel captures include the menu, shelf, and upper pane border.
This scenario requires the stated size to keep labels visible.

After completion, run:

```powershell
python tools/houdini_capture/shelf_gallery.py .captures/shelves-001
```

Open `index.html` in the run directory to filter shelves and tool labels.
`shelves.json` records shelf labels, tool names, labels, icons, and page membership.
`coverage.json` records counts after checking image hashes and exact tool order.
These checks establish captured page coverage, not tool behavior or pixel fidelity.

## Context menus

Run `scenarios/contexts.json` at 2560x1440 with a 300-second timeout.
Additional scenarios cover `context-submenus.json` and `creation-menus.json`.
`context_menu` targets a main-window point or a point relative to a named pane.
It sends Qt events inside the owned Houdini process, without desktop input or
cursor movement. Optional `hover: [x, y]` opens an observed menu row; use
`settle_ms: 1500` for these steps. No menu command is selected.
`floating_pane` creates a named disposable pane for additional editor contexts.

Popup widgets are captured separately because they are absent from main-window
captures. Each probe records its target, popup hashes, and capture status under
`contexts/<name>/result.json`. Houdini custom menus do not expose QAction labels;
the PNG is the source for their visible menu text. `no-popup` means the probe did
not produce a new visible popup, not proof that the surface has no context menu.

Package one or more completed or partial runs with:

```powershell
python tools/houdini_capture/context_gallery.py --output .captures/menu-gallery .captures/context-run .captures/submenu-run
```

The gallery verifies hashes and preserves failures and missing probe counts.
Exact-image deduplication does not count semantic menu types. Coordinates and
hover rows are specific to the observed Houdini 22.0.466 layout. Other versions,
different pane sizes, selection states, and deeper submenus remain unverified.

## Sequential UI survey

The eight scenarios in `scenarios/survey` cover top menus, dialogs, deeper menus,
selection states, modeling states, populated editors, gap retries, and hover states.
Run each sequentially at 2560x1440 with a 240-second timeout and a fresh output
folder beneath one survey directory. Build its gallery with:

```powershell
python tools/houdini_capture/survey_gallery.py .captures/survey
```

`context_menu` supports `button: "left"` for top menus, ordered `followups`
with popup-relative `point` and `kind: "hover"` or `"click"`, and `method`
`"hover"` or `"tooltip"` for probes without clicks. Followups use observed menu
coordinates. They can execute commands, so review the scenario before running it.
The included scenarios open dialogs without confirming imports or settings.
`capture_after_ms` extends the capture timer. `settle_ms` must exceed that timer.
Network targets can use `node` or a two-node `wire` instead of fixed coordinates.

`reference_state` creates disposable scene fixtures. Each result under `states`
records success or an error independently of overall scenario completion.
Tool states record the actual active viewer state. A completed scenario alone
does not prove every fixture or screenshot succeeded.

The survey gallery verifies image hashes and lists missing popups and fixture
errors. Optional `review.json` maps run folder names to visual review notes.
Top menus and modeling menus rendered cleanly in the October 14 survey.
Legacy dialogs retained dotted artifacts. Preferences and Color Theme did not
open through the probes. Selected keyframes remain unverified. Network captures
show editor references, not rendered scenes or simulation results. Tooltip
creation can fail because Qt cannot find a screen on the isolated desktop.
