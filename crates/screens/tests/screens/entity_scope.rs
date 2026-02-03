use crate::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct ScopedEntitiesScreen;
impl Screen for ScopedEntitiesScreen {
    fn builder(builder: ScreenScopeBuilder<Self>) -> ScreenScopeBuilder<Self> {
        builder.add_systems(ScreenSchedule::OnReady, |mut commands: Commands| {
            // todo: spawn screen-scoped entities
        })
    }
}

type Scr = ScopedEntitiesScreen;
// #[test]
// fn test_scoped_entities() {
//     let mut app = get_test_app::<Scr>();
//     app.register_screen::<ScopedEntitiesScreen>();
//     app.add_step(0, || {});
// }
