//! A central source of truth for the game and UI's color palettes.

use bevy::prelude::Color;

/// The hue of selected objects
pub(crate) const SELECTION_HUE: f32 = 100.;
/// The saturation of selected objects
pub(crate) const SELECTION_SATURATION: f32 = 0.5;
/// The lightness of selected objects
pub(crate) const SELECTION_LIGHTNESS: f32 = 0.6;
/// The color used to tint selected objects.
pub(crate) const SELECTION_COLOR: Color =
    Color::hsl(SELECTION_HUE, SELECTION_SATURATION, SELECTION_LIGHTNESS);

/// The hue of selected objects
pub(crate) const HOVER_HUE: f32 = 55.;
/// The saturation of selected objects
pub(crate) const HOVER_SATURATION: f32 = 0.5;
/// The lightness of selected objects
pub(crate) const HOVER_LIGHTNESS: f32 = 0.6;
/// The color used to tint hovered objects.
pub(crate) const HOVER_COLOR: Color = Color::hsl(HOVER_HUE, HOVER_SATURATION, HOVER_LIGHTNESS);

/// The hue value of ghost-like materials.
pub(crate) const GHOST_HUE: f32 = 0.0;
/// The saturation value of ghost-like materials.
pub(crate) const GHOST_SATURATION: f32 = 0.;
/// The lightness value of ghost-like materials.
pub(crate) const GHOST_LIGHTNESS: f32 = 0.9;
/// The alpha value of ghost-like materials.
pub(crate) const GHOST_ALPHA: f32 = 0.7;
/// The color used to tint ghosts
pub(crate) const GHOST_COLOR: Color =
    Color::hsla(GHOST_HUE, GHOST_SATURATION, GHOST_LIGHTNESS, GHOST_ALPHA);
/// The color used to tint selected ghosts
pub(crate) const SELECTED_GHOST_COLOR: Color = Color::hsla(
    SELECTION_HUE,
    SELECTION_SATURATION,
    SELECTION_LIGHTNESS,
    GHOST_ALPHA,
);

/// The color used to tint previews
pub(crate) const PREVIEW_COLOR: Color =
    Color::hsla(HOVER_HUE, HOVER_SATURATION, HOVER_LIGHTNESS, GHOST_ALPHA);

/// The color used to tint objects that are both selected and hovered.
pub(crate) const SELECTION_AND_HOVER_COLOR: Color = Color::hsl(
    (SELECTION_HUE + HOVER_HUE) / 2.,
    (SELECTION_SATURATION + HOVER_SATURATION) / 2.,
    (SELECTION_LIGHTNESS + HOVER_LIGHTNESS) / 2.,
);
