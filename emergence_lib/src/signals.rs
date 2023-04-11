//! Signals are used for pathfinding and decision-making.
//!
//! By collecting information about the local environment into a slowly updated, tile-centric data structure,
//! we can scale path-finding and decisionmaking in a clear and comprehensible way.

use crate as emergence_lib;
use crate::construction::ghosts::WorkplaceId;
use crate::crafting::item_tags::ItemKind;
use crate::items::item_manifest::ItemManifest;
use crate::player_interaction::abilities::{Intent, IntentAbility, IntentPool};
use crate::structures::structure_manifest::{Structure, StructureManifest};
use crate::terrain::terrain_manifest::TerrainManifest;
use crate::units::actions::{DeliveryMode, Purpose};
use crate::units::unit_manifest::{Unit, UnitManifest};
use bevy::{prelude::*, utils::HashMap};
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use derive_more::Display;
use emergence_macros::IterableEnum;
use itertools::Itertools;
use leafwing_abilities::prelude::Pool;
use rand::seq::SliceRandom;
use std::ops::MulAssign;

use crate::asset_management::manifest::Id;
use crate::simulation::geometry::{Facing, MapGeometry, TilePos};
use crate::simulation::SimulationSet;
use crate::units::goals::Goal;

/// The fraction of signals in each cell that will move to each of 6 neighbors each frame.
///
/// Higher values will result in more spread out signals.
///
/// If no neighbor exists, total diffusion will be reduced correspondingly.
/// As a result, this value *must* be below 1/6,
/// and probably should be below 1/7 to avoid weirdness.
pub const DIFFUSION_FRACTION: f32 = 0.1;

