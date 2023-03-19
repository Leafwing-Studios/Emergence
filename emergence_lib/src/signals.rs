//! Signals are used for pathfinding and decision-making.
//!
//! By collecting information about the local environment into a slowly updated, tile-centric data structure,
//! we can scale path-finding and decisionmaking in a clear and comprehensible way.

mod equation;

use bevy::{prelude::*, utils::HashMap};
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use itertools::Itertools;

use crate::asset_management::manifest::{
    Id, Item, ItemManifest, Structure, StructureManifest, Terrain, Unit, UnitManifest,
};
use crate::simulation::geometry::{MapGeometry, TilePos};
use crate::simulation::SimulationSet;
use crate::units::goals::Goal;

use self::equation::DiffusionEquation;

/// The diffusivity of signals: how fast signals diffuse to neighboring tiles, in seconds per
/// tile.
pub const DIFFUSIVITY: f32 = 0.2;

/// The decay rate of signals: how fast signals decay, in "signal strength" per second.
pub const DECAY_RATE: f32 = 2.0;

/// The strength below which a signal is considered negligible. The total
/// quantity of a single emission is considered when trimming signals.
pub const DECAY_THRESHOLD: f32 = 0.1;

/// The resources and systems need to work with signals
pub(crate) struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Signals>()
            .init_resource::<DebugColorScheme>()
            .insert_resource(DebugDisplayedSignal(SignalType::Push(Id::from_name(
                "acacia_leaf",
            ))))
            .add_systems(
                (emit_signals, update_signals)
                    .chain()
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .add_system(
                debug_display_signal_overlay
                    .run_if(debug_signal_overlay_enabled)
                    .in_base_set(CoreSet::PostUpdate),
            );
    }
}

/// The central resource that tracks all signals.
#[derive(Resource, Debug, Default)]
pub struct Signals {
    /// The equations associated to each signal type.
    signal_equations: HashMap<SignalType, DiffusionEquation>,
}

