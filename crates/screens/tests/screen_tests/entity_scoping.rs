use q_test_harness::prelude::*;

use crate::prelude::*;

// #[test]
// fn screen_transitions() {
//     let mut app = get_test_app::<EmptyScreen>();
//     app.add_step(
//         0,
//         |mut step: ResMut<NextState<Step>>,
//          mut settings: ResMut<NamedEntityScreenSettings>,
//          mut commands: Commands| {
//             settings.entity_name = "1".into();
//             commands.trigger(SwitchToScreen::<NamedEntityScreen>::default());
//             step.set(Step(1));
//         },
//     )
//     .add_step(
//         1,
//         |mut step: ResMut<NextState<Step>>,
//          mut commands: Commands,
//          q: Query<(Entity, &Name)>,
//          data: ScreenDataRef<NamedEntityScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.log_hierarchy();
//             let found = q.iter().any(|(_, ename)| (**ename).eq("1"));
//             if !found {
//                 error!("Could not find entity with name '1'");
//                 commands.write_message(AppExit::error());
//                 return;
//             }
//             commands.trigger(SwitchToScreen::<EmptyScreen>::default());
//             step.set(Step(2));
//         },
//     )
//     .add_step(
//         2,
//         |mut step: ResMut<NextState<Step>>,
//          mut settings: ResMut<NamedEntityScreenSettings>,
//          mut commands: Commands,
//          q: Query<(Entity, &Name)>,
//          data: ScreenDataRef<EmptyScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.log_hierarchy();
//             let found = q.iter().any(|(_, ename)| (**ename).eq("1"));
//             if found {
//                 error!("Could not find entity with name '1'");
//                 commands.write_message(AppExit::error());
//                 return;
//             }
//             settings.entity_name = "2".into();
//             commands.trigger(SwitchToScreen::<NamedEntityScreen>::default());
//             step.set(Step(3));
//         },
//     )
//     .add_step(
//         3,
//         |mut commands: Commands,
//          q: Query<(Entity, &Name)>,
//          data: ScreenDataRef<NamedEntityScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.log_hierarchy();
//             let found = q.iter().any(|(_, ename)| (**ename).eq("2"));
//             if !found {
//                 error!("Could not find entity with name '2'");
//                 commands.write_message(AppExit::error());
//                 return;
//             }
//             commands.write_message(AppExit::Success);
//         },
//     );

//     assert!(app.run().is_success());
// }

// #[test]
// fn persistent_entities() {
//     let mut app = get_test_app::<NamedEntityScreen>();
//     app.insert_resource(NamedEntityScreenSettings {
//         entity_name: "1".into(),
//     });
//     app.add_step(
//         0,
//         |mut commands: Commands, data: ScreenDataRef<NamedEntityScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.spawn((
//                 Name::new("Persistent"),
//                 bevy::app::Propagate(Persistent),
//                 children![(
//                     Name::new("Child"),
//                     children![(
//                         Name::new("Grandchild"),
//                         bevy::app::PropagateStop::<Persistent>::default(),
//                         children![Name::new("Great Grandchild")]
//                     )]
//                 )],
//             ));
//             commands.log_hierarchy();
//             commands.find_entity("Persistent");
//             commands.find_entity_filtered::<With<Persistent>>("Child");
//             commands.find_entity_filtered::<Without<Persistent>>("Grandchild");
//             commands.find_entity_filtered::<Without<Persistent>>("Great Grandchild");
//             commands.find_entity_filtered::<Without<Persistent>>("1");
//             commands.trigger(SwitchToScreen::<EmptyScreen>::default());
//         },
//     )
//     .add_step(
//         1,
//         |mut commands: Commands, data: ScreenDataRef<EmptyScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.log_hierarchy();
//             commands.find_entity("Persistent");
//             commands.find_entity("Child");
//             commands.find_no_entity("Grandchild");
//             commands.find_no_entity("Great Grandchild");
//             commands.find_no_entity("1");
//         },
//     );
// }

// #[derive(Component)]
// struct Empty;

// /// Child observers should be removed, but top-level observers should remain.
// /// NOTE: Child observers _probably_ shouldn't exist. This functionality has been
// /// replaced with [EntityEvent]. But, this checks the scoping query functions as intended.
// #[test]
// fn observer_cleanup() {
//     let mut app = get_test_app::<EmptyScreen>();
//     app.add_step(
//         0,
//         |mut commands: Commands,
//          data: ScreenDataRef<EmptyScreen>,
//          mut step: ResMut<NextState<Step>>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.spawn((
//                 Name::new("Parent"),
//                 children![(
//                     Name::new("Child"),
//                     Observer::new(|trigger: On<SwitchToScreen<NamedEntityScreen>>| {
//                         info!("Observer ({:?})", *trigger)
//                     }),
//                     Empty
//                 )],
//             ));
//             commands.find_entity("Parent");
//             commands.find_entity("Child");
//             commands.trigger(SwitchToScreen::<NamedEntityScreen>::default());
//             step.set(Step(1));
//         },
//     )
//     .add_step(
//         1,
//         |mut commands: Commands, data: ScreenDataRef<NamedEntityScreen>| {
//             if !data.data().state.is_ready() {
//                 return;
//             }
//             commands.find_no_entity("Parent");
//             commands.find_no_entity("Child");
//         },
//     );
// }
