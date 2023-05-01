//! Abilities spend intent, modifying the behavior of allied organisms in an area.

use crate as emergence_lib;
use crate::asset_management::manifest::Id;
use crate::organisms::energy::VigorModifier;
use crate::signals::{Emitter, ManageSignals, SignalModifier, SignalStrength, SignalType};
use crate::simulation::geometry::{MapGeometry, TilePos};
use crate::simulation::SimulationSet;
use crate::terrain::terrain_manifest::Terrain;
use crate::terrain::TerrainEmitters;

use super::clipboard::Tool;
use super::picking::CursorPos;
use super::selection::CurrentSelection;
use super::{InteractionSystem, PlayerAction, PlayerModifiesWorld};
use bevy::prelude::*;
use bevy::utils::HashSet;
use derive_more::Display;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use emergence_macros::IterableEnum;
use leafwing_abilities::pool::MaxPoolLessThanZero;
use leafwing_abilities::prelude::Pool;
use leafwing_input_manager::prelude::ActionState;
use std::ops::{Div, Mul};

/// Controls, interface and effects of intent-spending abilities.
pub(super) struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                regenerate_intent,
                use_ability
                    // Run after the terrain emitters, so that the our Lure / Repel signals are not overwritten.
                    .after(TerrainEmitters)
                    // Run before the signal manager, so that the signals are updated before they are used.
                    .before(ManageSignals)
                    // Run after we select tiles, so that we can use abilities on the selected tiles.
                    .after(InteractionSystem::SelectTiles),
            )
                .chain()
                .in_set(SimulationSet)
                .in_set(PlayerModifiesWorld)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .init_resource::<IntentPool>();
    }
}

/// The different intent-spending "abilities" that the hive mind can use.
///
/// Note that the order of these variants is important,
/// as it determines the order of the abilities in the UI.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, IterableEnum, Display)]
pub enum IntentAbility {
    /// Increases the working speed and maintenance costs of structures.
    Flourish,
    /// Gather allied units.
    Lure,
    /// Increase the signal strength of emitters.
    Amplify,
    /// Decreases the working speed and maintenance costs of structures.
    Fallow,
    /// Decrease the signal strength of emitters.
    Dampen,
    /// Repel allied units.
    Repel,
}

impl IntentAbility {
    /// The cost of each ability per second they are used.
    ///
    /// This cost is only paid for entities that this ability affects.
    pub(crate) fn cost(&self) -> Intent {
        Intent(match self {
            IntentAbility::Lure => 10.,
            IntentAbility::Repel => 10.,
            IntentAbility::Flourish => 5.,
            IntentAbility::Fallow => 5.,
            IntentAbility::Amplify => 2.,
            IntentAbility::Dampen => 2.,
        })
    }
}

/// Uses abilities when pressed at the cursor's location.
///
/// Note: [`Intent`] is spent when these abilities take effect, not when they are used.
/// This allows the player to use abilities over a broad area, and only pay for the tiles that actually matter.
fn use_ability(
    current_selection: Res<CurrentSelection>,
    cursor_pos: Res<CursorPos>,
    tool: Res<Tool>,
    player_actions: Res<ActionState<PlayerAction>>,
    mut terrain_query: Query<
        (&mut VigorModifier, &mut SignalModifier, &mut Emitter),
        With<Id<Terrain>>,
    >,
    map_geometry: Res<MapGeometry>,
    mut previously_modified_tiles: Local<HashSet<TilePos>>,
) {
    let relevant_tiles = current_selection.relevant_tiles(&cursor_pos);
    if relevant_tiles.is_empty() {
        return;
    }
    let Tool::Ability(ability) = *tool else { return };

    // Clear all previously modified tiles
    // By caching which tiles we touched, we can avoid iterating over the entire map every frame
    for &tile_pos in previously_modified_tiles.iter() {
        let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
        let (mut vigor_modifier, mut signal_modifier, _emitter) =
            terrain_query.get_mut(terrain_entity).unwrap();

        *vigor_modifier = VigorModifier::None;
        *signal_modifier = SignalModifier::None;
    }
    // Clear, rather than re-allocate
    previously_modified_tiles.clear();

    if player_actions.pressed(PlayerAction::UseTool) {
        for &tile_pos in relevant_tiles.selection() {
            let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
            let (mut vigor_modifier, mut signal_modifier, mut emitter) =
                terrain_query.get_mut(terrain_entity).unwrap();
            previously_modified_tiles.insert(tile_pos);

            match ability {
                IntentAbility::Lure => emitter
                    .signals
                    .push((SignalType::Lure, SignalStrength::new(50.))),
                IntentAbility::Repel => emitter
                    .signals
                    .push((SignalType::Repel, SignalStrength::new(50.))),
                IntentAbility::Flourish => *vigor_modifier = VigorModifier::Flourish,
                IntentAbility::Fallow => *vigor_modifier = VigorModifier::Fallow,
                IntentAbility::Amplify => *signal_modifier = SignalModifier::Amplify,
                IntentAbility::Dampen => *signal_modifier = SignalModifier::Dampen,
            }
        }
    }
}

