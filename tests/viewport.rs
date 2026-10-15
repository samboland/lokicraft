use glam::{Quat, Vec2, Vec3};
use lokicraft::viewport::*;

#[test]
fn trackball_matches_compiled_blender_source() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../tools/blender_reference/trackball.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let pair = |key: &str| {
            Vec2::new(
                case[key][0].as_f64().unwrap() as f32,
                case[key][1].as_f64().unwrap() as f32,
            )
        };
        let actual = trackball_vector(pair("size"), pair("point"));
        for i in 0..3 {
            assert!(
                (actual[i] - case["expected"][i].as_f64().unwrap() as f32).abs() < 2e-6,
                "{case}"
            );
        }
    }
}
#[test]
fn trackball_preserves_aspect_and_start_orientation() {
    let a = trackball_vector(Vec2::new(800.0, 600.0), Vec2::new(450.0, 320.0));
    let b = trackball_vector(Vec2::new(1600.0, 600.0), Vec2::new(850.0, 320.0));
    assert!(a.abs_diff_eq(b, 1e-6));
    let q = Quat::from_rotation_y(0.7);
    assert!(
        trackball_rotation(
            q,
            Vec2::splat(600.0),
            Vec2::splat(300.0),
            Vec2::splat(300.0),
            1.0
        )
        .abs_diff_eq(q, 1e-6)
    );
}
#[test]
fn cursor_zoom_preserves_projected_target_in_both_projections() {
    let size = Vec2::new(1000.0, 700.0);
    let cursor = Vec2::new(700.0, 460.0);
    for projection in [Projection::Perspective, Projection::Orthographic] {
        let mut view = View {
            projection,
            ..View::default()
        };
        let target = view.center() + view.window_delta(cursor - size * 0.5, size);
        let before = view.project(target, size).unwrap();
        view.zoom_steps(2.0, Some(cursor), size);
        assert!(
            view.project(target, size)
                .unwrap()
                .abs_diff_eq(before, 0.001)
        );
    }
}
#[test]
fn pan_tracks_pointer_at_orbit_depth() {
    let mut view = View::default();
    let size = Vec2::new(800.0, 600.0);
    let delta = Vec2::new(73.0, -40.0);
    let target = view.center();
    let before = view.project(target, size).unwrap();
    view.pan(delta, size);
    assert!(
        view.project(target, size)
            .unwrap()
            .abs_diff_eq(before + delta, 0.001)
    );
}
#[test]
fn cancel_restores_axis_projection_after_rotate_and_mode_switch() {
    let mut nav = Navigation::default();
    nav.view.set_axis(Axis::Top);
    let before = nav.view;
    nav.begin(Mode::Rotate, Vec2::splat(100.0));
    nav.update(Mode::Rotate, Vec2::splat(150.0), Vec2::splat(600.0));
    nav.update(Mode::Pan, Vec2::splat(200.0), Vec2::splat(600.0));
    nav.cancel();
    assert_eq!(nav.view, before);
    assert!(!nav.active());
}
#[test]
fn turntable_remains_normalized_through_poles_and_rolled_horizons() {
    for seed in [
        Quat::IDENTITY,
        Quat::from_rotation_z(1.57),
        Quat::from_rotation_x(std::f32::consts::PI),
    ] {
        let mut q = seed;
        for _ in 0..1000 {
            q = turntable_rotation(q, Vec2::new(4.0, 7.0), 0.007, 1.0);
            assert!(q.is_finite());
            assert!((q.length() - 1.0).abs() < 1e-5);
        }
    }
}
#[test]
fn axis_views_and_near_clipping_stay_finite() {
    let mut view = View::default();
    let size = Vec2::new(800.0, 600.0);
    for axis in [
        Axis::Front,
        Axis::Back,
        Axis::Right,
        Axis::Left,
        Axis::Top,
        Axis::Bottom,
    ] {
        view.set_axis(axis);
        assert!(
            view.project(view.center(), size)
                .unwrap()
                .abs_diff_eq(size * 0.5, 0.001)
        );
    }
    view.rotation = Quat::IDENTITY;
    assert!(
        view.project_segment(Vec3::new(1.0, 0.0, view.distance + 1.0), Vec3::ZERO, size)
            .unwrap()
            .iter()
            .all(|v| v.is_finite())
    );
    assert!(
        view.project_segment(
            Vec3::Z * (view.distance + 2.0),
            Vec3::Z * (view.distance + 1.0),
            size
        )
        .is_none()
    );
}
#[test]
fn wheel_step_reverses_and_zoom_clamps() {
    let mut view = View::default();
    let original = view.distance;
    let size = Vec2::splat(600.0);
    view.zoom_steps(1.0, None, size);
    assert!((view.distance - original / 1.2).abs() < 1e-6);
    view.zoom_steps(-1.0, None, size);
    assert!((view.distance - original).abs() < 1e-5);
    view.zoom_factor(0.0000001, None, size);
    assert_eq!(view.distance, 0.01);
    view.zoom_factor(f32::NAN, None, size);
    assert!(view.distance.is_finite());
}
#[test]
fn dynamic_orbit_offset_preserves_pivot() {
    let pivot = Vec3::new(2.0, 3.0, 4.0);
    let old = Quat::IDENTITY;
    let new = Quat::from_rotation_z(1.0);
    assert!(orbit_offset(pivot, old, new, pivot).abs_diff_eq(pivot, 1e-6));
    let offset = Vec3::new(4.0, 1.0, 0.0);
    assert!(
        (orbit_offset(offset, old, new, pivot).distance(pivot) - offset.distance(pivot)).abs()
            < 1e-5
    );
}
#[test]
fn dolly_is_identity_at_start_and_matches_upstream_ratio() {
    assert_eq!(dolly_factor(600.0, 100.0, 100.0), 1.0);
    assert!((dolly_factor(600.0, 100.0, 200.0) - (2.0 * (405.0 / 505.0 - 1.0) + 1.0)).abs() < 1e-6);
}

#[test]
fn pointer_drag_and_escape_use_navigation_transaction() {
    use egui::{Event, Key, Modifiers, PointerButton, pos2};
    use egui_kittest::Harness;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 960.0))
        .build_eframe(|cc| lokicraft::Shell::new(&cc.egui_ctx, craft_settings::Location::Memory));
    h.run();
    let original = h.state().navigation.view;
    h.event(Event::PointerMoved(pos2(400.0, 350.0)));
    h.event(Event::PointerButton {
        pos: pos2(400.0, 350.0),
        button: PointerButton::Middle,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    h.event(Event::PointerMoved(pos2(480.0, 410.0)));
    h.run_steps(3);
    assert!(h.state().navigation.active());
    assert_ne!(h.state().navigation.view.rotation, original.rotation);
    h.event(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert_eq!(h.state().navigation.view, original);
    assert!(!h.state().navigation.active());
}