/// The resources and systems need to work with signals
pub(crate) struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Signals>().add_systems(
            (emit_signals, diffuse_signals, degrade_signals)
                .chain()
                .in_set(ManageSignals)
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// A public system set for the signals plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ManageSignals;

/// The central resource that tracks all signals.
#[derive(Resource, Debug, Default)]
pub struct Signals {
    /// The spatialized map for each signal
    maps: HashMap<SignalType, SignalMap>,
}

impl Signals {
    /// Returns the signal strength of `signal_type` at the given `tile_pos`.
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    pub fn get(&self, signal_type: SignalType, tile_pos: TilePos) -> SignalStrength {
        match self.maps.get(&signal_type) {
            Some(map) => map.get(tile_pos),
            None => SignalStrength::ZERO,
        }
    }

    /// Returns `true` if any of the provided `signal_types` are detectable at the given `tile_pos`.
    pub fn detectable(&self, signal_types: Vec<SignalType>, tile_pos: TilePos) -> bool {
        signal_types
            .iter()
            .any(|signal_type| self.get(*signal_type, tile_pos) > SignalStrength::ZERO)
    }

    /// Adds `signal_strength` of `signal_type` at `tile_pos`.
    pub fn add_signal(
        &mut self,
        signal_type: SignalType,
        tile_pos: TilePos,
        signal_strength: SignalStrength,
    ) {
        match self.maps.get_mut(&signal_type) {
            Some(map) => map.add_signal(tile_pos, signal_strength),
            None => {
                let mut new_map = SignalMap::default();
                new_map.add_signal(tile_pos, signal_strength);
                self.maps.insert(signal_type, new_map);
            }
        }
    }

    /// Returns the complete set of signals at the given `tile_pos`.
    ///
    /// This is useful for decision-making.
    pub(crate) fn all_signals_at_position(&self, tile_pos: TilePos) -> LocalSignals {
        let mut all_signals = HashMap::new();
        for &signal_type in self.maps.keys() {
            let strength = self.get(signal_type, tile_pos);
            all_signals.insert(signal_type, strength);
        }

        LocalSignals { map: all_signals }
    }

    /// Returns the strongest goal related signal at the given `tile_pos`.
    ///
    /// This is useful for visualization.
    pub(crate) fn strongest_goal_signal_at_position(
        &self,
        tile_pos: TilePos,
    ) -> Option<SignalType> {
        let mut strongest_signal = None;
        let mut strongest_strength = SignalStrength::ZERO;

        for &signal_type in self.maps.keys() {
            if Goal::try_from(signal_type).is_ok() {
                let strength = self.get(signal_type, tile_pos);
                if strength > strongest_strength {
                    strongest_signal = Some(signal_type);
                    strongest_strength = strength;
                }
            }
        }

        strongest_signal
    }

    /// Returns the adjacent, empty tile position that contains the highest sum signal strength that can be used to meet the provided `goal`.
    ///
    /// If no suitable tile exists, [`None`] will be returned instead.
    pub(crate) fn upstream(
        &self,
        tile_pos: TilePos,
        goal: &Goal,
        item_manifest: &ItemManifest,
        map_geometry: &MapGeometry,
    ) -> Option<TilePos> {
        let mut best_choice: Option<TilePos> = None;
        let mut best_score = SignalStrength::ZERO;

        for (possible_tile, current_score) in
            self.relevant_neighboring_signals(tile_pos, goal, item_manifest, map_geometry)
        {
            if current_score > best_score {
                best_score = current_score;
                best_choice = Some(possible_tile);
            }
        }

        if let Some(best_tile_pos) = best_choice {
            if best_tile_pos == tile_pos {
                None
            } else {
                best_choice
            }
        } else {
            None
        }
    }

    /// Returns the adjacent, empty tile position that contains the lowest sum signal strength that can be used to meet the provided `goal`.
    ///
    /// If no suitable tile exists, [`None`] will be returned instead.
    pub(crate) fn downstream(
        &self,
        tile_pos: TilePos,
        goal: &Goal,
        item_manifest: &ItemManifest,
        map_geometry: &MapGeometry,
    ) -> Option<TilePos> {
        let mut best_choice: Option<TilePos> = None;
        let mut best_score = SignalStrength::INFINITY;

        for (possible_tile, current_score) in
            self.relevant_neighboring_signals(tile_pos, goal, item_manifest, map_geometry)
        {
            if current_score < best_score {
                best_score = current_score;
                best_choice = Some(possible_tile);
            }
        }

        if let Some(best_tile_pos) = best_choice {
            if best_tile_pos == tile_pos {
                None
            } else {
                best_choice
            }
        } else {
            None
        }
    }

    /// Returns the strength of goal-relevant signals in neighboring tiles.
    fn relevant_neighboring_signals(
        &self,
        tile_pos: TilePos,
        goal: &Goal,
        item_manifest: &ItemManifest,
        map_geometry: &MapGeometry,
    ) -> HashMap<TilePos, SignalStrength> {
        match goal {
            Goal::Wander { .. } => HashMap::new(),
            Goal::Fetch(item_kind)
            | Goal::Eat(item_kind)
            | Goal::Store(item_kind)
            | Goal::Deliver(item_kind)
            | Goal::Remove(item_kind) => {
                let relevant_signal_types = SignalType::item_signal_types(
                    *item_kind,
                    item_manifest,
                    goal.delivery_mode().unwrap(),
                    goal.purpose(),
                );
                let mut total_signals = HashMap::new();

                for signal_type in relevant_signal_types {
                    let signals = self.neighboring_signals(signal_type, tile_pos, map_geometry);
                    for (tile_pos, signal_strength) in signals {
                        if let Some(existing_signal_strength) = total_signals.get_mut(&tile_pos) {
                            *existing_signal_strength += signal_strength;
                        } else {
                            total_signals.insert(tile_pos, signal_strength);
                        }
                    }
                }
                total_signals
            }
            Goal::Work(structure_id) => {
                self.neighboring_signals(SignalType::Work(*structure_id), tile_pos, map_geometry)
            }
            Goal::Lure => self.neighboring_signals(SignalType::Lure, tile_pos, map_geometry),
            Goal::Repel => self.neighboring_signals(SignalType::Repel, tile_pos, map_geometry),
            Goal::Demolish(structure_id) => self.neighboring_signals(
                SignalType::Demolish(*structure_id),
                tile_pos,
                map_geometry,
            ),
        }
    }

    /// Returns the signal strength of the type `signal_type` in `tile_pos` and its 6 surrounding neighbors.
    fn neighboring_signals(
        &self,
        signal_type: SignalType,
        tile_pos: TilePos,
        map_geometry: &MapGeometry,
    ) -> HashMap<TilePos, SignalStrength> {
        let mut signal_strength_map = HashMap::with_capacity(7);

        signal_strength_map.insert(tile_pos, self.get(signal_type, tile_pos));
        for neighbor in tile_pos.passable_neighbors(map_geometry) {
            signal_strength_map.insert(neighbor, self.get(signal_type, neighbor));
        }

        signal_strength_map
    }

    /// Diffuses signals from one cell into the next
    pub fn diffuse(&mut self, map_geometry: &MapGeometry, diffusion_fraction: f32) {
        for original_map in self.maps.values_mut() {
            let num_elements = original_map.map.len();
            let size_hint = num_elements * 6;
            let mut addition_map = Vec::with_capacity(size_hint);
            let mut removal_map = Vec::with_capacity(size_hint);

            for (&occupied_tile, original_strength) in original_map
                .map
                .iter()
                .filter(|(_, signal_strength)| SignalStrength::ZERO.ne(signal_strength))
            {
                let amount_to_send_to_each_neighbor = *original_strength * diffusion_fraction;

                let mut num_neighbors = 0.0;
                for neighboring_tile in occupied_tile.passable_neighbors(map_geometry) {
                    num_neighbors += 1.0;
                    addition_map.push((neighboring_tile, amount_to_send_to_each_neighbor));
                }
                removal_map.push((
                    occupied_tile,
                    amount_to_send_to_each_neighbor * num_neighbors,
                ));
            }

            // We cannot do this in one step, as we need to avoid bizarre iteration order dependencies
            for (removal_pos, removal_strength) in removal_map.into_iter() {
                original_map.subtract_signal(removal_pos, removal_strength)
            }

            for (addition_pos, addition_strength) in addition_map.into_iter() {
                original_map.add_signal(addition_pos, addition_strength)
            }
        }
    }

    /// Returns a random signal type present in the map.
    pub(crate) fn random_signal_type(&self) -> Option<SignalType> {
        let mut rng = rand::thread_rng();
        let mut keys: Vec<&SignalType> = self.maps.keys().collect();
        keys.shuffle(&mut rng);
        keys.pop().copied()
    }
}

/// All of the signals on a single tile.
#[derive(Debug)]
pub(crate) struct LocalSignals {
    /// Internal data storage
    map: HashMap<SignalType, SignalStrength>,
}

impl LocalSignals {
    /// Returns the set of signals that might be used to pick a goal
    pub(crate) fn goal_relevant_signals(
        &self,
    ) -> impl Iterator<Item = (&SignalType, &SignalStrength)> + Clone {
        self.map
            .iter()
            .filter(|(signal_type, _signal_strength)| Goal::try_from(**signal_type).is_ok())
    }

    /// The pretty formatting for this type.
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
        terrain_manifest: &TerrainManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        let mut string = String::default();

        for signal_type in self.map.keys().sorted() {
            let signal_strength = self.map.get(signal_type).unwrap().0;

            let substring = format!(
                "{}: {signal_strength:.3}\n",
                signal_type.display(
                    item_manifest,
                    structure_manifest,
                    terrain_manifest,
                    unit_manifest
                )
            );

            string += &substring;
        }

        string
    }
}

