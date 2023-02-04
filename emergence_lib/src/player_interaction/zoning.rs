//! Selecting structures to place, and then setting tiles as those structures.

use bevy::prelude::*;

use crate::structures::StructureId;

use super::InteractionSystem;

/// Logic and resources for structure selection and placement.
pub struct ZoningPlugin;

impl Plugin for ZoningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedStructure>()
            .add_system(display_selected_structure.after(InteractionSystem::SelectStructure));
    }
}

/// Tracks which structure the player has selected, if any
#[derive(Resource, Default, Debug)]
pub struct SelectedStructure {
    /// Which structure is selected
    pub maybe_structure: Option<StructureId>,
}

/// Shows which structure the player has selected.
fn display_selected_structure(selected_structure: Res<SelectedStructure>) {
    if selected_structure.is_changed() {
        let selected_structure = &selected_structure.maybe_structure;
        info!("Currently selected: {selected_structure:?}");
    }
}
