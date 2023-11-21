//! Contains [`FallibleEntityCommandExt`] and related code.

use bevy::{ecs::system::EntityCommands, prelude::*};

/// An extension trait for [`EntityCommands`] that has fallible versions of
/// the most commonly used commands.

pub trait FallibleEntityCommandExt<'w, 's, 'a> {
    /// Attempts to remove a component or bundle from the entity.
    ///
    /// Fails silently (rather than panicking) if the entity does not exist.
    ///
    /// Fallible version of [`EntityCommands::remove`].
    fn try_remove<B: Bundle>(&mut self) -> &mut EntityCommands<'w, 's, 'a>;

    /// Attempts to add a child entity to the entity.
    ///
    /// Fails silently (rather than panicking) if the entity does not exist.
    ///
    /// Fallible version of [`BuildChildren::add_child`].
    fn try_add_child(&mut self, child: Entity) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> FallibleEntityCommandExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn try_remove<B: Bundle>(&mut self) -> &mut Self {
        self.add(|entity, world: &mut World| {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.remove::<B>();
            }
        });
        self
    }

    fn try_add_child(&mut self, child: Entity) -> &mut EntityCommands<'w, 's, 'a> {
        self.add(move |entity, world: &mut World| {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.add_child(child);
            }
        });
        self
    }
}
