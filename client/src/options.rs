//! Client settings

use std::marker::PhantomData;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use traffloat::def::{cargo, gas, liquid};
use traffloat::lerp;
use traffloat::space::Vector;
use yew::services::storage;

/// The localStorage key for options
pub const STORAGE_KEY: &str = "traffloat:game_options";

/// All settings for the client, serialized in `localStorage`.
#[derive(Debug, Clone, Serialize, Deserialize, getset::Getters, getset::MutGetters)]
pub struct Options {
    /// Graphics settings.
    #[getset(get = "pub", get_mut = "pub")]
    graphics: Graphics,
}

/// Graphics settings.
#[derive(
    Debug, Clone, Serialize, Deserialize, getset::CopyGetters, getset::Getters, getset::MutGetters,
)]
pub struct Graphics {
    /// Whether stars should be rendered.
    #[getset(get_copy = "pub", get_mut = "pub")]
    render_stars:      bool,
    /// Whether axis reticle should be rendered.
    #[getset(get_copy = "pub", get_mut = "pub")]
    render_reticle:    bool,
    /// Whether debug info should be rendered.
    #[getset(get_copy = "pub", get_mut = "pub")]
    render_debug_info: bool,
    /// Options for node rendering.
    #[getset(get = "pub", get_mut = "pub")]
    node:              Node,
    /// Options for edge rendering.
    #[getset(get = "pub", get_mut = "pub")]
    edge:              Edge,
}

/// Node rendering settings.
#[derive(
    Debug, Clone, Serialize, Deserialize, getset::CopyGetters, getset::Getters, getset::MutGetters,
)]
pub struct Node {
    /// Whether nodes should be rendered.
    #[getset(get_copy = "pub", get_mut = "pub")]
    render:              bool,
    /// The base color to draw upon.
    #[getset(get_copy = "pub", get_mut = "pub")]
    base:                Vector,
    /// The texture variant name to draw with, or `None` if textures should not be used.
    #[getset(get = "pub", get_mut = "pub")]
    texture:             Option<ArcStr>,
    /// A color filter based on total cargo storage size of a type in the node,
    /// relative to the largest rendered storage.
    #[getset(get = "pub", get_mut = "pub")]
    cargo:               Option<(cargo::TypeId, ColorMap)>,
    /// A color filter based on total liquid storage size of a type in the node,
    /// relative to the largest rendered storage.
    #[getset(get = "pub", get_mut = "pub")]
    liquid:              Option<(liquid::TypeId, ColorMap)>,
    /// A color filter based on total gas storage size of a type in the node,
    /// relative to the largest rendered storage.
    #[getset(get = "pub", get_mut = "pub")]
    gas:                 Option<(gas::TypeId, ColorMap)>,
    /// A color filter based on electricity used/generated by the node.
    /// The colormap is unevenly scaled into [0, 0.5] and [0.5, 1],
    /// the former mapped to maximum consumption and
    /// the latter mapped to the maximum production.
    #[getset(get_copy = "pub", get_mut = "pub")]
    electricity_usage:   Option<ColorMap>,
    /// A color filter based on electricity surplus of the electricity grid
    /// that the node belongs to, relative to the largest electricity grid.
    #[getset(get_copy = "pub", get_mut = "pub")]
    electricity_surplus: Option<ColorMap>,
    /// A color filter based on sunlight received by the node,
    /// relative to the largest rendered brightness.
    #[getset(get_copy = "pub", get_mut = "pub")]
    brightness:          Option<ColorMap>,
    /// A color filter based on the percentage hitpoint of the node,
    /// relative to the largest rendered percentage hitpoint.
    #[getset(get_copy = "pub", get_mut = "pub")]
    hitpoint:            Option<ColorMap>,
}