/// Stores the [`SignalStrength`] of the given [`SignalType`] at each [`TilePos`].
#[derive(Debug, Default)]
struct SignalMap {
    /// The lookup data structure
    map: HashMap<TilePos, SignalStrength>,
}

impl SignalMap {
    /// Returns the signal strength at the given [`TilePos`].
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, tile_pos: TilePos) -> SignalStrength {
        *self.map.get(&tile_pos).unwrap_or(&SignalStrength::ZERO)
    }

    /// Returns a mutable reference to the signal strength at the given [`TilePos`].
    ///
    /// Missing values will be inserted with [`SignalStrength::ZERO`].
    fn get_mut(&mut self, tile_pos: TilePos) -> &mut SignalStrength {
        self.map.entry(tile_pos).or_insert(SignalStrength::ZERO)
    }

    /// Adds the `signal_strength` to the signal at `tile_pos`.
    fn add_signal(&mut self, tile_pos: TilePos, signal_strength: SignalStrength) {
        *self.get_mut(tile_pos) += signal_strength
    }

    /// Subtracts the `signal_strength` to the signal at `tile_pos`.
    ///
    /// The value is capped a minimum of [`SignalStrength::ZERO`].
    fn subtract_signal(&mut self, tile_pos: TilePos, signal_strength: SignalStrength) {
        *self.get_mut(tile_pos) -= signal_strength;
    }
}

