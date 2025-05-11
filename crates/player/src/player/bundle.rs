//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use q_worldgen::util::SpawnAroundTracker;
#[derive(Component, Default, Debug)]
pub struct Player;
pub struct PlayerBundle;
impl PlayerBundle {
    /// TODO: Pass in a model with all the animations and etc.
    /// Alternatively, initialize the bundle before returning.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<M: Material>(
        transform: Transform,
        mesh: Handle<Mesh>,
        material: Handle<M>,
    ) -> impl Bundle {
        (
            Name::new("Player"),
            Player,
            transform,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.)),
            LockedAxes::ROTATION_LOCKED,
            SpawnAroundTracker,
        )
    }
}
