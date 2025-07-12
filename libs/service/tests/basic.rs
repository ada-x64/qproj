use bevy::{log::LogPlugin, prelude::*};
use q_service::{helpers::service_has_state, prelude::*};

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

#[derive(Resource, Debug, Default, PartialEq)]
pub struct TestHooks {
    init: bool,
    enable: bool,
    disable: bool,
    fail: bool,
}
#[test]
fn hooks() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    let spec = TestServiceSpec::new(TestServiceNames::A)
        .is_startup(true)
        .on_init(|world| {
            world.resource_mut::<TestHooks>().init = true;
            Ok(true)
        })
        .on_enable(|world| {
            world.resource_mut::<TestHooks>().enable = true;
            world
                .commands()
                .disable_service(TestServiceNames::A, TEST_SERVICE_MARKER);
            Ok(())
        })
        .on_disable(|world| {
            world.resource_mut::<TestHooks>().disable = true;
            Err(TestErr::A)
        })
        .on_failure(|_err, world| {
            world.resource_mut::<TestHooks>().fail = true;
        });
    app.add_service(spec);
    app.update();
    assert_eq!(
        app.world_mut().resource::<TestHooks>(),
        &TestHooks {
            init: true,
            enable: true,
            disable: true,
            fail: true,
        }
    );
}

#[test]
fn events() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    let spec = TestServiceSpec::new(TestServiceNames::A).is_startup(true);
    app.add_service(spec).add_observer(
        |t: Trigger<TestServiceStateChange>,
         mut r: ResMut<TestHooks>,
         mut commands: Commands| {
            match t.event().new_state {
                ServiceState::Initializing => {
                    r.init = true;
                }
                ServiceState::Enabled => {
                    r.enable = true;
                    commands.disable_service(
                        TestServiceNames::A,
                        TEST_SERVICE_MARKER,
                    );
                }
                ServiceState::Disabled => {
                    r.disable = true;
                    commands.fail_service(
                        TestServiceNames::A,
                        TestErr::A,
                        TEST_SERVICE_MARKER,
                    );
                }
                ServiceState::Failed(_) => r.fail = true,
                _ => {}
            }
        },
    );
    app.world_mut()
        .commands()
        .init_service(TestServiceNames::A, TEST_SERVICE_MARKER);
    app.update();
    assert_eq!(
        app.world_mut().resource::<TestHooks>(),
        &TestHooks {
            init: true,
            enable: true,
            disable: true,
            fail: true,
        }
    );
}

#[derive(Resource, Default)]
struct Ran(bool);

#[test]
fn run_conditions() {
    let mut app = setup();
    app.init_resource::<Ran>();
    app.add_service(TestServiceSpec::new(TestServiceNames::A).is_startup(true));
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.0 = true;
        })
        .run_if(service_has_state(
            TestServiceNames::A,
            ServiceState::Enabled,
            TEST_SERVICE_MARKER,
        )),
    );
    app.update();
    assert!(app.world().resource::<Ran>().0);
}

// TODO: Dependency initialization
// TODO: Implement DAG for deps
// TODO: Dependency error propagation
// TODO: Auto-initialize when enabled
// ------> should be configurable
// TODO: Async initialization
// ------> maybe do Initializing(f32) (gloss as percentage)
