use bevy::{log::LogPlugin, prelude::*};
use q_service::prelude::*;

#[derive(ServiceError, Debug, thiserror::Error, Clone, Copy, PartialEq)]
enum TestErr {
    #[error("A")]
    A,
}

service!(TestService, (), TestErr);
service!(TestService2, (), TestErr);
service!(TestService3, (), TestErr);

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
#[should_panic]
fn deps_fail_on_cycle() {
    let mut app = setup();
    app.add_service(
        TEST_SERVICE_SPEC
            .is_startup(true)
            .with_deps(vec![TEST_SERVICE2]),
    )
    .add_service(
        TEST_SERVICE2_SPEC
            .is_startup(true)
            .with_deps(vec![TEST_SERVICE]),
    );
    app.update();
}

#[test]
fn dependency_initialization() {
    let mut app = setup();
    app.add_service(
        TEST_SERVICE_SPEC
            .is_startup(true)
            .with_deps(vec![TEST_SERVICE2]),
    );
    app.add_service(TEST_SERVICE2_SPEC.with_deps(vec![TEST_SERVICE3]));
    app.add_service(TEST_SERVICE3_SPEC);

    // TODO: Should un-added services get automatically picked up and
    // initialized?

    app.update();
    let state = &app.world().resource::<TestService>().state;
    assert_eq!(state, &ServiceState::Enabled);
    let state = &app.world().resource::<TestService2>().state;
    assert_eq!(state, &ServiceState::Enabled);
    let state = &app.world().resource::<TestService3>().state;
    assert_eq!(state, &ServiceState::Enabled);
}

#[test]
fn failure_propogation() {
    let mut app = setup();
    app.add_service(
        TEST_SERVICE_SPEC
            .is_startup(true)
            .with_deps(vec![TEST_SERVICE2]),
    );
    app.add_service(TEST_SERVICE2_SPEC.with_deps(vec![TEST_SERVICE3]));
    app.add_service(TEST_SERVICE3_SPEC.on_init(|| Err(TestErr::A)));
    app.update();
    let err_str = TestErr::A.to_string();
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService>| {
            let state = &s.state;
            debug!("Checking state {state:#?}");
            match state {
                ServiceState::Failed(ServiceErrorKind::Dependency(a, b, e)) => {
                    assert_eq!(a, &TEST_SERVICE.to_string());
                    assert_eq!(b, &TEST_SERVICE2.to_string());
                    assert!(e.contains(&err_str));
                }
                _ => {
                    panic!()
                }
            }
        });
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService2>| {
            let state = &s.state;
            match state {
                ServiceState::Failed(ServiceErrorKind::Dependency(a, b, e)) => {
                    assert_eq!(a, &TEST_SERVICE2.to_string());
                    assert_eq!(b, &TEST_SERVICE3.to_string());
                    assert!(e.contains(&err_str));
                }
                _ => {
                    panic!()
                }
            }
        });
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService3>| {
            let state = &s.state;
            match state {
                ServiceState::Failed(ServiceErrorKind::Own(e)) => {
                    assert!(matches!(e, TestErr::A));
                }
                _ => {
                    panic!()
                }
            }
        });
}
