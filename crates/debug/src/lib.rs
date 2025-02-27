use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Default)]
pub struct DebugPlugin {
    pub debug_level: DebugLevel,
}

/// Visual debugging level for gizmos etc.
#[derive(Default, Component, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct DebugLevel(pub usize);

#[derive(Component, Default)]
pub struct DebugComponent;

#[derive(Component, Default)]
pub struct ShowAxes(pub Option<(DebugLevel, f32)>);

#[derive(Bundle, Default)]
pub struct DebugBundle {
    pub marker: DebugComponent,
    pub show_axes: ShowAxes,
}

fn draw_debug(
    mut gizmos: Gizmos,
    q: Query<(&DebugLevel, &ShowAxes, &Transform), With<DebugComponent>>,
) {
    q.iter().for_each(|(level, show, tf)| {
        if let Some(inner) = show.0 {
            let (show_level, length) = inner;
            if *level <= show_level {
                gizmos.axes(*tf, length);
            }
        }
    });
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                enabled: true,
                ..Default::default()
            },
        })
        .add_systems(
            Update,
            (
                draw_debug,
                //...
            ),
        );
    }
}
