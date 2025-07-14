use q_service::prelude::*;

#[derive(ServiceLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub enum ServiceLabel {
    PlayerCam,
    Player,
}

#[derive(ServiceError, Debug, Clone, thiserror::Error, PartialEq)]
pub enum PlayerError {}
#[derive(ServiceError, Debug, Clone, thiserror::Error, PartialEq)]
pub enum PlayerCamError {
    #[error("No player camera")]
    NoCam,
}

service!(Player, ServiceLabel, (), PlayerError);
service!(PlayerCam, ServiceLabel, (), PlayerCamError);
