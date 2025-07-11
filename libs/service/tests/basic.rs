use bevy::{log::LogPlugin, prelude::*};
use q_service::prelude::*;

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
enum TestErr {
    #[error("A")]
    A,
}

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
    app.add_service(SimpleServiceSpec::<TestErr>::new("MyService".to_string()));
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&SimpleService<TestErr>>();
    let s = services
        .iter(world)
        .find(|s| s.name == "MyService")
        .unwrap();
    assert_eq!(s.name, "MyService");
    assert_eq!(s.hooks, ServiceHooks::<TestErr>::default());
    assert_eq!(s.state, ServiceState::Uninitialized)
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.add_service(
        SimpleServiceSpec::<TestErr>::new("MyService".to_string())
            .is_startup(true)
            .on_init(|_| Err(TestErr::A)),
    );
    let world = app.world_mut();
    let mut services = world.query::<&SimpleService<TestErr>>();
    let s = services
        .iter(world)
        .find(|s| s.name == "MyService")
        .unwrap();
    assert_eq!(s.name, "MyService");
    assert_eq!(s.state, ServiceState::Failed(TestErr::A));
}

#[test]
fn manual_init() {
    let mut app = setup();
    app.add_service(SimpleServiceSpec::<TestErr>::new("MyService".to_string()));
    app.update();
    app.world_mut()
        .commands()
        .init_service::<String, (), TestErr>("MyService".to_string());
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&SimpleService<TestErr>>();
    let s = services
        .iter(world)
        .find(|s| s.name == "MyService")
        .unwrap();
    assert_eq!(s.name, "MyService");
    assert_eq!(s.state, ServiceState::Enabled);
}

// TODO: Test hooks
// TODO: Test dependencies
// TODO: Test run conditions
