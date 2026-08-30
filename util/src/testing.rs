use bevy::app::App;
use bevy::log::tracing_subscriber;
use bevy::math::Vec3;

#[track_caller]
pub fn expect_float(actual: f32, expect: f32) {
    if expect.is_nan() {
        assert!(actual.is_nan(), "expect {actual:?} to be nan");
    } else if expect.is_infinite() {
        assert!(
            actual.is_infinite() && actual.signum() == expect.signum(),
            "expect {actual:?} to be {expect:?}"
        );
    } else if expect == 0.0 {
        assert!(actual.abs() < 1e-4, "expect {actual:?} to be zero");
    } else {
        assert!(
            (actual - expect).abs() <= (expect * 1e-4).abs(),
            "got {actual:?}, expected {expect:?}",
        );
    }
}

#[track_caller]
pub fn expect_vec3(actual: Vec3, expect: Vec3) {
    expect_float(actual.x, expect.x);
    expect_float(actual.y, expect.y);
    expect_float(actual.z, expect.z);
}

#[track_caller]
pub fn expect_float_near(actual: f32, expect: f32, threshold: f32) {
    assert!(expect.is_finite());
    assert!(
        (actual - expect).abs() <= threshold,
        "got {actual:?}, expected {expect:?} within {threshold}"
    );
}

#[track_caller]
pub fn expect_small(actual: f32, max_abs: f32) {
    assert!(actual.abs() < max_abs, "got abs({actual:?}), should be smaller than {max_abs}");
}

#[track_caller]
pub fn expect_between(actual: f32, min: f32, max: f32) {
    assert!(min < actual && actual < max, "got {actual:?}, should be between {min:?} and {max:?}");
}

pub fn configure_logging(app: &mut App) {
    app.add_plugins(bevy::log::LogPlugin {
        fmt_layer: |_app| {
            Some(Box::new(
                tracing_subscriber::fmt::layer()
                    .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
            ))
        },
        ..Default::default()
    });
}
