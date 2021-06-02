use std::rc::Rc;

use crate::{
    util::{
        Entity, 
        Completable, 
        Direction,
        Coordinate,
    },
    storage::{
        data, data_mut, player::PlayerSave,
    },
    input::{
        down, pressed, Control,
        debug_pressed, DebugBind,
    },
    pokedex::{
        moves::FieldMoveId,
        item::{ItemStack, ItemId},
    },
    tetra::Context,
    log::info,
    battle_glue::BattleEntryRef,
    gui::{
        party::PartyGui,
        bag::BagGui,
    },
    state::GameStateAction,
    is_debug, keybind,
};

use worldlib::{
    serialized::SerializedWorld,
    map::{
        World,
        manager::{WorldMapManager, TryMoveResult, Door},
    },
    character::{
        MoveType,
        npc::npc_type::NPCType,
    },
};

use crate::world::{
    GameWorld, 
    WorldTextures,
    RenderCoords,
    gui::{
        TextWindow,
        StartMenu,
    },
    battle::random_wild_battle,
};

use super::warp::WarpTransition;

const PLAYER_MOVE_TIME: f32 = 0.12;

pub struct WorldManager {

    pub map_manager: WorldMapManager,

    textures: WorldTextures,

    warp_transition: WarpTransition,

    start_menu: StartMenu,
    text_window: TextWindow,

    // Other

    pub render_coords: RenderCoords,
    // noclip_toggle: bool,

    first_direction: Direction,
    player_move_accumulator: f32,

}

fn on_start(wm: &mut WorldMapManager, ctx: &mut Context, music: bool) {
    if let Some(map) = get_mut(wm) {
        map.on_start(ctx, music)
    }
}

fn update(wm: &mut WorldMapManager, ctx: &mut Context, delta: f32, battle: &mut Option<crate::battle_glue::BattleEntry>, text_window: &mut TextWindow) {

    if let Some(door) = &mut wm.data.door {
        match door.open {
            true => {
                if door.accumulator < Door::DOOR_MAX {
                    door.accumulator += delta * 6.0;
                    if door.accumulator >= Door::DOOR_MAX {
                        door.accumulator = Door::DOOR_MAX;
                    }
                }
            }
            false => {
                if door.accumulator > 0.0 {
                    door.accumulator -= delta * 6.0;
                    if door.accumulator <= 0.0 {
                        wm.data.door = None;
                    }
                }
            }
        }
    }

    if let Some(map) = match wm.data.current.as_ref() {
        Some(cur) => wm.maps.get_mut(cur),
        None => None,
    } {
        map.update(ctx, delta, &mut wm.data, battle, text_window);
    }
}

fn draw(wm: &WorldMapManager, ctx: &mut Context, textures: &WorldTextures, screen: &RenderCoords, border: bool) {
    if let Some(map) = wm.get() {
        match &map.chunk {
            Some(chunk) => {
                map.draw(ctx, textures, &wm.data.door, &screen.offset(chunk.coords), border);
                for map in chunk.connections.iter().flat_map(|id| wm.maps.get(id)) {
                    if let Some(chunk) = &map.chunk {
                        map.draw(ctx, textures, &None, &screen.offset(chunk.coords), false);
                    }
                }
            },
            None => map.draw(ctx, textures, &wm.data.door, screen, border),
        }
    }
}

fn get_mut(wm: &mut WorldMapManager) -> Option<&mut firecore_world_lib::map::WorldMap> {
    match wm.data.current.as_ref() {
        Some(cur) => wm.maps.get_mut(cur),
        None => None,
    }
}

impl WorldManager {

    pub fn new(ctx: &mut Context, party: Rc<PartyGui>, bag: Rc<BagGui>) -> Self {        
        Self {

            map_manager: WorldMapManager {
                maps: Default::default(),
                data: Default::default(),
            },

            textures: WorldTextures::new(ctx),

            warp_transition: WarpTransition::new(),
            start_menu: StartMenu::new(ctx, party, bag),
            text_window: TextWindow::new(ctx),
            first_direction: Direction::default(),
            render_coords: RenderCoords::default(),
            // noclip_toggle: false,
            player_move_accumulator: 0.0,
        }
    }

    pub fn load(&mut self, ctx: &mut Context, world: SerializedWorld) {

        self.textures.setup(ctx, world.textures, &world.npc_types);
        
        info!("Finished loading textures!");

        unsafe { crate::world::npc::NPC_TYPES = 
            Some(
                world.npc_types.into_iter().map(|npc_type| {
                    self.textures.npcs.add_npc_type(ctx, &npc_type);
                    (
                        npc_type.config.identifier,
                        NPCType {
                            sprite: firecore_world_lib::character::sprite::SpriteIndexes::from_index(npc_type.config.sprite),
                            text_color: npc_type.config.text_color,
                            trainer: npc_type.config.trainer,
                        }
                    )
                }
                ).collect()
            ); 
        }

        self.map_manager = world.manager;
    
    }

    pub fn load_with_data(&mut self) {
        self.load_player(data());
    }

