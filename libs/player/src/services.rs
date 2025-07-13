use q_service::prelude::*;

#[derive(ServiceName, Debug, PartialEq, Eq, Hash, Clone)]
pub enum ServiceNames {
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

service!(Player, ServiceNames, (), PlayerError);
service!(PlayerCam, ServiceNames, (), PlayerCamError);
