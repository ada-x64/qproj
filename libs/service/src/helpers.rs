use crate::*;

pub type SimpleService<E> = Service<String, (), E>;
pub type SimpleServiceSpec<E> = ServiceSpec<String, (), E>;
