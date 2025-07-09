// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{prelude::*, scene::inspector_cam::InspectorCam};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::PrimaryWindow,
};
use q_utils::InspectorIgnore;

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct RenderToTextureGroup;

#[derive(Component, Default, Copy, Clone)]
pub struct AxesCam;
#[derive(Component, Default, Copy, Clone)]
pub struct AxesNode;

pub fn render_axes(
    cam_tf: Single<&Transform, (With<InspectorCam>, Without<AxesCam>)>,
    mut cam2_tf: Single<&mut Transform, (With<AxesCam>, Without<InspectorCam>)>,
    mut gizmos: Gizmos<RenderToTextureGroup>,
    mut node: Single<&mut Node, With<AxesNode>>,
    ui_state: Res<UiState>,
    window: Single<&mut Window, With<PrimaryWindow>>,
) {
    gizmos.axes(Isometry3d::IDENTITY, 5.);
    **cam2_tf = cam_tf.with_translation(
        Vec3::new(0., 0., 0.) - cam_tf.forward().as_vec3() * 15.,
    );
    let state = &ui_state.tab_data;
    node.top = Val::Px(state.viewport_rect.top());
    node.right =
        Val::Px(window.physical_width() as f32 - state.viewport_rect.right());
}

// TODO: Implement outline shader. Apply it to selected entities.
// see this: https://github.com/komadori/bevy_mod_outline/tree/master
// Probably want vertex extrusion.
// fn highlight_selected(ui_state: Res<UiState>) {
//     ui_state.selected_entities.iter().for_each(|e| e)
// }

/// Allows us to render to a bevy_ui instance
pub(crate) fn setup_overlay_ui(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 128,
        height: 128,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image
    // as a render target
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let first_pass_layer = RenderLayers::layer(1);

    // TODO: Move this.
    commands.spawn((
        AxesCam,
        Name::new("Axes Cam"),
        InspectorIgnore,
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        first_pass_layer,
    ));
    let node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.),
        right: Val::Px(0.),
        width: Val::Px(64.),
        height: Val::Px(64.),
        ..Default::default()
    };
    let image_node = ImageNode::new(image_handle);
    commands.spawn((AxesNode, node)).with_child(image_node);
}
