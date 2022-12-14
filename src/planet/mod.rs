mod action;
mod atmo;
mod buildings;
mod defs;
mod heat_transfer;
mod resources;
mod sim;

pub use self::atmo::Atmosphere;
pub use self::defs::*;
pub use self::resources::*;
pub use self::sim::Sim;
use fnv::FnvHashMap;
use geom::{Array2d, Coords};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::f32::consts::PI;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub structure: Structure,
    pub height: f32,
    pub biomass: f32,
    pub temp: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub buildable_structures: BTreeSet<StructureKind>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Rock,
            structure: Structure::None,
            height: 0.0,
            biomass: 0.0,
            temp: 300.0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub n: u32,
    pub enabled: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub days: u64,
    pub basics: PlanetBasics,
    pub player: Player,
    pub res: Resources,
    pub map: Array2d<Tile>,
    pub atmo: Atmosphere,
    pub orbit: FnvHashMap<OrbitalBuildingKind, Building>,
    pub star_system: FnvHashMap<StarSystemBuildingKind, Building>,
}

impl Planet {
    pub fn new(w: u32, h: u32, start_params: &StartParams) -> Planet {
        let map = Array2d::new(w, h, Tile::default());

        let mut planet = Planet {
            days: 0,
            basics: start_params.basics.clone(),
            player: Player::default(),
            res: Resources::new(start_params),
            map,
            atmo: Atmosphere::from_params(start_params),
            orbit: OrbitalBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
            star_system: StarSystemBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
        };

        for (kind, &n) in &start_params.orbital_buildings {
            let building = planet.orbit.get_mut(kind).unwrap();
            building.n = n;
            building.enabled = n;
        }
        for (kind, &n) in &start_params.star_system_buildings {
            let building = planet.star_system.get_mut(kind).unwrap();
            building.n = n;
            building.enabled = n;
        }

        planet
            .player
            .buildable_structures
            .insert(StructureKind::OxygenGenerator);
        planet
            .player
            .buildable_structures
            .insert(StructureKind::FertilizationPlant);
        planet
            .player
            .buildable_structures
            .insert(StructureKind::Heater);

        planet
    }

    pub fn advance(&mut self, sim: &mut Sim, params: &Params) {
        self.days += 1;

        self::buildings::advance(self, params);
        self::heat_transfer::advance(self, sim, params);

        atmo::sim_atmosphere(self, params);
    }

    pub fn calc_longitude_latitude<T: Into<Coords>>(&self, coords: T) -> (f32, f32) {
        let coords = coords.into();
        let (nx, ny) = self.map.size();

        let x = (2.0 * coords.0 as f32 + 1.0) / (2.0 * nx as f32); // 0.0 ~ 1.0
        let y = ((2.0 * coords.1 as f32 + 1.0) / (2.0 * ny as f32)) * 2.0 - 1.0; // -1.0 ~ 1.0

        let longtitude = x * 2.0 * PI; // 0.0 ~ 2pi
        let latitude = y.asin(); // -pi/2 ~ pi/2
        (longtitude, latitude)
    }
}
