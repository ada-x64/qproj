use crate::prelude::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Reflect, Default, Resource)]
struct LoadStrategyScreenSettings {
    pub initial_value: u32,
    pub unload_value: u32,
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
struct LoadStrategyScreen;
impl Screen for LoadStrategyScreen {
    type SETTINGS = LoadStrategyScreenSettings;
}
#[derive(Resource, Debug, Default)]
struct FinalValue(u32);

impl LoadStrategyScreen {
    fn load(mut count: Local<u32>, mut data: ScreenDataMut<Self>) {
        *count += 1;
        if *count == 100 {
            data.finish_loading();
            info!("Finished loading!");
        }
    }

    fn update(
        mut count: Local<u32>,
        data: ScreenDataRef<Self>,
        mut commands: Commands,
        mut value: ResMut<FinalValue>,
    ) {
        *count += 1;
        if data.data().state() == ScreenState::Ready {
            value.0 = *count;
            commands.trigger(switch_to_screen::<EmptyScreen>());
        }
    }

    fn unloaded(data: ScreenDataRef<Self>, value: Res<FinalValue>, mut commands: Commands) {
        let expected_value = match data.data().load_strategy() {
            LoadStrategy::Blocking => 100,
            LoadStrategy::Nonblocking => 1,
        };
        info!("Got {}, expected {}", value.0, expected_value);
        if value.0 != expected_value {
            commands.write_message(AppExit::error());
        } else {
            commands.write_message(AppExit::Success);
        }
    }

    fn plugin(app: &mut App, load_strategy: LoadStrategy) {
        ScreenScopeBuilder::<LoadStrategyScreen>::new(app)
            .with_load_strategy(load_strategy)
            .add_systems(ScreenSchedule::Update, Self::update)
            .add_systems(ScreenSchedule::Loading, Self::load)
            .on_unloaded(Self::unloaded)
            .build();
        app.init_resource::<FinalValue>();
    }

    pub fn plugin_blocking(app: &mut App) {
        Self::plugin(app, LoadStrategy::Blocking);
    }

    pub fn plugin_nonblocking(app: &mut App) {
        Self::plugin(app, LoadStrategy::Nonblocking);
    }
}

type Scr = LoadStrategyScreen;

#[test]
fn blocking() {
    let mut app = get_test_app::<Scr>();
    app.add_plugins(Scr::plugin_blocking);
    assert!(app.run().is_success());
}

#[test]
fn nonblocking() {
    let mut app = get_test_app::<Scr>();
    app.add_plugins(Scr::plugin_nonblocking);
    assert!(app.run().is_success());
}
