use bevy::{
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

#[derive(Component)]
struct FPSText;

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let fontpath = "fonts/FiraCodeNerdFont-Medium.ttf";
    commands
        .spawn((
            FPSText,
            Text::new("FPS: "),
            TextFont {
                font: assets.load(fontpath),
                ..Default::default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font: assets.load(fontpath),
                ..Default::default()
            },
            FPSText,
        ));
}

fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut q: Query<&mut TextSpan, With<FPSText>>,
) {
    for mut span in &mut q {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (update_fps, draw_debug));
    }
}