/// The variety of signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalType {
    /// Take this item away from here.
    Push(ItemKind),
    /// Bring me an item of this type.
    Pull(ItemKind),
    /// Perform work at this type of structure.
    Work(WorkplaceId),
    /// Destroy a structure of this type
    Demolish(Id<Structure>),
    /// Has an item of this type, in case you were looking.
    ///
    /// The passive form of `Push`.
    Contains(ItemKind),
    /// Stores items of this type, in case you were looking.
    ///
    /// The passive form of `Pull`.
    Stores(ItemKind),
    /// Has a unit of this type.
    Unit(Id<Unit>),
    /// Draws units in.
    ///
    /// Corrresponds to [`IntentAbility::Lure`].
    Lure,
    /// Pushes units away.
    ///
    /// Corresponds to [`IntentAbility::Repel`].
    Repel,
}

impl SignalType {
    /// Returns a list of all signals that are relevant to the provided [`ItemKind`].
    ///
    /// If `delivery_mode` is [`DeliveryMode::PickUp`], this will return [`SignalType::Push`] and [`SignalType::Contains`].
    /// If `delivery_mode` is [`DeliveryMode::DropOff`], this will return [`SignalType::Pull`] and [`SignalType::Stores`].
    /// If `purpose` is [`Purpose::Intrinsic`], [`SignalType::Stores`] and [`SignalType::Contains`] are excluded.
    pub(crate) fn item_signal_types(
        item_kind: ItemKind,
        item_manifest: &ItemManifest,
        delivery_mode: DeliveryMode,
        purpose: Purpose,
    ) -> Vec<SignalType> {
        let mut signal_types = Vec::new();
        let kinds = match item_kind {
            ItemKind::Single(item_id) => item_manifest.kinds(item_id),
            ItemKind::Tag(tag) => item_manifest.kinds_with_tag(tag),
        };

        for item_kind in kinds {
            match (delivery_mode, purpose) {
                (DeliveryMode::PickUp, Purpose::Intrinsic) => {
                    signal_types.push(SignalType::Push(item_kind));
                }
                (DeliveryMode::PickUp, Purpose::Instrumental) => {
                    signal_types.push(SignalType::Push(item_kind));
                    signal_types.push(SignalType::Contains(item_kind));
                }
                (DeliveryMode::DropOff, Purpose::Intrinsic) => {
                    signal_types.push(SignalType::Pull(item_kind));
                }
                (DeliveryMode::DropOff, Purpose::Instrumental) => {
                    signal_types.push(SignalType::Pull(item_kind));
                    signal_types.push(SignalType::Stores(item_kind));
                }
            }
        }
        signal_types
    }

    /// The pretty formatting for this type
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
        terrain_manifest: &TerrainManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        match self {
            SignalType::Push(item_kind) => {
                format!("Push({})", item_manifest.name_of_kind(*item_kind))
            }
            SignalType::Pull(item_kind) => {
                format!("Pull({})", item_manifest.name_of_kind(*item_kind))
            }
            SignalType::Work(workplace_id) => {
                format!(
                    "Work({})",
                    workplace_id.name(structure_manifest, terrain_manifest)
                )
            }
            SignalType::Demolish(structure_id) => {
                format!("Demolish({})", structure_manifest.name(*structure_id))
            }
            SignalType::Contains(item_kind) => {
                format!("Contains({})", item_manifest.name_of_kind(*item_kind))
            }
            SignalType::Stores(item_kind) => {
                format!("Stores({})", item_manifest.name_of_kind(*item_kind))
            }
            SignalType::Unit(unit_id) => format!("Unit({})", unit_manifest.name(*unit_id)),
            SignalType::Lure => "Lure".to_string(),
            SignalType::Repel => "Repel".to_string(),
        }
    }
}

