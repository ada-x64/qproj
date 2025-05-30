//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures_lite::future},
};

#[derive(Component)]
pub struct TaskComponent(pub Task<CommandQueue>);

fn poll_tasks(mut commands: Commands, tasks: Query<&mut TaskComponent>) {
    for mut task in tasks {
        if let Some(mut q) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut q);
        }
    }
}

// WWID?
// Generalizing the pattern found in UiState::save_scene
pub fn spawn_io_task(world: &mut World, task: impl AsyncFn(&mut CommandQueue)) {
    let mut entity = world.spawn_empty();
    let id = entity.id();
    let task = IoTaskPool::get().spawn(async move {
        let mut q = CommandQueue::default();
        task(&mut q).await;
        q.push(|world: &mut World| {
            world.despawn(id);
        });
        q
    });
    entity.insert(TaskComponent(task));
}

pub struct TaskPlugin;
impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, poll_tasks);
    }
}
