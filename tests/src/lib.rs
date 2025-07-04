use std::sync::Arc;

use bevy::{
    app::ScheduleRunnerPlugin,
    core_pipeline::CorePipelinePlugin,
    dev_tools::ci_testing::CiTestingPlugin,
    diagnostic::FrameCountPlugin,
    pbr::PbrPlugin,
    prelude::*,
    render::{
        RenderPlugin,
        renderer::{
            RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance,
            RenderQueue, WgpuWrapper,
        },
        settings::{RenderCreation, RenderResources},
    },
    tasks::block_on,
    time::TimePlugin,
    window::ExitCondition,
};
use wgpu::{DeviceDescriptor, InstanceDescriptor, RequestAdapterOptions};

#[cfg(test)]
pub mod worldgen;

#[allow(clippy::test_attr_in_doctest)]
/// Runs a headless instance. In order to succesfully exit the app, make sure
/// you send an AppExit event. This can be accomplished with an
/// [`EventWriter`] or with [`World::send_event`]. This does _not_
/// work with [`Commands::trigger`].
///
/// Basic usage:
/// ```rust
/// use bevy::prelude::*;
/// use integration::Runner;
///
/// fn example_test() -> AppExit {
///    Runner::new(|app| {
///        app.add_plugins(MyPlugin);
///        app.add_systems(Update, |mut events: EventWriter<AppExit>| {
///            events.write(AppExit::Success);
///        });
///        app.run()
///    })
///    .run()
/// }
///
/// struct MyPlugin;
/// impl Plugin for MyPlugin {
///    fn build(&self, _: &mut App) {
///        //...
///    }
/// }
/// ```
pub struct Runner {
    test_fn: Box<dyn Fn(&mut App) -> AppExit>,
    timeout: f32,
}
impl Runner {
    /// Initializes the test runner.
    pub fn new(test: impl Fn(&mut App) -> AppExit + 'static) -> Self {
        Self {
            test_fn: Box::new(test),
            timeout: 3.,
        }
    }
    /// Sets timeout in seconds.
    pub fn with_timeout(&mut self, timeout: f32) -> &mut Self {
        self.timeout = timeout;
        self
    }
    /// Runs the test.
    pub fn run(&mut self) -> AppExit {
        block_on(async {
            let mut app = App::new();
            app.add_plugins((
                TaskPoolPlugin::default(),
                FrameCountPlugin,
                TimePlugin,
                ScheduleRunnerPlugin::default(),
                CiTestingPlugin,
                #[cfg(feature = "log")]
                bevy::log::LogPlugin::default(),
            ));
            debug!("Initializing headless app.");

            let instance = wgpu::Instance::new(&InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&RequestAdapterOptions::default())
                .await
                .expect("Failed to get wgpu adapter.");
            let (device, queue) = adapter
                .request_device(&DeviceDescriptor::default(), None)
                .await
                .expect("Failed to get wpu device.");
            let adapter_info = adapter.get_info();

            app.add_plugins((
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "Headless test".into(),
                        resizable: true,
                        focused: true,
                        visible: false,
                        desired_maximum_frame_latency: None,
                        ..Default::default()
                    }),
                    close_when_requested: true,
                    exit_condition: ExitCondition::OnPrimaryClosed,
                },
                AssetPlugin::default(),
                RenderPlugin {
                    render_creation: RenderCreation::Manual(RenderResources(
                        RenderDevice::new(WgpuWrapper::new(device)),
                        RenderQueue(Arc::new(WgpuWrapper::new(queue))),
                        RenderAdapterInfo(WgpuWrapper::new(adapter_info)),
                        RenderAdapter(Arc::new(WgpuWrapper::new(adapter))),
                        RenderInstance(Arc::new(WgpuWrapper::new(instance))),
                    )),
                    ..Default::default()
                },
                ImagePlugin::default(),
                CorePipelinePlugin,
                PbrPlugin {
                    prepass_enabled: false,
                    add_default_deferred_lighting_plugin: false,
                    use_gpu_instance_buffer_builder: false,
                    ..Default::default()
                },
            ));

            let timeout = self.timeout;
            app.add_systems(
                Update,
                move |time: Res<Time<Real>>,
                      mut events: EventWriter<AppExit>| {
                    let elapsed = time.elapsed_secs();
                    if elapsed > timeout {
                        error!("Timeout after {elapsed}s");
                        events.write(AppExit::error());
                    }
                },
            );

            debug!("Running internal function.");
            self.test_fn.as_mut()(&mut app)
        })
    }
}

#[test]
fn timeout() {
    let exit = Runner::new(|app| app.run()).with_timeout(0.5).run();
    assert_eq!(exit, AppExit::error());
}