/// The amount of Intent available to the player.
/// If they spend it all, they can no longer act.
///
/// This is stored as a single global resource.
#[derive(Debug, Clone, PartialEq, Resource)]
pub(crate) struct IntentPool {
    /// The current amount of available intent.
    current: Intent,
    /// The maximum intent that can be stored.
    max: Intent,
    /// The amount of intent regenerated per second.
    regen_per_second: Intent,
}

/// The maximum amount of intent that can be stored at once
const MAX_INTENT: Intent = Intent(100.);
/// Amount of intent that is regenerated each second
const INTENT_REGEN: Intent = Intent(10.);

impl Default for IntentPool {
    fn default() -> Self {
        IntentPool {
            current: MAX_INTENT,
            max: MAX_INTENT,
            regen_per_second: INTENT_REGEN,
        }
    }
}

/// A quantity of Intent, used to modify an [`IntentPool`].
///
/// This is used to measure the amount of Intent that must be spent to perform various actions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Add, Sub, AddAssign, SubAssign)]
pub(crate) struct Intent(pub(crate) f32);

impl Mul<f32> for Intent {
    type Output = Intent;

    fn mul(self, rhs: f32) -> Intent {
        Intent(self.0 * rhs)
    }
}

impl Div<f32> for Intent {
    type Output = Intent;

    fn div(self, rhs: f32) -> Intent {
        Intent(self.0 / rhs)
    }
}

impl Pool for IntentPool {
    type Quantity = Intent;
    const ZERO: Intent = Intent(0.);

    fn new(current: Self::Quantity, max: Self::Quantity, regen_per_second: Self::Quantity) -> Self {
        IntentPool {
            current,
            max,
            regen_per_second,
        }
    }

    fn current(&self) -> Self::Quantity {
        self.current
    }

    fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
        let actual_value = Intent(new_quantity.0.clamp(0., self.max.0));
        self.current = actual_value;
        self.current
    }

    fn max(&self) -> Self::Quantity {
        self.max
    }

    fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanZero> {
        if new_max < Self::ZERO {
            Err(MaxPoolLessThanZero)
        } else {
            self.max = new_max;
            self.set_current(self.current);
            Ok(())
        }
    }

    fn regen_per_second(&self) -> Self::Quantity {
        self.regen_per_second
    }

    fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity) {
        self.regen_per_second = new_regen_per_second;
    }
}

/// Regenerates the [`Intent`] of the hive mind.
///
/// Note that we cannot use the built-in system for this, as our pool is stored somewhat unusually as a resource.
fn regenerate_intent(mut intent_pool: ResMut<IntentPool>, time: Res<FixedTime>) {
    if intent_pool.current() != intent_pool.max() {
        intent_pool.regenerate(time.period);
    }
}
