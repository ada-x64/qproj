use q_service::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ServiceNames {
    PlayerCam,
    Player,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PlayerError {}
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PlayerCamError {
    #[error("No player camera")]
    NoCam,
}

alias!(Player, ServiceNames, PlayerError);
alias!(PlayerCam, ServiceNames, PlayerCamError);
