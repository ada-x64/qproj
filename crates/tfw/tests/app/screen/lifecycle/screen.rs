use crate::prelude::*;

/// This struct can be used to dynamically change the screen's behavior.
#[derive(PartialEq, Eq, Clone, Debug, Hash, Reflect, Default, Resource)]
pub struct LifecycleSettings;

macro_rules! gen_fns {
    ($($name:ident),*) => {
        $(
            fn $name(mut r: ResMut<LifecycleStatus>) {
                r.$name = true;
            }
        )*
        #[derive(Resource, Debug, Default)]
        pub struct LifecycleStatus {
            $(pub $name:bool,)*
        }
        impl LifecycleStatus {
            pub fn ok(&self) -> bool {
                $(self.$name) &&*
            }
        }
    }
}
gen_fns!(
    loading,
    update,
    fixed_update,
    unloading,
    load,
    ready,
    unload,
    unloaded
);

/// The main [Screen] implementation.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct LifecycleScreen;
impl Screen for LifecycleScreen {
    type SETTINGS = LifecycleSettings;
}

macro_rules! progress_by {
    ($name:ident) => {
        |mut data: ScreenDataMut<LifecycleScreen>| {
            data.$name();
        }
    };
}

pub fn plugin(app: &mut App) {
    ScreenScopeBuilder::<LifecycleScreen>::new(app)
        .add_systems(
            ScreenSchedule::Loading,
            (loading, progress_by!(finish_loading)),
        )
        // progress to unload by loading in EmptyScreen
        .add_systems(
            ScreenSchedule::Update,
            (update, |r: Res<LifecycleStatus>, mut commands: Commands| {
                if r.fixed_update && r.update {
                    commands.trigger(switch_to_screen::<EmptyScreen>());
                }
            }),
        )
        .add_systems(ScreenSchedule::FixedUpdate, fixed_update)
        .add_systems(
            ScreenSchedule::Unloading,
            (unloading, progress_by!(finish_unloading)),
        )
        .on_load(load)
        .on_ready(ready)
        .on_unload(unload)
        .on_unloaded(unloaded)
        .build();
    app.init_resource::<LifecycleStatus>();
}
