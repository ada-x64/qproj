pub use bevy::prelude::*;
use bevy::{log::LogPlugin, platform::collections::HashMap};
use q_service::*;

/// ServiceNames is auto impled
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum TestServices {
    One,
    Two,
    Three,
}

type TestService = Service<TestServices>;
type TestServiceDependencies = ServiceDependencies<TestServices>;

#[test]
fn basic() {
    // set up specs
    let spec1 = ServiceSpec::new(TestServices::One);
    let spec2 = ServiceSpec::new(TestServices::Two).with_startup(true);
    let spec3 = ServiceSpec::new(TestServices::Three)
        .with_deps(vec![TestServices::One, TestServices::Two]);

    // set up app
    let mut app = App::new();
    app.add_plugins((
        LogPlugin::default(),
        ServicePlugin::<TestServices> {
            services: vec![spec1, spec2, spec3],
        },
    ));
    app.update();

    // check world structure
    let world = app.world_mut();
    let mut manager = world.query::<(&ServiceManager, &Children)>();
    let (_manager, children) = manager.single(world).unwrap();

    children.iter().for_each(|child| {
        let service = world.get::<TestService>(child).unwrap();
        let state = world.get::<ServiceState>(child).unwrap();
        if **service == TestServices::Two {
            assert!(matches!(**state, ServiceStatus::Enabled));
        } else {
            assert!(matches!(**state, ServiceStatus::Uinitialized));
        }
    });

    let deps = world.resource::<TestServiceDependencies>();
    let expected = HashMap::from_iter(vec![
        (TestServices::One, vec![]),
        (TestServices::Two, vec![]),
        (
            TestServices::Three,
            vec![TestServices::One, TestServices::Two],
        ),
    ]);
    assert_eq!(deps.0, expected);
}

#[test]
fn init_failure() {
    // set up specs
    let spec1 = ServiceSpec::new(TestServices::One)
        .with_startup(true)
        .on_init(|_| Err("uh oh".into()));

    // set up app
    let mut app = App::new();
    app.add_plugins((
        LogPlugin::default(),
        ServicePlugin::<TestServices> {
            services: vec![spec1],
        },
    ));
    app.update();

    let world = app.world_mut();
    let mut q = world.query::<(&TestService, &ServiceState)>();
    let (_, state) = q
        .iter(world)
        .find(|(s, _)| ***s == TestServices::One)
        .unwrap();
    assert!(matches!(**state, ServiceStatus::Failed(_)));
}
