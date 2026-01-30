use crate::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct EmptyScreen;
impl Screen for EmptyScreen {}
impl EmptyScreen {
    pub fn plugin(app: &mut App) {
        ScreenScopeBuilder::<EmptyScreen>::new(app).build();
    }
}