/// Edge rendering settings.
#[derive(
    Debug, Clone, Serialize, Deserialize, getset::Getters, getset::CopyGetters, getset::MutGetters,
)]
pub struct Edge {
    /// Whether nodes should be rendered.
    #[getset(get_copy = "pub", get_mut = "pub")]
    render:              bool,
    /// The base color to draw upon.
    #[getset(get_copy = "pub", get_mut = "pub")]
    base:                Vector,
    /// A color filter based on total cargo transfer rate of a type across the edge,
    /// relative to the largest rendered rate.
    #[getset(get = "pub", get_mut = "pub")]
    cargo:               Option<(cargo::TypeId, ColorMap)>, // unimplemented
    /// A color filter based on total liquid storage size of a type in the edge,
    /// relative to the largest rendered rate.
    #[getset(get = "pub", get_mut = "pub")]
    liquid:              Option<(liquid::TypeId, ColorMap)>, // unimplemented
    /// A color filter based on total gas storage size of a type in the edge,
    /// relative to the largest rendered rate.
    #[getset(get = "pub", get_mut = "pub")]
    gas:                 Option<(gas::TypeId, ColorMap)>, // unimplemented
    /// A color filter based on electric current transferred across the edge,
    /// relative to the largest rendered current.
    #[getset(get_copy = "pub", get_mut = "pub")]
    electricity_current: Option<ColorMap>, // unimplemented
    /// A color filter based on electricity surplus of the electricity grid
    /// that the edge belongs to, relative to the largest electricity grid.
    #[getset(get_copy = "pub", get_mut = "pub")]
    electricity_surplus: Option<ColorMap>, // unimplemented
    // /// A color filter based on sunlight received by the edge,
    // /// relative to the largest rendered brightness.
    // brightness: Option<ColorMap>,
    /// A color filter based on the percentage hitpoint of the edge,
    /// relative to the largest rendered percentage hitpoint.
    #[getset(get_copy = "pub", get_mut = "pub")]
    hitpoint:            Option<ColorMap>, // unimplemented

    /// Arguments for reflection rendering.
    #[getset(get = "pub", get_mut = "pub")]
    reflection: ReflectionArgs,
}

/// A trapezium function.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, derive_new::new, getset::CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Trapezium {
    /// The scalar value at which this channel starts increasing.
    min_start: f64,
    /// The scalar value at which this channel reaches the maximum.
    max_start: f64,
    /// The scalar value at which this channel starts decreasing.
    max_end:   f64,
    /// The scalar value at which this channel reaches zero.
    min_end:   f64,
    /// The maximum height of the trapezium.
    maximum:   f64,
}

impl Trapezium {
    /// Convert a scalar [0, 1] to the value for this trapezium.
    pub fn convert(&self, value: f64) -> f64 {
        if value <= self.min_start {
            0.
        } else if value <= self.max_start {
            (value - self.min_start) / (self.max_start - self.min_start) * self.maximum
        } else if value < self.max_end {
            self.maximum
        } else if value < self.min_end {
            (self.min_end - value) / (self.min_end - self.max_end) * self.maximum
        } else {
            0.
        }
    }
}

/// Arguments for the Phong reflection model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, derive_new::new, getset::CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ReflectionArgs {
    /// Phong ambient weight.
    ambient:       f64,
    /// Phong diffuse weight.
    diffuse:       f64,
    /// Phong specular weight.
    specular:      f64,
    /// Phong ambient coefficient.
    specular_coef: f64,
}

/// A function mapping a scalar [0, 1] to a color [0, 1]^3.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorMap {
    /// A constant color value.
    Monochrome(Vector),
    /// A linear color value.
    Linear(Vector, Vector),
    /// A colormap formed by three trapeziums.
    Trapeziums([Trapezium; 3]),
}

impl ColorMap {
    /// Convert a scalar [0, 1] to a color [0, 1]^3
    pub fn convert(&self, value: f64) -> Vector {
        match self {
            Self::Monochrome(color) => *color,
            Self::Linear(low, high) => lerp(*low, *high, value),
            Self::Trapeziums([r, g, b]) => {
                Vector::new(r.convert(value), g.convert(value), b.convert(value))
            }
        }
    }
}

