use q_service::prelude::ServiceError;
use q_service_macros::*;

#[derive(ServiceError, thiserror::Error, PartialEq, Eq, Debug, Clone)]
enum Err {}

#[derive(ServiceLabel, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Name;

#[derive(ServiceData, Clone, Debug, PartialEq, Default)]
struct Data;
