use super::*;
use std::f32::consts::PI;

/// Holds data for simulation
pub struct Sim {
    /// The number of tiles
    pub n_tile: u32,
    /// Tile area [m^2]
    pub tile_area: f32,
    /// Atmosphere temprature
    pub atemp: Array2d<f32>,
    /// Atmosphere temprature (used for calculation)
    pub atemp_new: Array2d<f32>,
    /// Atmosphere heat capacity [J/K]
    pub atmo_heat_cap: Array2d<f32>,
    /// Tile albedo
    pub albedo: Array2d<f32>,
}

impl Sim {
    pub fn new(planet: &Planet) -> Self {
        let size = planet.map.size();
        let tile_area = 4.0 * PI * planet.basics.radius * planet.basics.radius
            / (size.0 as f32 * size.1 as f32);

        Sim {
            n_tile: size.0 * size.1,
            tile_area,
            atemp: Array2d::new(size.0, size.1, 0.0),
            atemp_new: Array2d::new(size.0, size.1, 0.0),
            atmo_heat_cap: Array2d::new(size.0, size.1, 0.0),
            albedo: Array2d::new(size.0, size.1, 0.0),
        }
    }
}
