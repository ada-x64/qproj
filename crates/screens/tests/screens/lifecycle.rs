use crate::prelude::*;

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

macro_rules! progress_by {
    ($name:ident) => {
        |mut data: ScreenDataMut<LifecycleScreen>| {
            data.$name();
        }
    };
}

/// The main [Screen] implementation.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct LifecycleScreen;
impl Screen for LifecycleScreen {}
impl LifecycleScreen {
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
}

macro_rules! gen_test_fns {
    ($app:expr, $($name:ident),*) => {
        #[derive(Resource, Default)]
        struct TestRes {
            $($name: bool,)*
        }
        impl TestRes {
            pub fn ok(&self) -> bool {
                $(self.$name) &&*
            }
        }

        $(
            $app.add_systems(
                $name::<LifecycleScreen>(),
                |mut r: ResMut<TestRes>| {r.$name = true;}
            );
        )*
    }
}

type Scr = LifecycleScreen;
#[test]
fn lifecycle() {
    let mut app = get_test_app::<Scr>();
    app.add_plugins((Scr::plugin, EmptyScreen::plugin));
    gen_test_fns!(app, on_screen_load, on_screen_ready, on_screen_unloaded);
    app.init_resource::<TestRes>();
    app.add_systems(
        // this should _not_ trigger on initialization
        on_screen_unloaded::<Scr>(),
        |r: Res<LifecycleStatus>, r2: Res<TestRes>, mut commands: Commands| {
            let ok = r.ok() && r2.ok();
            if ok {
                info!("OK!");
                commands.write_message(AppExit::Success);
            } else {
                error!("Did not reach all expected points.");
                error!(?r);
                commands.write_message(AppExit::error());
            }
        },
    );
    assert!(app.run().is_success());
}
