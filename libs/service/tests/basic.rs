use bevy::{log::LogPlugin, prelude::*};
use q_service::prelude::*;

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
enum TestErr {
    #[error("A")]
    A,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum TestServiceNames {
    A,
}

alias!(Test, TestServiceNames, TestErr);

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "debug".into(),
            ..Default::default()
        },
    ));
    app
}

#[test]
fn simple() {
    let mut app = setup();
    app.add_service(TestServiceSpec::new(TestServiceNames::A));
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::A)
        .unwrap();
    assert_eq!(s.name, TestServiceNames::A);
    assert_eq!(s.hooks, TestServiceHooks::default());
    assert_eq!(s.state, ServiceState::Uninitialized)
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.add_service(
        TestServiceSpec::new(TestServiceNames::A)
            .is_startup(true)
            .on_init(|_| Err(TestErr::A)),
    );
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::A)
        .unwrap();
    assert_eq!(s.state, ServiceState::Failed(TestErr::A));
}

#[test]
fn manual_init() {
    let mut app = setup();
    app.add_service(TestServiceSpec::new(TestServiceNames::A));
    app.update();
    app.world_mut()
        .commands()
        .init_service(TestServiceNames::A, TEST_SERVICE_MARKER);
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::A)
        .unwrap();
    assert_eq!(s.state, ServiceState::Enabled);
}

// TODO: Test hooks
// TODO: Don't require that hooks are exclusive
// TODO: Test dependencies
// TODO: Test run conditions
