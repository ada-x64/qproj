//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    asset::RenderAssetUsages,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Default)]
pub struct DebugPlugin {
    pub debug_level: DebugLevel,
    pub wireframes: bool,
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

        if self.wireframes {
            app.add_plugins(WireframePlugin);
            app.insert_resource(WireframeConfig {
                global: true,
                ..Default::default()
            });
        }
    }
}

/// Creates a colorful test pattern
pub fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255,
        102, 255, 102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255,
        236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)]
            .copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
