use crate::prelude::*;

#[derive(clap::Parser, Clone, Debug)]
#[command(name = "set")]
pub struct SetCmd {
    var: String,
    val: String,
}

fn on_msg(
    mut reader: MessageReader<CommandMsg<SetCmd>>,
    mut assets: ResMut<Assets<ConsoleEnvVars>>,
    handles: Query<&ConsoleAssetHandle<ConsoleEnvVars>>,
) {
    for msg in reader.read() {
        if let Ok(handle) = handles.get(msg.console_id)
            && let Some(env_vars) = assets.get_mut(handle.id())
        {
            env_vars.insert(msg.command.var.clone(), msg.command.val.clone());
        } else {
            warn!("Could not set env vars for console id {}", msg.console_id);
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_console_command::<SetCmd>();
    app.add_systems(PreUpdate, on_msg);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_harness;
    use q_test_harness::prelude::*;
    #[test]
    fn test_set_cmd() {
        let mut app = App::new();
        app.add_plugins(test_harness::plugin);
        let console_id = app.world_mut().spawn(Console).id();

        app.add_step(
            0,
            move |mut commands: Commands, mut step: ResMut<NextState<Step>>| {
                commands.write_message(CommandMsg::<SetCmd> {
                    console_id,
                    command: SetCmd {
                        var: "foo".into(),
                        val: "bar".into(),
                    },
                });
                step.set(Step(1));
            },
        );

        app.add_step(
            1,
            move |mut commands: Commands,
                  q: Query<&ConsoleAssetHandle<ConsoleEnvVars>>,
                  assets: Res<Assets<ConsoleEnvVars>>| {
                let ok = (|| {
                    let handle = q.get(console_id).ok()?;
                    let vars = assets.get(handle.id())?;
                    let ok = vars.get(&"foo".to_string()) == Some(&"bar".to_string());
                    ok.then_some(true)
                })();
                if ok.unwrap_or_default() {
                    commands.write_message(AppExit::Success);
                } else {
                    commands.write_message(AppExit::error());
                }
            },
        );

        assert!(app.run().is_success());
    }
}