impl Signals {
    /// Returns the signal strength of `signal_type` at the given `tile_pos`.
    ///
    /// Missing values will be filled with [`SignalStrength::ZERO`].
    fn get(&self, signal_type: SignalType, tile_pos: TilePos) -> SignalStrength {
        match self.signal_equations.get(&signal_type) {
            Some(equation) => equation.evaluate_signal(tile_pos),
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
        match self.signal_equations.get_mut(&signal_type) {
            Some(equation) => equation.emit_signal(tile_pos, signal_strength),
            None => {
                let mut new_equation = DiffusionEquation::default();
                new_equation.emit_signal(tile_pos, signal_strength);
                self.signal_equations.insert(signal_type, new_equation);
            }
        }
    }

    /// Returns the complete set of signals at the given `tile_pos`.
    ///
    /// This is useful for decision-making.
    pub(crate) fn all_signals_at_position(&self, tile_pos: TilePos) -> LocalSignals {
        let mut all_signals = HashMap::new();
        for &signal_type in self.signal_equations.keys() {
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
            Goal::Wander { .. } => return None,
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
            Goal::Store(item_id) => {
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
            Goal::Deliver(item_id) => {
                self.neighboring_signals(SignalType::Pull(*item_id), tile_pos, map_geometry)
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
        unit_manifest: &UnitManifest,
    ) -> String {
        let mut string = String::default();

        for signal_type in self.map.keys().sorted() {
            let signal_strength = self.map.get(signal_type).unwrap().0;

            let substring = format!(
                "{}: {signal_strength:.3}\n",
                signal_type.display(item_manifest, structure_manifest, unit_manifest)
            );

            string += &substring;
        }

        string
    }
}

/// The variety of signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalType {
    /// Take this item away from here.
    Push(Id<Item>),
    /// Bring me an item of this type.
    Pull(Id<Item>),
    /// Perform work at this type of structure.
    #[allow(dead_code)]
    Work(Id<Structure>),
    /// Destroy a structure of this type
    Demolish(Id<Structure>),
    /// Has an item of this type, in case you were looking.
    ///
    /// The passive form of `Push`.
    Contains(Id<Item>),
    /// Stores items of this type, in case you were looking.
    ///
    /// The passive form of `Pull`.
    Stores(Id<Item>),
    /// Has a unit of this type.
    Unit(Id<Unit>),
}

impl SignalType {
    /// The pretty formatting for this type
    pub(crate) fn display(
        &self,
        item_manifest: &ItemManifest,
        structure_manifest: &StructureManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        match self {
            SignalType::Push(item_id) => format!("Push({})", item_manifest.name(*item_id)),
            SignalType::Pull(item_id) => format!("Pull({})", item_manifest.name(*item_id)),
            SignalType::Work(structure_id) => {
                format!("Work({})", structure_manifest.name(*structure_id))
            }
            SignalType::Demolish(structure_id) => {
                format!("Demolish({})", structure_manifest.name(*structure_id))
            }
            SignalType::Contains(item_id) => format!("Contains({})", item_manifest.name(*item_id)),
            SignalType::Stores(item_id) => format!("Stores({})", item_manifest.name(*item_id)),
            SignalType::Unit(unit_id) => format!("Unit({})", unit_manifest.name(*unit_id)),
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
fn update_signals(mut signals: ResMut<Signals>, time: Res<Time>) {
    let current_time = time.elapsed_seconds_wrapped();
    let mut signals_to_clear = Vec::new();
    for (signal_type, equation) in signals.signal_equations.iter_mut() {
        equation.advance_time(current_time);
        if equation.trim_neglible_emissions() {
            signals_to_clear.push(*signal_type);
        }
    }
    for signal_type in signals_to_clear {
        signals.signal_equations.remove(&signal_type);
        trace!("Cleared signal {:?}", signal_type);
    }
    trace!("{} Signal types left", signals.signal_equations.len());
}

#[derive(Resource, Debug)]
pub(crate) struct DebugDisplayedSignal(SignalType);

#[derive(Resource, Debug)]
struct DebugColorScheme(Vec<Handle<StandardMaterial>>);

impl FromWorld for DebugColorScheme {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        let mut color_scheme = Vec::with_capacity(256);
        // FIXME: This color palette is not very colorblind-friendly, even though it was inspired
        // by matlab's veridis
        for i in 0..256 {
            let s = i as f32 / 255.0;
            color_scheme.push(material_assets.add(StandardMaterial {
                base_color: Color::Rgba {
                    red: 0.8 * (2.0 * s - s * s),
                    green: 0.8 * s.sqrt(),
                    blue: s * s * 0.6,
                    alpha: 0.8,
                },
                unlit: true,
                alpha_mode: AlphaMode::Add,
                ..Default::default()
            }));
        }

        color_scheme.shrink_to_fit();
        DebugColorScheme(color_scheme)
    }
}

fn debug_signal_overlay_enabled(displayed_signal: Option<Res<DebugDisplayedSignal>>) -> bool {
    displayed_signal.is_some()
}

pub(crate) fn debug_signal_overlay_disabled(
    displayed_signal: Option<Res<DebugDisplayedSignal>>,
) -> bool {
    displayed_signal.is_none()
}

fn debug_display_signal_overlay(
    terrain_query: Query<(&TilePos, &Children), With<Id<Terrain>>>,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    signals: Res<Signals>,
    displayed_signal: Res<DebugDisplayedSignal>,
    color_scheme: Res<DebugColorScheme>,
) {
    for (tile_pos, children) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        let (mut overlay_material, mut overlay_visibility) =
            overlay_query.get_mut(overlay_entity).unwrap();

        let signal_strength = signals.get(displayed_signal.0, *tile_pos).0;
        // Just a simple dark red (low strength) to bright yellow (high strength) color scheme
        // The scale is logarithmic, so that small nuances are still pretty visible
        let scaled_strength = signal_strength.ln_1p() / 6.0;
        let color_index = if signal_strength < f32::EPSILON {
            *overlay_visibility = Visibility::Hidden;
            continue;
        } else {
            *overlay_visibility = Visibility::Visible;
            ((scaled_strength * 254.0) as usize) + 1
        };
        *overlay_material.as_mut() = color_scheme.0[color_index.min(255)].clone_weak();
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
            signals.upstream(TilePos::ORIGIN, &Goal::Store(test_item()), &map_geometry),
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
            signals.upstream(TilePos::ORIGIN, &Goal::default(), &map_geometry),
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
            signals.upstream(TilePos::ORIGIN, &Goal::Store(test_item()), &map_geometry),
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
            signals.upstream(TilePos::ORIGIN, &Goal::Store(test_item()), &map_geometry),
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
            .upstream(TilePos::ORIGIN, &Goal::Store(test_item()), &map_geometry)
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
            .upstream(TilePos::ORIGIN, &Goal::Store(test_item()), &map_geometry)
            .is_some());
    }
}