/// The data-less equivalent of [`SignalType`].
///
/// This has an infallible conversion from [`SignalType`] using the [`From`] trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IterableEnum)]
pub(crate) enum SignalKind {
    /// Take this item away from here.
    Push,
    /// Bring me an item of this type.
    Pull,
    /// Perform work at this type of structure.
    Work,
    /// Destroy a structure of this type
    Demolish,
    /// Has an item of this type, in case you were looking.
    ///
    /// The passive form of `Push`.
    Contains,
    /// Stores items of this type, in case you were looking.
    ///
    /// The passive form of `Pull`.
    Stores,
    /// Has a unit of this type.
    Unit,
    /// Draws units in.
    ///
    /// Corrresponds to [`IntentAbility::Lure`].
    Lure,
    /// Pushes units away.
    ///
    /// Corresponds to [`IntentAbility::Repel`].
    Repel,
}

impl From<SignalType> for SignalKind {
    fn from(signal_type: SignalType) -> Self {
        match signal_type {
            SignalType::Push(_) => SignalKind::Push,
            SignalType::Pull(_) => SignalKind::Pull,
            SignalType::Work(_) => SignalKind::Work,
            SignalType::Demolish(_) => SignalKind::Demolish,
            SignalType::Contains(_) => SignalKind::Contains,
            SignalType::Stores(_) => SignalKind::Stores,
            SignalType::Unit(_) => SignalKind::Unit,
            SignalType::Lure => SignalKind::Lure,
            SignalType::Repel => SignalKind::Repel,
        }
    }
}

/// How strong a signal is.
///
/// This has a minimum value of 0.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct SignalStrength(f32);

impl SignalStrength {
    /// No signal is present.
    pub const ZERO: SignalStrength = SignalStrength(0.);

    /// An infinitely strong signal.
    pub const INFINITY: SignalStrength = SignalStrength(f32::INFINITY);

    /// Creates a new [`SignalStrength`], ensuring that it has a minimum value of 0.
    pub fn new(value: f32) -> Self {
        SignalStrength(value.max(0.))
    }

    /// The underlying value
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Applies a [`SignalModifier`] to this signal strength.
    pub fn apply_modifier(&mut self, modifier: SignalModifier) {
        *self *= match modifier {
            SignalModifier::None => 1.,
            SignalModifier::Amplify => SignalModifier::RATIO,
            SignalModifier::Dampen => 1. / SignalModifier::RATIO,
        }
    }
}

impl Add<SignalStrength> for SignalStrength {
    type Output = SignalStrength;

    fn add(self, rhs: SignalStrength) -> Self::Output {
        SignalStrength(self.0 + rhs.0)
    }
}

impl AddAssign<SignalStrength> for SignalStrength {
    fn add_assign(&mut self, rhs: SignalStrength) {
        *self = *self + rhs
    }
}

impl Sub<SignalStrength> for SignalStrength {
    type Output = SignalStrength;

    fn sub(self, rhs: SignalStrength) -> Self::Output {
        SignalStrength((self.0 - rhs.0).max(0.))
    }
}

impl SubAssign<SignalStrength> for SignalStrength {
    fn sub_assign(&mut self, rhs: SignalStrength) {
        *self = *self - rhs
    }
}

impl Mul<f32> for SignalStrength {
    type Output = SignalStrength;

    fn mul(self, rhs: f32) -> Self::Output {
        SignalStrength(self.0 * rhs)
    }
}

impl MulAssign<f32> for SignalStrength {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

/// The component that causes a game object to emit a signal.
///
/// This can change over time, and multiple signals may be emitted at once.
#[derive(Default, Component, Debug, Clone)]
pub(crate) struct Emitter {
    /// The list of signals to emit at a provided
    pub(crate) signals: Vec<(SignalType, SignalStrength)>,
}

/// Modifies the strength of a signal.
///
/// This is stored as a component on each tile, and is applied to all signals emitted from entities at that tile position.
#[derive(Component, Debug, Display, Default, Clone, Copy, PartialEq, Eq)]
pub enum SignalModifier {
    /// No modifier is applied.
    #[default]
    None,
    /// The signal strength is multiplied for the duration of this effect.
    Amplify,
    /// The signal strength is divided for the duration of this effect.
    Dampen,
}

impl SignalModifier {
    /// Controls the ratio of the signal strength when modified.
    ///
    /// This should be greater than 1.
    const RATIO: f32 = 10.;

