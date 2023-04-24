//! Loads and manages asset state for in-game UI

use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use core::fmt::Debug;
use core::hash::Hash;

use crate::{
    asset_management::{manifest::Id, AssetState, Loadable},
    construction::terraform::TerraformingTool,
    enum_iter::IterableEnum,
    items::item_manifest::{Item, ItemManifest},
    player_interaction::abilities::IntentAbility,
    structures::structure_manifest::{Structure, StructureManifest},
    terrain::terrain_manifest::TerrainManifest,
    units::{
        goals::GoalKind,
        unit_manifest::{Unit, UnitManifest},
    },
};

use super::status::CraftingProgress;

/// The size of icons used to represent choices in menus
pub(crate) const CHOICE_ICON_SIZE: f32 = 64.0;

/// Stores all structural elements of the UI: buttons, frames, widgets and so on
#[derive(Resource)]
pub(crate) struct UiElements {
    /// The background image used by hex menus
    pub(crate) hex_menu_background: Handle<Image>,
}

impl Loadable for UiElements {
    const STAGE: AssetState = AssetState::LoadAssets;

    fn initialize(world: &mut World) {
        let asset_server = world.resource::<AssetServer>();
        world.insert_resource(UiElements {
            hex_menu_background: asset_server.load("ui/hex-menu-background.png"),
        });
    }

    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        asset_server.get_load_state(&self.hex_menu_background)
    }
}

/// Stores the icons of type `D`.
#[derive(Resource)]
pub(crate) struct Icons<D: Send + Sync + 'static> {
    /// The map used to look-up handles
    map: HashMap<D, Handle<Image>>,
}

impl<D: Send + Sync + 'static + Hash + Eq> Icons<D> {
    /// Returns a weakly cloned handle to the image of the icon corresponding to `key`.
    pub(crate) fn get(&self, key: D) -> Handle<Image> {
        self.map.get(&key).unwrap().clone_weak()
    }
}

impl FromWorld for Icons<Id<Item>> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let item_manifest = world.resource::<ItemManifest>();
        let item_names = item_manifest.names();

        let mut map = HashMap::new();

        for id in item_names {
            let item_id = Id::from_name(id.to_string());
            let item_path = format!("icons/items/{id}.png");
            let icon = asset_server.load(item_path);
            map.insert(item_id, icon);
        }

        Icons { map }
    }
}

impl FromWorld for Icons<Id<Structure>> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let structure_manifest = world.resource::<StructureManifest>();
        let structure_names = structure_manifest.prototype_names();

        let mut map = HashMap::new();

        for id in structure_names {
            let structure_id = Id::from_name(id.to_string());
            let structure_path = format!("icons/structures/{id}.png");
            let icon = asset_server.load(structure_path);
            map.insert(structure_id, icon);
        }

        Icons { map }
    }
}

impl FromWorld for Icons<TerraformingTool> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();

        let terrain_names = world.resource::<TerrainManifest>().names();

        for id in terrain_names {
            let terrain_id = Id::from_name(id.to_string());
            let terrain_path = format!("icons/terrain/{id}.png");
            let icon = asset_server.load(terrain_path);

            let choice = TerraformingTool::Change(terrain_id);
            map.insert(choice, icon);
        }

        map.insert(
            TerraformingTool::Lower,
            asset_server.load("icons/terraforming/lower.png"),
        );

        map.insert(
            TerraformingTool::Raise,
            asset_server.load("icons/terraforming/raise.png"),
        );

        Icons { map }
    }
}

impl FromWorld for Icons<Id<Unit>> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let unit_manifest = world.resource::<UnitManifest>();
        let unit_names = unit_manifest.names();

        let mut map = HashMap::new();

        for id in unit_names {
            let unit_id = Id::from_name(id.to_string());
            let unit_path = format!("icons/units/{id}.png");
            let icon = asset_server.load(unit_path);
            map.insert(unit_id, icon);
        }

        Icons { map }
    }
}

impl FromWorld for Icons<IntentAbility> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();

        for ability in IntentAbility::variants() {
            let ability_name = format!("{ability}").to_lowercase();
            let ability_path = format!("icons/abilities/{ability_name}.png");
            let icon = asset_server.load(ability_path);
            map.insert(ability, icon);
        }

        Icons { map }
    }
}

impl FromWorld for Icons<CraftingProgress> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();

        map.insert(
            CraftingProgress::NoRecipe,
            asset_server.load("icons/crafting_progress/no_recipe.png"),
        );
        map.insert(
            CraftingProgress::NeedsInput,
            asset_server.load("icons/crafting_progress/needs_input.png"),
        );
        map.insert(
            CraftingProgress::FullAndBlocked,
            asset_server.load("icons/crafting_progress/full_and_blocked.png"),
        );

        for wedges in 0..=6 {
            let path = format!("icons/crafting_progress/progress_{wedges}_of_6.png");
            let icon = asset_server.load(path);
            map.insert(CraftingProgress::InProgress(wedges), icon);
        }

        Icons { map }
    }
}

impl FromWorld for Icons<GoalKind> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();

        map.insert(GoalKind::Avoid, asset_server.load("icons/goals/avoid.png"));
        map.insert(
            GoalKind::Deliver,
            asset_server.load("icons/goals/deliver.png"),
        );
        map.insert(
            GoalKind::Demolish,
            asset_server.load("icons/goals/demolish.png"),
        );
        map.insert(GoalKind::Eat, asset_server.load("icons/goals/eat.png"));
        map.insert(GoalKind::Fetch, asset_server.load("icons/goals/fetch.png"));
        map.insert(GoalKind::Lure, asset_server.load("icons/goals/lure.png"));
        map.insert(
            GoalKind::Remove,
            asset_server.load("icons/goals/remove.png"),
        );
        map.insert(GoalKind::Repel, asset_server.load("icons/goals/repel.png"));
        map.insert(GoalKind::Store, asset_server.load("icons/goals/store.png"));
        map.insert(
            GoalKind::Wander,
            asset_server.load("icons/goals/wander.png"),
        );

        Icons { map }
    }
}

impl<D: Send + Sync + Debug + 'static> Loadable for Icons<D>
where
    Icons<D>: FromWorld,
{
    const STAGE: AssetState = AssetState::LoadAssets;

    fn initialize(world: &mut World) {
        let icons = Self::from_world(world);
        world.insert_resource(icons);
    }

    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        for (data, icon_handle) in &self.map {
            let load_state = asset_server.get_load_state(icon_handle);

            if load_state != LoadState::Loaded {
                info!("{data:?}'s icon is {load_state:?}");
                return load_state;
            }
        }

        LoadState::Loaded
    }
}
