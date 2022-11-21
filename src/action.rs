use bevy::prelude::*;
use geom::Coords;

use crate::draw::UpdateMap;
use crate::planet::*;
use crate::screen::CursorMode;
use crate::GameState;

#[derive(Clone, Copy, Debug)]
pub struct ActionPlugin;

#[derive(Clone, Copy, Debug)]
pub struct CursorAction {
    pub coords: Coords,
    pub drag: bool,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorAction>()
            .add_system_set(SystemSet::on_update(GameState::Running).with_system(cursor_action));
    }
}

fn cursor_action(
    mut er: EventReader<CursorAction>,
    mut update_map: ResMut<UpdateMap>,
    cursor_mode: Res<CursorMode>,
    params: Res<Params>,
    mut planet: ResMut<Planet>,
) {
    for e in er.iter() {
        let CursorAction { coords, .. } = *e;

        match *cursor_mode {
            CursorMode::Normal => (),
            CursorMode::EditBiome(biome) => {
                update_map.update();
                planet.edit_biome(coords, biome);
            }
            CursorMode::Build(kind) => match kind {
                StructureKind::None => (),
                _ => {
                    update_map.update();
                    let size = params.structures[&kind].size;
                    if planet.placeable(coords, size) {
                        planet.place(coords, size, new_structure(kind));
                    }
                }
            },
        }
    }
}

fn new_structure(kind: StructureKind) -> Structure {
    match kind {
        StructureKind::OxygenGenerator => Structure::OxygenGenerator,
        StructureKind::FertilizationPlant => Structure::FertilizationPlant,
        _ => unreachable!(),
    }
}