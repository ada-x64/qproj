//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_dolly::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::{TnuaAvian3dPlugin, TnuaAvian3dSensorShape};
use cam::{PlayerCamDriver, PlayerCamRigOptionsBuilder};
use q_debug::uv_debug_texture;
use q_worldgen::util::SpawnAroundTracker;
use std::f32::consts::PI;

mod cam;
#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(States, Debug, Hash, Eq, PartialEq, Copy, Clone, Default)]
pub enum PlayerState {
    #[default]
    Init,
    Active,
    Inactive,
}
impl From<PlayerState> for bool {
    fn from(value: PlayerState) -> bool {
        match value {
            PlayerState::Active => true,
            PlayerState::Init | PlayerState::Inactive => false,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct PlayerCam;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .init_state::<PlayerState>()
        .add_systems(
            FixedUpdate,
            apply_controls.in_set(TnuaUserControlsSystemSet),
        )
        .add_systems(OnExit(PlayerState::Init), init)
        .add_systems(
            Update,
            (Dolly::<PlayerCam>::update_active, update_camera)
                .run_if(in_state(PlayerState::Active)),
        );
    }
}

// TODO: This position needs to vary depending on the terrain. Probably want to wait until it's loaded.
// But, game state should wait until terrain is loaded to transition to PlayerState::Active
pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    debug!("EXITING PLAYERSTATE::INIT");
    let pos = Vec3::ZERO;
    let capsule = meshes.add(Capsule3d::new(0.5, 1.));
    let sphere = meshes.add(Sphere::new(1.));
    let img = images.add(uv_debug_texture());
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(img),
        ..Default::default()
    });
    commands
        .spawn((
            Player,
            Mesh3d(capsule),
            MeshMaterial3d(material.clone()),
            Name::new("Player"),
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.)),
            LockedAxes::ROTATION_LOCKED,
            SpawnAroundTracker,
            Transform::from_translation(pos),
        ))
        .with_child((
            Camera3d::default(),
            PlayerCam,
            Rig::builder()
                .with(PlayerCamDriver::new(
                    PlayerCamRigOptionsBuilder::default()
                        .rot_smoothing(0.)
                        .build()
                        .unwrap(),
                ))
                .build(),
            Transform::default(),
            Collider::sphere(1.),
            Mesh3d(sphere),
            MeshMaterial3d(material),
            PointLight::default(),
        ));
}

#[allow(clippy::type_complexity)]
pub fn update_camera(
    player_tf: Single<&Transform, With<Player>>,
    mut rig_tf: Single<&mut Rig, With<PlayerCam>>,
) {
    rig_tf.driver_mut::<PlayerCamDriver>().set_position(
        player_tf.translation,
        Quat::from_axis_angle(player_tf.forward().as_vec3(), PI / 3.),
    );
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController>,
) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        desired_forward: Dir3::new(direction.normalize_or_zero()).ok(),
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 1.5,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }
}