/// Collects statistics to compute the color.
///
/// The `T` type argument is the components queried.
#[derive(derive_new::new)]
pub struct ColorMapCounter<T, F: Fn(T) -> f64> {
    color_map: ColorMap,
    /// The function to convert components to queried type.
    mapper:    F,
    #[new(default)]
    bounds:    Option<(f64, f64)>,
    #[new(default)]
    _ph:       PhantomData<fn(T) -> f64>,
}

impl<T, F: Fn(T) -> f64> ColorMapCounter<T, F> {
    /// Creates an instance, or return `None` if the filter should be unused, indicated by a `None` `color_map`.
    pub fn try_new(color_map: Option<ColorMap>, mapper: F) -> Option<Self> {
        color_map.map(|color_map| Self::new(color_map, mapper))
    }
}

/// A general trait backed by [`ColorMapCounter`] on different closure types,
/// implemented on `Option` to support unused filters.
pub trait ColorMapCount<T> {
    /// Proxies [`Option::is_some`].
    fn is_some(&self) -> bool;

    /// Feed the components into the counter, which forwards to the function for type counting.
    ///
    /// Do not call after using [`compute`].
    fn feed(&mut self, comps: T);

    /// Compute the color given the components, using the extrema collected in [`feed`].
    ///
    /// Only use after calling [`feed`] on all entities.
    fn compute(&self, comps: T) -> Vector;
}

impl<T, F: Fn(T) -> f64> ColorMapCount<T> for Option<ColorMapCounter<T, F>> {
    fn is_some(&self) -> bool { Option::is_some(self) }

    fn feed(&mut self, comps: T) {
        if let Some(this) = self {
            let value = (this.mapper)(comps);
            this.bounds = Some(match this.bounds {
                Some((min, max)) => (min.min(value), max.max(value)),
                None => (value, value),
            })
        }
    }

    fn compute(&self, comps: T) -> Vector {
        if let Some(this) = self {
            let (min, max) = this.bounds.expect("Call to compute() before feeding");

            let value = (this.mapper)(comps);
            let scale = (value - min) / (max - min);
            this.color_map.convert(scale)
        } else {
            Vector::new(1., 1., 1.) // the passthru color filter
        }
    }
}

impl Options {
    /// This method deliberately does not implement [`Default`]
    /// to avoid accidentally overwriting the default initialization.
    pub fn default() -> Self {
        Self {
            graphics: Graphics {
                render_stars:      true,
                render_reticle:    true,
                render_debug_info: true,
                node:              Node {
                    render:              true,
                    base:                Vector::new(1., 1., 1.),
                    texture:             Some(arcstr::literal!("fancy")),
                    cargo:               None,
                    liquid:              None,
                    gas:                 None,
                    electricity_usage:   None,
                    electricity_surplus: None,
                    brightness:          Some(ColorMap::Linear(
                        Vector::new(0.5, 0.5, 0.5),
                        Vector::new(1., 1., 1.),
                    )),
                    hitpoint:            None,
                },
                edge:              Edge {
                    render:              true,
                    base:                Vector::new(0.3, 0.5, 0.8),
                    cargo:               None,
                    liquid:              None,
                    gas:                 None,
                    electricity_current: None,
                    electricity_surplus: None,
                    hitpoint:            None,
                    reflection:          ReflectionArgs {
                        ambient:       0.3,
                        diffuse:       0.2,
                        specular:      1.,
                        specular_coef: 10.,
                    },
                },
            },
        }
    }
}

/// Sets up legion ECS.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let storage =
        storage::StorageService::new(storage::Area::Local).expect("Failed to fetch localStorage");
    let yew::format::Json(options) = storage.restore(STORAGE_KEY);
    let options: Options = options.unwrap_or_else(|_| Options::default());
    setup.resource(options)
}
