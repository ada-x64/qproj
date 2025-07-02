use std::sync::Arc;

use bevy::{
    app::ScheduleRunnerPlugin,
    core_pipeline::CorePipelinePlugin,
    diagnostic::FrameCountPlugin,
    log::LogPlugin,
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

/// Runs a headless instance.
pub fn run_headless<T>(f: impl Fn(&mut App) -> T) -> T {
    block_on(async {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            FrameCountPlugin,
            TimePlugin,
            ScheduleRunnerPlugin::default(),
            // #[cfg(feature = "ci")]
            // bevy_dev_tools::ci_testing:::CiTestingPlugin,
            LogPlugin::default(),
        ));
        debug!("Initializing headless app.");

        let instance = wgpu::Instance::new(&InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await
            .unwrap();
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

        app.add_systems(
            Update,
            |time: Res<Time<Real>>, mut events: EventWriter<AppExit>| {
                // TODO: set timeout length
                let elapsed = time.elapsed_secs();
                if elapsed > 3. {
                    // panic!("Timeout!");
                    error!("Timeout after {elapsed}s");
                    events.write(AppExit::error());
                }
            },
        );

        debug!("Running internal function.");
        let res = f(&mut app);
        debug!("f returned");
        res
    })
}
