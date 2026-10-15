// SPDX-FileCopyrightText: 2008-2026 Blender Authors
// SPDX-FileCopyrightText: 2026 LokiCraft contributors
// SPDX-License-Identifier: GPL-2.0-or-later
//! Rust navigation port. Source functions and deliberate host adaptations are
//! listed in docs/blender-port/coverage.md. This module has no egui dependency.
use glam::{Quat, Vec2, Vec3};
use std::f32::consts::{FRAC_1_SQRT_2, PI};

pub const TRACKBALL_RADIUS: f32 = 1.1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection {
    Perspective,
    Orthographic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orbit {
    Turntable,
    Trackball,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Rotate,
    Pan,
    Zoom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Blender's viewquat: world-to-view rotation; camera looks down local -Z.
    pub rotation: Quat,
    /// Blender's ofs: negative orbit center in world space.
    pub offset: Vec3,
    pub distance: f32,
    pub projection: Projection,
    pub axis: Option<Axis>,
    pub fov_y: f32,
}
impl Default for View {
    fn default() -> Self {
        Self {
            rotation: Quat::from_rotation_x(-1.05) * Quat::from_rotation_z(-0.6),
            offset: Vec3::ZERO,
            distance: 18.0,
            projection: Projection::Perspective,
            axis: None,
            fov_y: 50.0_f32.to_radians(),
        }
    }
}
impl View {
    pub fn center(&self) -> Vec3 {
        -self.offset
    }
    pub fn units_per_pixel(&self, size: Vec2) -> f32 {
        2.0 * self.distance * (self.fov_y * 0.5).tan() / size.y.max(1.0)
    }
    pub fn window_delta(&self, delta: Vec2, size: Vec2) -> Vec3 {
        self.rotation.conjugate() * Vec3::new(delta.x, delta.y, 0.0) * self.units_per_pixel(size)
    }
    /// Port of viewmove_apply's unlocked, non-camera branch. Input uses bottom-left coordinates.
    pub fn pan(&mut self, delta: Vec2, size: Vec2) {
        self.offset += self.window_delta(delta, size);
    }
    /// Port of view_zoom_to_window_xy_3d. Clamp is a LokiCraft host range.
    pub fn zoom_factor(&mut self, factor: f32, cursor: Option<Vec2>, size: Vec2) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let next = (self.distance * factor).clamp(0.01, 1_000_000.0);
        let factor = next / self.distance;
        if let Some(cursor) = cursor {
            let dvec = self.window_delta(cursor - size * 0.5, size);
            self.offset += dvec * (factor - 1.0);
        }
        self.distance = next;
    }
    /// Blender's 1.2 wheel step, generalized to fractional steps for touchpads.
    pub fn zoom_steps(&mut self, steps: f32, cursor: Option<Vec2>, size: Vec2) {
        self.zoom_factor(1.2_f32.powf(-steps), cursor, size);
    }
    pub fn set_axis(&mut self, axis: Axis) {
        // Look directions match Blender's Z-up world. UI adapter remaps Houdini Y-up separately.
        self.rotation = match axis {
            Axis::Top => Quat::IDENTITY,
            Axis::Bottom => Quat::from_rotation_x(PI),
            Axis::Front => Quat::from_rotation_x(-PI * 0.5),
            Axis::Back => Quat::from_rotation_x(-PI * 0.5) * Quat::from_rotation_z(PI),
            Axis::Right => Quat::from_rotation_x(-PI * 0.5) * Quat::from_rotation_z(-PI * 0.5),
            Axis::Left => Quat::from_rotation_x(-PI * 0.5) * Quat::from_rotation_z(PI * 0.5),
        };
        self.axis = Some(axis);
        self.projection = Projection::Orthographic;
    }
    pub fn project_segment(&self, mut a: Vec3, mut b: Vec3, size: Vec2) -> Option<[Vec2; 2]> {
        let da = self.distance - (self.rotation * (a + self.offset)).z;
        let db = self.distance - (self.rotation * (b + self.offset)).z;
        let near = 0.01;
        if da < near && db < near {
            return None;
        }
        if da < near {
            a = a.lerp(b, (near - da) / (db - da));
        } else if db < near {
            b = b.lerp(a, (near - db) / (da - db));
        }
        Some([self.project(a, size)?, self.project(b, size)?])
    }
    /// Host projection adapter; world points are already in Blender's Z-up basis.
    pub fn project(&self, point: Vec3, size: Vec2) -> Option<Vec2> {
        let local = self.rotation * (point + self.offset);
        let depth = self.distance - local.z;
        if depth <= 0.001 {
            return None;
        }
        let scale = if self.projection == Projection::Perspective {
            size.y / (2.0 * depth * (self.fov_y * 0.5).tan())
        } else {
            1.0 / self.units_per_pixel(size)
        };
        Some(size * 0.5 + Vec2::new(local.x, local.y) * scale)
    }
}
/// Direct port of calctrackballvec. Coordinates and viewport size are logical pixels, Y-up.
pub fn trackball_vector(size: Vec2, point: Vec2) -> Vec3 {
    let minimum = size.min_element().max(1.0);
    let xy = (point - size * 0.5) / (minimum * 0.5);
    let d = xy.length();
    let t = TRACKBALL_RADIUS * FRAC_1_SQRT_2;
    let z = if d < t {
        (TRACKBALL_RADIUS * TRACKBALL_RADIUS - d * d).sqrt()
    } else {
        t * t / d
    };
    xy.extend(z)
}
/// Direct port of the trackball branch of viewrotate_apply, without scene callbacks.
pub fn trackball_rotation(
    initial: Quat,
    size: Vec2,
    start: Vec2,
    current: Vec2,
    sensitivity: f32,
) -> Quat {
    let first = trackball_vector(size, start);
    let next = trackball_vector(size, current);
    let angle = ((next - first).length() / (2.0 * TRACKBALL_RADIUS)) * PI * sensitivity;
    let wrapped = (angle + PI).rem_euclid(2.0 * PI) - PI;
    let axis = first.cross(next);
    if axis.length_squared() < 1e-16 {
        return initial;
    }
    (Quat::from_axis_angle(axis.normalize(), wrapped) * initial).normalize()
}
/// Direct port of Blender's turntable blend, including recovery from a rolled horizon.
pub fn turntable_rotation(current: Quat, delta: Vec2, sensitivity: f32, reverse: f32) -> Quat {
    let inverse = current.conjugate();
    let view_x = inverse * Vec3::X;
    let view_z = inverse * Vec3::Z;
    let xaxis = if Vec3::Z.distance_squared(view_z) > 0.001 {
        let mut x = Vec3::Z.cross(view_z);
        if x.dot(view_x) < 0.0 {
            x = -x;
        }
        let fac = ((Vec3::Z.dot(view_z).clamp(-1.0, 1.0).acos() / PI - 0.5).abs() * 2.0).powi(2);
        x.lerp(view_x, fac)
    } else {
        view_x
    };
    let local = if xaxis.length_squared() > 1e-16 {
        Quat::from_axis_angle(xaxis.normalize(), -delta.y * sensitivity)
    } else {
        Quat::IDENTITY
    };
    (current * local * Quat::from_rotation_z(sensitivity * reverse * delta.x)).normalize()
}
/// viewzoom_scale_value's vertical USER_ZOOM_DOLLY branch, at UI scale 1.
pub fn dolly_factor(height: f32, start_y: f32, current_y: f32) -> f32 {
    let old = 5.0 + height - start_y;
    let new = 5.0 + height - current_y;
    2.0 * (new / old.max(1.0) - 1.0) + 1.0
}
/// Direct port of view3d_orbit_apply_dyn_ofs.
pub fn orbit_offset(
    old_offset: Vec3,
    old_rotation: Quat,
    new_rotation: Quat,
    pivot_offset: Vec3,
) -> Vec3 {
    let rotation = (old_rotation.conjugate() * new_rotation).conjugate();
    rotation * (old_offset - pivot_offset) + pivot_offset
}
#[derive(Clone, Copy, Debug)]
struct Gesture {
    backup: View,
    base: View,
    start: Vec2,
    previous: Vec2,
    mode: Mode,
    reverse: f32,
}
#[derive(Debug)]
pub struct Navigation {
    pub view: View,
    pub orbit: Orbit,
    pub zoom_to_cursor: bool,
    gesture: Option<Gesture>,
}
impl Default for Navigation {
    fn default() -> Self {
        Self {
            view: View::default(),
            orbit: Orbit::Turntable,
            zoom_to_cursor: true,
            gesture: None,
        }
    }
}
impl Navigation {
    pub fn active(&self) -> bool {
        self.gesture.is_some()
    }
    pub fn begin(&mut self, mode: Mode, position: Vec2) {
        let backup = self.view;
        if mode == Mode::Rotate {
            self.view.projection = Projection::Perspective;
            self.view.axis = None;
        }
        let reverse = if (self.view.rotation * Vec3::Z).y < 0.0 {
            -1.0
        } else {
            1.0
        };
        self.gesture = Some(Gesture {
            backup,
            base: self.view,
            start: position,
            previous: position,
            mode,
            reverse,
        });
    }
    pub fn update(&mut self, mode: Mode, position: Vec2, size: Vec2) {
        let Some(mut g) = self.gesture else {
            return;
        };
        if mode != g.mode {
            g.mode = mode;
            g.base = self.view;
            g.start = g.previous;
            if mode == Mode::Rotate {
                self.view.projection = Projection::Perspective;
                self.view.axis = None;
            }
        }
        match mode {
            Mode::Rotate => {
                self.view.rotation = match self.orbit {
                    Orbit::Trackball => {
                        trackball_rotation(g.base.rotation, size, g.start, position, 1.0)
                    }
                    Orbit::Turntable => turntable_rotation(
                        self.view.rotation,
                        position - g.previous,
                        0.007,
                        g.reverse,
                    ),
                };
            }
            Mode::Pan => self.view.pan(position - g.previous, size),
            Mode::Zoom => {
                let desired = g.base.distance * dolly_factor(size.y, g.start.y, position.y);
                let factor = (desired / self.view.distance).max(0.0001);
                self.view
                    .zoom_factor(factor, self.zoom_to_cursor.then_some(g.previous), size);
            }
        }
        g.previous = position;
        self.gesture = Some(g);
    }
    pub fn confirm(&mut self) {
        self.gesture = None;
    }
    pub fn cancel(&mut self) {
        if let Some(g) = self.gesture.take() {
            self.view = g.backup;
        }
    }
}
