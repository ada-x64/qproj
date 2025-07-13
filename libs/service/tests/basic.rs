use bevy::{log::LogPlugin, prelude::*};
use q_service::{helpers::service_has_state, prelude::*};

#[derive(ServiceError, Debug, thiserror::Error, Clone, Copy, PartialEq)]
enum TestErr {
    #[error("A")]
    A,
}

#[derive(ServiceName, Debug, Clone, Hash, PartialEq, Eq)]
enum TestServiceNames {
    Test,
}

service!(Test, TestServiceNames, (), TestErr);

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
    app.add_service(TEST_SERVICE_SPEC);
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::Test)
        .unwrap();
    assert_eq!(s.name, TestServiceNames::Test);
    assert_eq!(s.hooks, TestServiceHooks::default());
    assert_eq!(s.state, ServiceState::Uninitialized)
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.add_service(
        TEST_SERVICE_SPEC
            .is_startup(true)
            .on_init(|_| Err(TestErr::A)),
    );
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::Test)
        .unwrap();
    assert_eq!(s.state, ServiceState::Failed(TestErr::A));
}

#[test]
fn manual_init() {
    let mut app = setup();
    app.add_service(TEST_SERVICE_SPEC);
    app.update();
    app.world_mut().commands().init_service(TEST_SERVICE);
    app.update();
    let world = app.world_mut();
    let mut services = world.query::<&TestService>();
    let s = services
        .iter(world)
        .find(|s| s.name == TestServiceNames::Test)
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

fn run_all_hooks() -> TestServiceSpec {
    TEST_SERVICE_SPEC
        .is_startup(true)
        .on_init(|world| {
            world.resource_mut::<TestHooks>().init = true;
            Ok(true)
        })
        .on_enable(|world| {
            world.resource_mut::<TestHooks>().enable = true;
            world.commands().disable_service(TEST_SERVICE);
            Ok(())
        })
        .on_disable(|world| {
            world.resource_mut::<TestHooks>().disable = true;
            Err(TestErr::A)
        })
        .on_failure(|_err, world| {
            world.resource_mut::<TestHooks>().fail = true;
        })
}
#[test]
fn hooks() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    app.add_service(run_all_hooks());
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
    // NOTE: This fails with `is_startup(true)`. Probably because observers need
    // to be instantiated before events can fire.
    app.add_service(TEST_SERVICE_SPEC).add_observer(
        |t: Trigger<TestServiceStateChange>,
         mut r: ResMut<TestHooks>,
         mut commands: Commands| {
            match t.event().new_state {
                ServiceState::Initializing => {
                    r.init = true;
                }
                ServiceState::Enabled => {
                    r.enable = true;
                    commands.disable_service(TEST_SERVICE);
                }
                ServiceState::Disabled => {
                    r.disable = true;
                    commands.fail_service(TEST_SERVICE, TestErr::A);
                }
                ServiceState::Failed(_) => r.fail = true,
                _ => {}
            }
        },
    );
    app.world_mut().commands().init_service(TEST_SERVICE);
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

#[derive(Resource, Default, Debug, PartialEq)]
struct Ran {
    service_has_state: bool,
    service_uninitialized: bool,
    // service_initializing: bool,
    service_enabled: bool,
    service_disabled: bool,
    service_failed: bool,
    service_failed_with_error: bool,
}

macro_rules! check_run_condition {
    ($app:ident, $condition:ident) => {
        $app.add_systems(
            Update,
            (|mut ran: ResMut<Ran>| {
                ran.$condition = true;
            })
            .run_if($condition(TEST_SERVICE)),
        );
    };
}

#[test]
fn run_conditions() {
    let mut app = setup();
    app.init_resource::<Ran>();
    app.add_service(TEST_SERVICE_SPEC);
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_has_state = true;
        })
        .run_if(service_has_state(TEST_SERVICE, ServiceState::Enabled)),
    );
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_failed_with_error = true;
        })
        .run_if(service_failed_with_error(TEST_SERVICE, TestErr::A)),
    );
    check_run_condition!(app, service_uninitialized);
    // check_run_condition!(app, service_initializing);
    check_run_condition!(app, service_enabled);
    check_run_condition!(app, service_disabled);
    check_run_condition!(app, service_failed);

    app.update();
    app.world_mut().commands().init_service(TEST_SERVICE);
    app.update();
    app.world_mut().commands().enable_service(TEST_SERVICE);
    app.update();
    app.world_mut().commands().disable_service(TEST_SERVICE);
    app.update();
    app.world_mut()
        .commands()
        .fail_service(TEST_SERVICE, TestErr::A);
    app.update();

    let all_ok = Ran {
        service_has_state: true,
        service_uninitialized: true,
        // TODO: This will only be called if initializing takes more than one
        // frame! Need async init before we can test this.
        // service_initializing: true,
        service_enabled: true,
        service_disabled: true,
        service_failed: true,
        service_failed_with_error: true,
    };
    assert_eq!(app.world().resource::<Ran>(), &all_ok);
}

// TODO: Dependency initialization
// TODO: Implement DAG for deps
// TODO: Dependency error propagation
// TODO: Auto-initialize when enabled
// ------> should be configurable
// TODO: Async initialization
// ------> maybe do Initializing(f32) (gloss as percentage)
// TODO: Minimize bevy dependencies (just ECS?)
