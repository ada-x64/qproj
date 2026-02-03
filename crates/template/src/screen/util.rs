use std::marker::PhantomData;

use crate::prelude::*;

#[derive(States, Copy, Clone, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ScreenLoadingState<S: Screen> {
    Loading,
    Ready,
    Unloaded,
    _Ghost(PhantomData<S>),
}
impl<S: Screen> ScreenLoadingState<S> {
    pub fn finish_loading(mut data: ScreenDataMut<S>) {
        data.finish_loading();
    }
    fn register(app: &mut App) {
        app.add_systems(OnEnter(Self::Ready), Self::finish_loading);
        app.add_systems(
            on_screen_unloaded::<S>(),
            |mut next: ResMut<NextState<Self>>| next.set(Self::Unloaded),
        );
        app.add_systems(
            on_screen_load::<S>(),
            |mut next: ResMut<NextState<Self>>| next.set(Self::Unloaded),
        );
    }
}

pub trait ScreenLoadingExt {
    fn register_screen_loading_state<S: Screen>(&mut self);
}

impl ScreenLoadingExt for App {
    fn register_screen_loading_state<S: Screen>(&mut self) {
        ScreenLoadingState::<S>::register(self);
    }
}