    pub fn on_start(&mut self, ctx: &mut Context, battle: BattleEntryRef) {
        self.map_start(ctx, true);
        on_tile(&mut self.map_manager, &mut self.textures, battle);
    }

    pub fn map_start(&mut self, ctx: &mut Context, music: bool) {
        on_start(&mut self.map_manager, ctx, music);
    }

    pub fn update(&mut self, ctx: &mut Context, delta: f32, battle: BattleEntryRef, action: &mut Option<GameStateAction>) {

        if self.start_menu.alive() {
            self.start_menu.update(ctx, delta, action);
        } else {

            if pressed(ctx, Control::Start) {
                self.start_menu.spawn();
            }

            if is_debug() {
                self.debug_input(ctx, battle)
            }

            if !(!self.map_manager.data.player.character.position.offset.is_zero() || self.map_manager.data.player.is_frozen() ) {

                if down(ctx, Control::B) {
                    if self.map_manager.data.player.character.move_type == MoveType::Walking {
                        self.map_manager.data.player.character.move_type = MoveType::Running;
                    }
                } else if self.map_manager.data.player.character.move_type == MoveType::Running {
                    self.map_manager.data.player.character.move_type = MoveType::Walking;
                }

                const SURF: FieldMoveId = unsafe { FieldMoveId::new_unchecked(1718777203) };

                if down(ctx, crate::keybind(self.first_direction)) {
                    if self.player_move_accumulator > PLAYER_MOVE_TIME {
                        if let Some(result) = self.map_manager.try_move(self.first_direction, delta) {
                            match result {
                                TryMoveResult::MapUpdate => self.map_start(ctx, true),
                                TryMoveResult::TrySwim => {
                                    for id in data().party.iter().map(|pokemon| pokemon.moves.iter().flat_map(|instance| &instance.move_ref.value().field_id)).flatten() {
                                        if id == &SURF {
                                            self.map_manager.data.player.character.move_type = MoveType::Swimming;
                                            self.map_manager.try_move(self.first_direction, delta);
                                            break;
                                        }
                                    }
                                    // self.text_window.set_text(firecore_game::text::Message::new(firecore_game::text::TextColor::Black, vec![]));
                                }
                            }
                        }
                    } else {
                        self.player_move_accumulator += delta;
                    }
                } else {
                    let mut movdir: Option<Direction> = None;
                    for direction in &Direction::DIRECTIONS {
                        let direction = *direction;
                        if down(ctx, keybind(direction)) {
                            movdir = if let Some(dir) = movdir {
                                if dir.inverse() == direction {
                                    None
                                } else {
                                    Some(direction)
                                }
                            } else {
                                Some(direction)
                            };
                        }                        
                    }
                    if let Some(direction) = movdir {
                        self.map_manager.data.player.character.position.direction = direction;
                        self.first_direction = direction;
                    } else {
                        self.player_move_accumulator = 0.0;
                        self.map_manager.data.player.character.moving = false;
                    }
                }
            }

            // Update

            self.textures.tiles.update(delta);
            self.textures.player.update(delta, &mut self.map_manager.data.player.character);
    
           update(&mut self.map_manager, ctx, delta, battle, &mut self.text_window);
    
            if self.warp_transition.alive() {
                self.warp_transition.update(delta);
                if self.warp_transition.switch() {
                    if let Some(destination) = self.map_manager.data.warp.clone() {
                        self.textures.player.draw = !destination.transition.move_on_exit;
                        let change_music = destination.transition.change_music;
                        self.map_manager.warp(destination);
                        self.map_start(ctx, change_music);
                        on_tile(&mut self.map_manager, &mut self.textures, battle);
                    }
                }
                if self.warp_transition.finished() {
                    self.textures.player.draw = true;
                    self.warp_transition.despawn();
                    self.map_manager.data.player.unfreeze();
                    if let Some(destination) = self.map_manager.data.warp.take() {
                        if destination.transition.move_on_exit {
                            self.map_manager.try_move(destination.position.direction.unwrap_or(self.map_manager.data.player.character.position.direction), delta);
                        }
                    }
                    
                }
            } else {
                if let Some(warp) = &self.map_manager.data.warp {
                    if if let Some(door) = &self.map_manager.data.door {
                        door.accumulator == 0.0 && !door.open
                    } else {
                        true
                    } || !warp.transition.warp_on_tile {
                        self.warp_transition.spawn();                   
                        self.map_manager.data.player.freeze_input();
                    }
                }
            }
    
            if !self.map_manager.data.player.is_frozen() {
                if self.map_manager.do_move(delta) {
                    self.stop_player(battle);
                }
            }

            let offset = match self.map_manager.get().map(|map| map.chunk.as_ref()).flatten() {
                Some(chunk) => chunk.coords,
                None => Coordinate::ZERO,
            };
    
            self.render_coords = RenderCoords::new(offset, &self.map_manager.data.player.character);

        }
        
    }

