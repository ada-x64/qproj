use crate::prelude::*;
use q_screens_derive::Screen;

#[derive(Screen, Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct EmptyScreen;
impl EmptyScreen {
    pub fn plugin(app: &mut App) {
        ScreenScopeBuilder::<EmptyScreen>::new(app).build();
    }
}
