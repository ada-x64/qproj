use crate::prelude::*;

macro_rules! gen_fns {
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

#[test]
fn lifecycle() {
    let mut app = get_test_app::<LifecycleScreen>();
    gen_fns!(app, on_screen_load, on_screen_ready, on_screen_unloaded);
    app.init_resource::<TestRes>();
    app.add_systems(
        on_screen_ready::<EmptyScreen>(),
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
