//! Signals are used for pathfinding and decision-making.
//!
//! By collecting information about the local environment into a slowly updated, tile-centric data structure,
//! we can scale path-finding and decisionmaking in a clear and comprehensible way.

use bevy::{prelude::*, utils::HashMap};
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use itertools::Itertools;

use crate::asset_management::manifest::{Id, Item, ItemManifest, Structure, StructureManifest};
use crate::simulation::geometry::{MapGeometry, TilePos};
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
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

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
    fn get(&self, signal_type: SignalType, tile_pos: TilePos) -> SignalStrength {
        match self.maps.get(&signal_type) {
            Some(map) => map.get(tile_pos),
            None => SignalStrength::ZERO,
        }
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

    /// Returns the adjacent, empty tile position that contains the highest sum signal strength that can be used to meet the provided `goal`.
    ///
    /// If no suitable tile exists, [`None`] will be returned instead.
    pub(crate) fn upstream(
        &self,
        tile_pos: TilePos,
        goal: &Goal,
        map_geometry: &MapGeometry,
    ) -> Option<TilePos> {
        let mut best_choice: Option<TilePos> = None;
        let mut best_score = SignalStrength::ZERO;

        let neighboring_signals = match goal {
            Goal::Wander => return None,
            Goal::Pickup(item_id) | Goal::Eat(item_id) => {
                let push_signals =
                    self.neighboring_signals(SignalType::Push(*item_id), tile_pos, map_geometry);
                let contains_signals = self.neighboring_signals(
                    SignalType::Contains(*item_id),
                    tile_pos,
                    map_geometry,
                );
                let mut total_signals = push_signals;

                for (tile_pos, signal_strength) in contains_signals {
                    if let Some(existing_signal_strength) = total_signals.get_mut(&tile_pos) {
                        *existing_signal_strength += signal_strength;
                    } else {
                        total_signals.insert(tile_pos, signal_strength);
                    }
                }

                total_signals
            }
            Goal::DropOff(item_id) => {
                let pull_signals =
                    self.neighboring_signals(SignalType::Pull(*item_id), tile_pos, map_geometry);
                let stores_signals =
                    self.neighboring_signals(SignalType::Stores(*item_id), tile_pos, map_geometry);
                let mut total_signals = pull_signals;

                for (tile_pos, signal_strength) in stores_signals {
                    if let Some(existing_signal_strength) = total_signals.get_mut(&tile_pos) {
                        *existing_signal_strength += signal_strength;
                    } else {
                        total_signals.insert(tile_pos, signal_strength);
                    }
                }

                total_signals
            }
            Goal::Work(structure_id) => {
                self.neighboring_signals(SignalType::Work(*structure_id), tile_pos, map_geometry)
            }
            Goal::Demolish(structure_id) => self.neighboring_signals(
                SignalType::Demolish(*structure_id),
                tile_pos,
                map_geometry,
            ),
        };

        for (possible_tile, current_score) in neighboring_signals {
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

    /// Returns the signal strength of the type `signal_type` in `tile_pos` and its 6 surrounding neighbors.
    fn neighboring_signals(
        &self,
        signal_type: SignalType,
        tile_pos: TilePos,
        map_geometry: &MapGeometry,
    ) -> HashMap<TilePos, SignalStrength> {
        let mut signal_strength_map = HashMap::with_capacity(7);

        signal_strength_map.insert(tile_pos, self.get(signal_type, tile_pos));
        for neighbor in tile_pos.all_neighbors(map_geometry) {
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
                for neighboring_tile in occupied_tile.empty_neighbors(map_geometry) {
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
        self.map.iter().filter(|(signal_type, _signal_strength)| {
            !matches!(
                **signal_type,
                SignalType::Contains(_) | SignalType::Stores(_)
            )
        })
    }

    /// The pretty formatting for this type.
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
    ) -> String {
        let mut string = String::default();

        for signal_type in self.map.keys().sorted() {
            let signal_strength = self.map.get(signal_type).unwrap().0;

            let substring = format!(
                "{}: {signal_strength:.3}\n",
                signal_type.display(item_manifest, structure_manifest)
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
    /// Returns the signal strenth at the given [`TilePos`].
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
    Push(Id<Item>),
    /// Bring me an item of this type.
    Pull(Id<Item>),
    /// Has an item of this type, in case you were looking.
    ///
    /// The passive form of `Push`.
    Contains(Id<Item>),
    /// Stores items of this type, in case you were looking.
    ///
    /// The passive form of `Pull`.
    Stores(Id<Item>),
    /// Perform work at this type of structure.
    #[allow(dead_code)]
    Work(Id<Structure>),
    /// Destroy a structure of this type
    Demolish(Id<Structure>),
}

impl SignalType {
    /// The pretty formatting for this type
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
    ) -> String {
        match self {
            SignalType::Push(item_id) => format!("Push({})", item_manifest.name(*item_id)),
            SignalType::Pull(item_id) => format!("Pull({})", item_manifest.name(*item_id)),
            SignalType::Contains(item_id) => format!("Contains({})", item_manifest.name(*item_id)),
            SignalType::Stores(item_id) => format!("Stores({})", item_manifest.name(*item_id)),
            SignalType::Work(structure_id) => {
                format!("Work({})", structure_manifest.name(*structure_id))
            }
            SignalType::Demolish(structure_id) => {
                format!("Demolish({})", structure_manifest.name(*structure_id))
            }
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

    /// Creates a new [`SignalStrength`], ensuring that it has a minimum value of 0.
    pub fn new(value: f32) -> Self {
        SignalStrength(value.max(0.))
    }

    /// The underlying value
    pub fn value(&self) -> f32 {
        self.0
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

/// The component that causes a game object to emit a signal.
///
/// This can change over time, and multiple signals may be emitted at once.
#[derive(Default, Component, Debug, Clone)]
pub(crate) struct Emitter {
    /// The list of signals to emit at a provided
    pub(crate) signals: Vec<(SignalType, SignalStrength)>,
}

/// Emits signals from [`Emitter`] sources.
fn emit_signals(mut signals: ResMut<Signals>, emitter_query: Query<(&TilePos, &Emitter)>) {
    for (&tile_pos, emitter) in emitter_query.iter() {
        for (signal_type, signal_strength) in &emitter.signals {
            signals.add_signal(*signal_type, tile_pos, *signal_strength);
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
    use super::*;

    fn test_item() -> Id<Item> {
        Id::from_name("12345")
    }

    fn test_structure() -> Id<Structure> {
        Id::from_name("67890")
    }

    #[test]
    fn neighboring_signals_checks_origin_tile() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Contains(test_item()),
            TilePos::ORIGIN,
            SignalStrength(1.),
        );

        let neighboring_signals = signals.neighboring_signals(
            SignalType::Contains(test_item()),
            TilePos::ORIGIN,
            &map_geometry,
        );

        assert_eq!(neighboring_signals.len(), 7);

        assert_eq!(
            neighboring_signals.get(&TilePos::ORIGIN),
            Some(&SignalStrength(1.))
        );
    }

    #[test]
    fn upstream_returns_none_with_no_signals() {
        let signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::DropOff(test_item()), &map_geometry),
            None
        );
        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::Pickup(test_item()), &map_geometry),
            None
        );
        assert_eq!(
            signals.upstream(
                TilePos::ORIGIN,
                &Goal::Work(test_structure()),
                &map_geometry
            ),
            None
        );
        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::Wander, &map_geometry),
            None
        );
    }

    #[test]
    fn upstream_returns_none_at_trivial_peak() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ORIGIN,
            SignalStrength(1.),
        );

        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::DropOff(test_item()), &map_geometry),
            None
        );
    }

    #[test]
    fn upstream_returns_none_at_peak() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Push(test_item()),
            TilePos::ORIGIN,
            SignalStrength(1.),
        );

        for neighbor in TilePos::ORIGIN.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Push(test_item()), neighbor, SignalStrength(0.5));
        }

        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::Pickup(test_item()), &map_geometry),
            None
        );
    }

    #[test]
    // The logic for Goal::DropOff is significantly more complex and worth testing separately
    fn upstream_returns_none_at_peak_dropoff() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ORIGIN,
            SignalStrength(1.),
        );

        for neighbor in TilePos::ORIGIN.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(0.5));
        }

        assert_eq!(
            signals.upstream(TilePos::ORIGIN, &Goal::DropOff(test_item()), &map_geometry),
            None
        );
    }

    #[test]
    fn upstream_returns_some_at_trivial_valley() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        for neighbor in TilePos::ORIGIN.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(0.5));
        }

        assert!(signals
            .upstream(TilePos::ORIGIN, &Goal::DropOff(test_item()), &map_geometry)
            .is_some());
    }

    #[test]
    fn upstream_returns_some_at_valley() {
        let mut signals = Signals::default();
        let map_geometry = MapGeometry::new(1);

        signals.add_signal(
            SignalType::Pull(test_item()),
            TilePos::ORIGIN,
            SignalStrength(0.5),
        );

        for neighbor in TilePos::ORIGIN.all_neighbors(&map_geometry) {
            signals.add_signal(SignalType::Pull(test_item()), neighbor, SignalStrength(1.));
        }

        assert!(signals
            .upstream(TilePos::ORIGIN, &Goal::DropOff(test_item()), &map_geometry)
            .is_some());
    }
}