    pub fn draw(&self, ctx: &mut Context) {
        draw(&self.map_manager, ctx, &self.textures, &self.render_coords, true);
        self.textures.player.draw(ctx, &self.map_manager.data.player.character);

        let offset = match self.map_manager.get().map(|map| map.chunk.as_ref()).flatten() {
            Some(chunk) => chunk.coords,
            None => Coordinate::ZERO,
        };

        let screen = self.render_coords.offset(offset);

        self.textures.player.bush.draw(ctx, &screen);

        self.text_window.draw(ctx);
        self.start_menu.draw(ctx); 
        self.warp_transition.draw(ctx);
    }

    pub fn save_data(&self, save: &mut PlayerSave) {
        use crate::storage::player;
        save.location = self.map_manager.data.current.unwrap_or(player::default_location());
		save.character = self.map_manager.data.player.character.clone();
    }

    fn stop_player(&mut self, battle: BattleEntryRef) {

        self.map_manager.data.player.character.stop_move();

        if let Some(destination) = self.map_manager.warp_at(self.map_manager.data.player.character.position.coords) { // Warping does not trigger tile actions!
            self.map_manager.data.warp = Some(*destination);
        } else if self.map_manager.in_bounds(self.map_manager.data.player.character.position.coords) {
            on_tile(&mut self.map_manager, &mut self.textures, battle);
        }

        if let Some(door) = &mut self.map_manager.data.door {
            if self.map_manager.data.warp.is_some() {
                self.textures.player.draw = false;
            }
            door.open = false;
        }

    }

    pub fn load_player(&mut self, data: &PlayerSave) {

        self.map_manager.data.player.character = data.character.clone();

        self.map_manager.data.current = Some(data.location);

        // if let Some(map) = data.location.map {
        //     self.map_manager.chunk_active = false;
        //     self.map_manager.update_map_set(map, data.location.index);
        // } else {
        //     self.map_manager.chunk_active = true;
        //     self.map_manager.update_chunk(data.location.index);
        // }     
    }

    fn debug_input(&mut self, ctx: &Context, battle: BattleEntryRef) {

        if debug_pressed(ctx, DebugBind::F1) {
            random_wild_battle(battle);
        }

        if debug_pressed(ctx, DebugBind::F2) {
            // self.noclip_toggle = true;
            self.map_manager.data.player.character.noclip = !self.map_manager.data.player.character.noclip;
        }

        if debug_pressed(ctx, DebugBind::F3) {

            info!("Local Coordinates: {}", self.map_manager.data.player.character.position.coords);
            // info!("Global Coordinates: ({}, {})", self.map_manager.player.position.get_x(), self.map_manager.player.position.get_y());

            if let Some(tile) = self.map_manager.tile(self.map_manager.data.player.character.position.coords) {
                info!("Current Tile ID: {:x}", tile);
            } else {
                info!("Currently out of bounds");
            }

            info!("Player is {:?}", self.map_manager.data.player.character.move_type);
            
        }

        if debug_pressed(ctx, DebugBind::F4) {
            for (slot, instance) in data().party.iter().enumerate() {
                info!("Party Slot {}: Lv{} {}", slot, instance.level, instance.name());
            }
        }

        if debug_pressed(ctx, DebugBind::F5) {
            if let Some(map) = self.map_manager.get() {
                info!("Resetting battled trainers in this map! ({})", map.name);
                data_mut().world.get_map(&map.id).battled.clear();
            }
        }

        if debug_pressed(ctx, DebugBind::F6) {
            info!("Clearing used scripts in player data!");
            data_mut().world.scripts.clear();
        }

        if debug_pressed(ctx, DebugBind::F7) {
            self.map_manager.data.player.character.freeze();
            self.map_manager.data.player.character.unfreeze();
            self.map_manager.data.player.character.noclip = true;
            info!("Unfroze player!");
        }

        // F8 in use
        
        if debug_pressed(ctx, DebugBind::F9) {
            use std::sync::atomic::Ordering::Relaxed;
            let wild = !super::WILD_ENCOUNTERS.load(Relaxed);
            super::WILD_ENCOUNTERS.store(wild, Relaxed);
            info!("Wild Encounters: {}", wild);
        }

        if debug_pressed(ctx, DebugBind::H) {
            data_mut().party.iter_mut().for_each(|pokemon| {
                pokemon.heal();
            });
        }

        if debug_pressed(ctx, DebugBind::B) {
            data_mut().bag.add_item(ItemStack::new(&"pokeball".parse::<ItemId>().unwrap(), 50));
        }
        
    }
    
}

fn on_tile(wm: &mut WorldMapManager, textures: &mut WorldTextures, battle: BattleEntryRef) {
    textures.player.bush.in_bush = wm.tile(wm.data.player.character.position.coords) == Some(0x0D);
    if textures.player.bush.in_bush {
        textures.player.bush.add(wm.data.player.character.position.coords);
    }
    if let Some(map) = match wm.data.current.as_ref() {
        Some(cur) => wm.maps.get_mut(cur),
        None => None,
    } {
        map.on_tile(&mut wm.data, battle)
    }
}