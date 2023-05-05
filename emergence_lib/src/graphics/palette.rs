//! A central source of truth for the game and UI's color palettes.

/// Colors used for visualizing information about the world.
pub(crate) mod infovis {
    use bevy::prelude::Color;

    use crate::signals::SignalKind;

    /// The alpha value used for selection/hovering/other UI overlay
    pub(crate) const OVERLAY_ALPHA: f32 = 0.5;

    /// The hue of selected objects
    pub(crate) const SELECTION_HUE: f32 = 100.;
    /// The saturation of selected objects
    pub(crate) const SELECTION_SATURATION: f32 = 0.5;
    /// The lightness of selected objects
    pub(crate) const SELECTION_LIGHTNESS: f32 = 0.6;
    /// The color used to tint selected objects.
    pub(crate) const SELECTION_COLOR: Color = Color::hsla(
        SELECTION_HUE,
        SELECTION_SATURATION,
        SELECTION_LIGHTNESS,
        OVERLAY_ALPHA,
    );

    /// The hue used to indicate that an action is forbidden.
    pub(crate) const FORBIDDEN_HUE: f32 = 0.;

    /// The hue of selected objects
    pub(crate) const HOVER_HUE: f32 = 55.;
    /// The saturation of selected objects
    pub(crate) const HOVER_SATURATION: f32 = 0.5;
    /// The lightness of selected objects
    pub(crate) const HOVER_LIGHTNESS: f32 = 0.6;

    /// The color used to tint hovered objects.
    pub(crate) const HOVER_COLOR: Color =
        Color::hsla(HOVER_HUE, HOVER_SATURATION, HOVER_LIGHTNESS, OVERLAY_ALPHA);

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
    /// The color used to tint previews that cannot be built
    pub(crate) const FORBIDDEN_PREVIEW_COLOR: Color = Color::hsla(
        FORBIDDEN_HUE,
        HOVER_SATURATION,
        HOVER_LIGHTNESS,
        GHOST_ALPHA,
    );

    /// The color used to tint objects that are both selected and hovered.
    pub(crate) const SELECTION_AND_HOVER_COLOR: Color = Color::hsla(
        (SELECTION_HUE + HOVER_HUE) / 2.,
        (SELECTION_SATURATION + HOVER_SATURATION) / 2.,
        (SELECTION_LIGHTNESS + HOVER_LIGHTNESS) / 2.,
        OVERLAY_ALPHA,
    );

    impl SignalKind {
        /// The saturation used to indicate that the signal strength is low.
        const SIGNAL_SATURATION_LOW: f32 = 0.0;
        /// The saturation used to indicate that the signal strength is high.
        const SIGNAL_SATURATION_HIGH: f32 = 1.0;

        /// The lightness used to indicate that the signal strength is low.
        const SIGNAL_LIGHTNESS_LOW: f32 = 0.0;
        /// The lightness used to indicate that the signal strength is high.
        const SIGNAL_LIGHTNESS_HIGH: f32 = 1.0;

        /// The base hue used for each signal kind.
        /// The principles here are:
        /// - Red is for destruction
        /// - Similar colors are for similar signals
        /// - More vibrant colors are for signals that generate goals
        pub(crate) const fn hue(&self) -> f32 {
            match self {
                // Orange
                SignalKind::Pull => 20.,
                // Yellow
                SignalKind::Stores => 70.,
                // Green
                SignalKind::Push => 130.,
                // Teal
                SignalKind::Contains => 180.,
                // Purple
                SignalKind::Work => 300.,
                // Red
                SignalKind::Demolish => 0.,
                // Blue
                SignalKind::Unit => 220.,
                // Blue-purple
                SignalKind::Lure => 270.,
                // Orange-red
                SignalKind::Repel => 10.,
            }
        }

        /// The base color used for each signal kind.
        pub(crate) const fn color(&self) -> Color {
            Color::hsla(self.hue(), 0.6, 0.5, 1.0)
        }