    /// The cost (in intent) of applying to a single emitter for one second.
    fn cost(&self) -> Intent {
        match self {
            SignalModifier::None => Intent(0.),
            SignalModifier::Amplify => IntentAbility::Amplify.cost(),
            SignalModifier::Dampen => IntentAbility::Dampen.cost(),
        }
    }
}

/// Emits signals from [`Emitter`] sources.
fn emit_signals(
    mut signals: ResMut<Signals>,
    emitter_query: Query<(&TilePos, &Emitter, Option<&Id<Structure>>, Option<&Facing>)>,
    modifier_query: Query<&SignalModifier>,
    structure_manifest: Res<StructureManifest>,
    mut intent_pool: ResMut<IntentPool>,
    fixed_time: Res<FixedTime>,
    map_geometry: Res<MapGeometry>,
) {
    let delta_time = fixed_time.period.as_secs_f32();

    /// Emits signals that correspond to a single [`Emitter`].
    fn emit(
        signals: &mut Signals,
        tile_pos: TilePos,
        emitter: &Emitter,
        modifier: &SignalModifier,
    ) {
        for (signal_type, signal_strength) in &emitter.signals {
            let mut signal_strength = *signal_strength;
            signal_strength.apply_modifier(*modifier);
            signals.add_signal(*signal_type, tile_pos, signal_strength);
        }
    }

    for (&center, emitter, maybe_structure_id, maybe_facing) in emitter_query.iter() {
        match maybe_structure_id {
            // Signals should be emitted from all tiles in the footprint of a structure.
            Some(structure_id) => {
                let facing = *maybe_facing.expect("Structures must have a facing");
                let footprint = &structure_manifest
                    .get(*structure_id)
                    .footprint
                    .rotated(facing);

                for tile_pos in footprint.in_world_space(center) {
                    let terrain_entity = map_geometry.get_terrain(tile_pos).unwrap();
                    let modifier = modifier_query.get(terrain_entity).unwrap();
                    let cost = modifier.cost() * delta_time;

                    if intent_pool.current() >= cost {
                        intent_pool.expend(cost).unwrap();
                    }

                    emit(&mut signals, tile_pos, emitter, modifier);
                }
            }
            None => {
                let terrain_entity = map_geometry.get_terrain(center).unwrap();
                let modifier = modifier_query.get(terrain_entity).unwrap();
                let cost = modifier.cost() * delta_time;

                if intent_pool.current() >= cost {
                    intent_pool.expend(cost).unwrap();
                }

                emit(&mut signals, center, emitter, modifier);
            }
        }
    }
}

/// Spreads signals between tiles.
fn diffuse_signals(mut signals: ResMut<Signals>, map_geometry: Res<MapGeometry>) {
    let map_geometry = &*map_geometry;
    signals.diffuse(map_geometry, DIFFUSION_FRACTION);
}

/// Degrades signals, allowing them to approach an asymptotically constant level.
fn degrade_signals(mut signals: ResMut<Signals>) {
    /// The fraction of signal that will decay at each step.
    ///
    /// Higher values lead to faster decay and improved signal responsiveness.
    /// This must always be between 0 and 1.
    const DEGRADATION_FRACTION: f32 = 0.01;

    /// The value below which decayed signals are eliminated completely
    ///
    /// Increasing this value will:
    ///  - increase computational costs
    ///  - increase the range at which tasks can be detected
    ///  - increase the amount of time units will wait around for more production
    const EPSILON_STRENGTH: SignalStrength = SignalStrength(1e-8);

    for signal_map in signals.maps.values_mut() {
        let mut tiles_to_clear: Vec<TilePos> = Vec::with_capacity(signal_map.map.len());

        for (tile_pos, signal_strength) in signal_map.map.iter_mut() {
            let new_strength = *signal_strength * (1. - DEGRADATION_FRACTION);

            if new_strength > EPSILON_STRENGTH {
                *signal_strength = new_strength;
            } else {
                tiles_to_clear.push(*tile_pos);
            }
        }

        for tile_to_clear in tiles_to_clear {
            signal_map.map.remove(&tile_to_clear);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::items::item_manifest::ItemData;

    use super::*;

    fn test_item() -> ItemKind {
        ItemKind::Single(Id::from_name("12345".to_string()))
    }

    fn test_structure() -> Id<Structure> {
        Id::from_name("67890".to_string())
    }

    fn test_manifest() -> ItemManifest {
        let mut manifest = ItemManifest::new();
        manifest.insert(
            "12345".to_string(),
            ItemData {
                stack_size: 1,
                compostable: false,
                seed: None,
            },
        );
        manifest
    }

    #[test]
    fn neighboring_signals_checks_origin_tile() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Contains(test_item()),
            TilePos::ZERO,
            SignalStrength(1.),
        );

        let neighboring_signals = signals.neighboring_signals(
            SignalType::Contains(test_item()),
            TilePos::ZERO,
            &map_geometry,
        );

        assert_eq!(neighboring_signals.len(), 7);

        assert_eq!(
            neighboring_signals.get(&TilePos::ZERO),
            Some(&SignalStrength(1.))
        );
    }

    #[test]
    fn upstream_returns_none_with_no_signals() {
        let signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Store(test_item()),
                &item_manifest,
                &map_geometry
            ),
            None
        );
        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Fetch(test_item()),
                &item_manifest,
                &map_geometry
            ),
            None
        );
        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Work(WorkplaceId::structure(test_structure())),
                &item_manifest,
                &map_geometry
            ),
            None
        );
        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::default(),
                &item_manifest,
                &map_geometry
            ),
            None
        );
    }

    #[test]
    fn upstream_returns_none_at_trivial_peak() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ZERO,
            SignalStrength(1.),
        );

        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Store(test_item()),
                &item_manifest,
                &map_geometry
            ),
            None
        );
    }

    #[test]
    fn upstream_returns_none_at_peak() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        signals.add_signal(
            SignalType::Push(test_item()),
            TilePos::ZERO,
            SignalStrength(1.),
        );

        for neighbor in TilePos::ZERO.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Push(test_item()), neighbor, SignalStrength(0.5));
        }

        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Fetch(test_item()),
                &item_manifest,
                &map_geometry
            ),
            None
        );
    }

    #[test]
    // The logic for Goal::DropOff is significantly more complex and worth testing separately
    fn upstream_returns_none_at_peak_dropoff() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ZERO,
            SignalStrength(1.),
        );

        for neighbor in TilePos::ZERO.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(0.5));
        }

        assert_eq!(
            signals.upstream(
                TilePos::ZERO,
                &Goal::Store(test_item()),
                &item_manifest,
                &map_geometry
            ),
            None
        );
    }

    #[test]
    fn upstream_returns_some_at_trivial_valley() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        for neighbor in TilePos::ZERO.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(0.5));
        }

        assert!(signals
            .upstream(
                TilePos::ZERO,
                &Goal::Store(test_item()),
                &item_manifest,
                &map_geometry
            )
            .is_some());
    }

    #[test]
    fn upstream_returns_some_at_valley() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);
        let item_manifest = test_manifest();

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ZERO,
            SignalStrength(0.5),
        );

        for neighbor in TilePos::ZERO.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(1.));
        }

        assert!(signals
            .upstream(
                TilePos::ZERO,
                &Goal::Store(test_item()),
                &item_manifest,
                &map_geometry
            )
            .is_some());
    }

    #[test]
    fn item_signal_types_are_correct() {
        let item_kind = test_item();
        let item_manifest = test_manifest();

        assert_eq!(
            SignalType::item_signal_types(
                item_kind,
                &item_manifest,
                DeliveryMode::PickUp,
                Purpose::Intrinsic
            ),
            vec![SignalType::Push(item_kind)]
        );
        assert_eq!(
            SignalType::item_signal_types(
                item_kind,
                &item_manifest,
                DeliveryMode::PickUp,
                Purpose::Instrumental
            ),
            vec![SignalType::Push(item_kind), SignalType::Contains(item_kind)]
        );
        assert_eq!(
            SignalType::item_signal_types(
                item_kind,
                &item_manifest,
                DeliveryMode::DropOff,
                Purpose::Intrinsic
            ),
            vec![SignalType::Pull(item_kind)]
        );
        assert_eq!(
            SignalType::item_signal_types(
                item_kind,
                &item_manifest,
                DeliveryMode::DropOff,
                Purpose::Instrumental
            ),
            vec![SignalType::Pull(item_kind), SignalType::Stores(item_kind)]
        );
    }
}
