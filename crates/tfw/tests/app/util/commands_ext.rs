use bevy::ecs::query::QueryFilter;
use bevy_inspector_egui::bevy_inspector::{
    guess_entity_name,
    hierarchy::{Hierarchy, SelectedEntities},
};

use crate::prelude::*;

pub trait CommandsExt {
    fn log_hierarchy(&mut self);
    fn find_entity(&mut self, name: impl ToString);
    fn find_no_entity(&mut self, name: impl ToString);
    fn find_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString);
    fn find_no_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString);
    fn find_entity_with<C: Component + PartialEq>(
        &mut self,
        name: impl ToString,
        value: C,
        invert: bool,
    );
}
impl<'w, 's> CommandsExt for Commands<'w, 's> {
    fn log_hierarchy(&mut self) {
        self.run_system_cached(|world: &mut World| {
            let h = Hierarchy {
                world,
                selected: &mut SelectedEntities::default(),
                context_menu: None,
                shortcircuit_entity: None,
                extra_state: &mut (),
            };

            let mut root_query = h.world.query_filtered::<Entity, Without<ChildOf>>();
            let entities: Vec<_> = root_query.iter(h.world).collect();
            let mut output = String::new();
            log_hierarchy_inner(world, &mut output, entities, 0);
            info!("{output}")
        });
    }
    fn find_entity(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity, (name.to_string(), false));
    }
    fn find_no_entity(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity, (name.to_string(), true));
    }
    fn find_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity_filtered::<F>, (name.to_string(), false));
    }
    fn find_no_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity_filtered::<F>, (name.to_string(), true));
    }
    fn find_entity_with<C: Component + PartialEq>(
        &mut self,
        name: impl ToString,
        value: C,
        invert: bool,
    ) {
        self.run_system_cached_with(find_entity_with, (name.to_string(), invert, value));
    }
}

fn log_hierarchy_inner(world: &mut World, output: &mut String, entities: Vec<Entity>, depth: u32) {
    for &entity in &entities {
        let entity_name = guess_entity_name(world, entity);
        let mut tags = vec![];
        if world.entity(entity).get::<Persistent>().is_some() {
            tags.push("Persistent");
        }
        if world.entity(entity).get::<Observer>().is_some() {
            tags.push("Observer");
        }
        let indent = (0..depth).map(|_| "-").collect::<Vec<_>>().join("");
        #[allow(clippy::obfuscated_if_else)]
        let tags = (!tags.is_empty())
            .then(|| format!("<{}>", tags.join(", ")))
            .unwrap_or_default();

        *output = format!("{output}\n{indent}> {entity_name} {tags}");

        if let Some(children) = world.entity(entity).get::<Children>() {
            let children = children.iter().collect::<Vec<Entity>>();
            log_hierarchy_inner(world, output, children, depth + 1);
        }
    }
}

/// Searches for an entity with the given [Name] component.
/// This _will not_ show entities marked with [Internal], including Observers.
fn find_entity(input: In<(String, bool)>, q: Query<&Name>, mut commands: Commands) {
    let (name, invert) = input.0;
    let any = q.iter().any(|ename| (**ename).eq(&name));
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}

/// Searches for an entity with the given [Name] component
fn find_entity_filtered<F: QueryFilter>(
    input: In<(String, bool)>,
    q: Query<&Name, F>,
    mut commands: Commands,
) {
    let (name, invert) = input.0;
    let any = q.iter().any(|ename| (**ename).eq(&name));
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}

/// Searches for an entity with the given [Name] and component C.
fn find_entity_with<C: Component + PartialEq>(
    input: In<(String, bool, C)>,
    q: Query<(&Name, &C)>,
    mut commands: Commands,
) {
    let (name, invert, value) = input.0;
    let any = q
        .iter()
        .any(|(ename, c)| (**ename).eq(&name) && *c == value);
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}
