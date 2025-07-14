use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use q_service::prelude::*;

#[derive(ServiceError, Debug, thiserror::Error, Clone, Copy, PartialEq)]
enum TestErr {
    #[error("A")]
    A,
}

#[derive(ServiceLabel, Debug, Clone, Hash, PartialEq, Eq, Default)]
struct TestServiceLabel;

service!(Test, TestServiceLabel, (), TestErr);

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "debug".into(),
            ..Default::default()
        },
    ))
    .add_systems(Startup, || debug!("STARTUP"))
    .add_systems(Update, || debug!("UPDATE"));
    app
}

#[test]
fn simple() {
    let mut app = setup();
    app.add_service(TEST_SERVICE_SPEC);
    app.update();
    let world = app.world_mut();
    let s = world.resource_mut::<TestService>();
    assert_eq!(s.state, ServiceState::Uninitialized)
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.add_service(TEST_SERVICE_SPEC.is_startup(true).on_init(|| {
        info!("In hook");
        Err(TestErr::A)
    }));
    app.update();
    let world = app.world_mut();
    let service = world.resource_mut::<TestService>();
    assert_eq!(service.state, ServiceState::Failed(TestErr::A));
}

#[test]
fn manual_init() {
    let mut app = setup();
    app.add_service(TEST_SERVICE_SPEC);
    app.update();
    app.world_mut().commands().init_service(TEST_SERVICE);
    app.update();
    let world = app.world_mut();
    let service = world.resource_mut::<TestService>();
    assert_eq!(service.state, ServiceState::Enabled);
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
    let spec = TEST_SERVICE_SPEC
        .is_startup(true)
        .on_init(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("init");
            hooks_ran.init = true;
            Ok(true)
        })
        .on_enable(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("enable");
            hooks_ran.enable = true;
            Ok(())
        })
        .on_disable(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("disable");
            hooks_ran.disable = true;
            Err(TestErr::A)
        })
        .on_failure(|_err: In<TestErr>, mut hooks_ran: ResMut<TestHooks>| {
            debug!("failure");
            hooks_ran.fail = true;
        });
    println!("{spec:#?}");
    app.add_service(spec);
    app.update();
    let world = app.world_mut();
    world.resource_scope::<TestService, ()>(|world, mut r| {
        let _ = r.on_disable(world);
    });
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

// TODO: Failing. Need to switch from commands to events.
#[test]
fn hooks_within_hooks() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    let spec = TEST_SERVICE_SPEC
        .is_startup(true)
        .on_enable(|mut commands: Commands| {
            debug!("enable");
            commands.disable_service(TEST_SERVICE);
            Ok(())
        })
        .on_disable(|mut commands: Commands| {
            debug!("disable");
            commands.enable_service(TEST_SERVICE);
            Ok(())
        });
    app.add_service(spec);
    app.update();
    let service = app.world().resource::<TestService>();
    assert!(matches!(service.state, ServiceState::Enabled));
    app.update();
    let service = app.world().resource::<TestService>();
    assert!(matches!(service.state, ServiceState::Disabled));
    app.update();
    let service = app.world().resource::<TestService>();
    assert!(matches!(service.state, ServiceState::Enabled));
    app.update();
    let service = app.world().resource::<TestService>();
    assert!(matches!(service.state, ServiceState::Disabled));
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
