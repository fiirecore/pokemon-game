use serde::{Deserialize, Serialize};
use deps::{
    tinystr::TinyStr16,
    hash::HashMap,
};
use util::{
    Coordinate,
};
use firecore_audio_lib::music::MusicId;

use crate::MapSize;
use crate::MovementId;
use crate::TileId;

use crate::character::npc::{NPCId, NPC};
use crate::script::world::WorldScript;

use wild::WildEntry;
use warp::{WarpMap, WarpDestination};

pub mod set;
pub mod chunk;
pub mod manager;

pub mod warp;
pub mod wild;
// pub mod object;

pub type MapIdentifier = TinyStr16;
pub type NPCMap = HashMap<NPCId, NPC>;

pub trait World {

    fn in_bounds(&self, coords: Coordinate) -> bool;

    fn tile(&self, coords: Coordinate) -> Option<TileId>;

    fn walkable(&self, coords: Coordinate) -> MovementId; // not an option because can return 1

    fn check_warp(&self, coords: Coordinate) -> Option<WarpDestination>;

}

#[derive(Serialize, Deserialize)]
pub struct WorldMap {

    pub id: MapIdentifier,

    pub name: String,
    pub music: MusicId,

    pub width: MapSize,
    pub height: MapSize,

    pub palettes: [u8; 2],

    pub tiles: Vec<TileId>,
    pub movements: Vec<MovementId>,

    pub border: [TileId; 4],//Border, // border blocks

    // Map objects

    pub warps: WarpMap,

    pub wild: Option<WildEntry>,
    
    pub npcs: NPCMap,

    // pub objects: HashMap<u8, MapObject>,

    pub scripts: Vec<WorldScript>,

    #[serde(skip)]
    pub state: WorldMapState,

}

#[derive(Default)]
pub struct WorldMapState {

    pub npc: Option<NPCId>,

}

impl World for WorldMap {

    fn in_bounds(&self, coords: Coordinate) -> bool {
        return !(coords.x.is_negative() || coords.x >= self.width as i32 || coords.y.is_negative() || coords.y >= self.height as i32);
    }

    fn tile(&self, coords: Coordinate) -> Option<TileId> {
        if self.in_bounds(coords) {
            Some(self.tiles[coords.x as usize + coords.y as usize * self.width])
        } else {
            None
        }        
    }

    fn walkable(&self, coords: Coordinate) -> MovementId {
        for npc in self.npcs.values() {
            if /*npc.is_alive() &&*/ npc.character.position.coords == coords {
                return 1;
            }
        }
        self.movements[coords.x as usize + coords.y as usize * self.width]
    }

    fn check_warp(&self, coords: Coordinate) -> Option<WarpDestination> {
        for warp in self.warps.values() {
            if warp.location.in_bounds(&coords) {
                return Some(warp.destination.clone());
            }
        }
        None
    }

}

// #[derive(Default, Serialize, Deserialize)]
// pub struct Border {

//     pub tiles: Vec<TileId>,
//     pub size: u8, // length or width (border is a square)

// }