        /// The color used to indicate that the signal strength is low.
        pub(crate) const fn color_low(&self) -> Color {
            Color::hsla(
                self.hue(),
                Self::SIGNAL_SATURATION_LOW,
                Self::SIGNAL_LIGHTNESS_LOW,
                OVERLAY_ALPHA,
            )
        }

        /// The color used to indicate that the signal strength is high.
        pub(crate) const fn color_high(&self) -> Color {
            Color::hsla(
                self.hue(),
                Self::SIGNAL_SATURATION_HIGH,
                Self::SIGNAL_LIGHTNESS_HIGH,
                OVERLAY_ALPHA,
            )
        }
    }

    /// The color used to represent a "neutral" value in bicolor gradients.
    pub(crate) const NEUTRAL_INFOVIS_COLOR: Color = Color::hsla(0., 0., 0.7, OVERLAY_ALPHA);

    /// The color used to indicate that water is far from the surface.
    pub(crate) const WATER_TABLE_COLOR_HIGH: Color = Color::hsla(5., 0.6, 0.5, OVERLAY_ALPHA);
    /// The color used to indicate that water is near the surface.
    pub(crate) const WATER_TABLE_COLOR_LOW: Color = Color::hsla(195., 0.7, 0.2, OVERLAY_ALPHA);

    /// The color used to indicate that it is dark.
    pub(crate) const LIGHT_LEVEL_COLOR_LOW: Color = Color::hsla(232., 0.68, 0.4, OVERLAY_ALPHA);
    /// The color used to indicate that it is bright.
    // This is very hard to see at low alpha, so we use a higher alpha here.
    pub(crate) const LIGHT_LEVEL_COLOR_HIGH: Color =
        Color::hsla(52., 0.7, 0.95, (1.0 + OVERLAY_ALPHA) / 2.);
}

/// Colors used for the world's environment
pub(crate) mod environment {
    use bevy::prelude::Color;

    use crate::simulation::weather::Weather;

    /// The color used for columns of dirt underneath tiles
    pub(crate) const COLUMN_COLOR: Color = Color::hsl(21., 0.6, 0.15);

    impl Weather {
        pub(crate) const fn sky_color(&self) -> Color {
            match self {
                Weather::Clear => Color::hsl(209., 0.7, 0.8),
                Weather::Cloudy => Color::hsl(209., 0.3, 0.6),
                Weather::Rainy => Color::hsl(209., 0.3, 0.5),
            }
        }
    }

    /// The color of the surface water.
    pub(crate) const WATER: Color = Color::Hsla {
        hue: 202.,
        saturation: 0.7,
        lightness: 0.5,
        alpha: 0.3,
    };
}

/// Colors used for lighting
pub(crate) mod lighting {
    use bevy::prelude::Color;

    /// The color of daylight
    pub(crate) const LIGHT_SUN: Color = Color::Hsla {
        hue: 30.,
        saturation: 1.0,
        lightness: 1.,
        alpha: 1.,
    };

    /// The color of moonlight
    pub(crate) const LIGHT_MOON: Color = Color::Hsla {
        hue: 198.,
        saturation: 1.0,
        lightness: 1.,
        alpha: 1.,
    };

    /// The color of starlight
    pub(crate) const LIGHT_STARS: Color = Color::WHITE;
}

/// Colors used in the UI
pub(crate) mod ui {
    use bevy::prelude::Color;

    /// The color used by highlighted /selected menu options
    pub(crate) const MENU_HIGHLIGHT_COLOR: Color = Color::Hsla {
        hue: 0.,
        saturation: 0.,
        lightness: 0.95,
        alpha: 1.0,
    };

    /// The color used by neutral / unselected menu options
    pub(crate) const MENU_NEUTRAL_COLOR: Color = Color::Hsla {
        hue: 0.,
        saturation: 0.,
        lightness: 0.7,
        alpha: 1.0,
    };
}
