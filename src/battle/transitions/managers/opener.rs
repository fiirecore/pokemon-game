use firecore_util::{Entity, Reset, Completable, battle::BattleType};

use crate::battle::{
    Battle,
    transitions::{
        BattleOpener, 
        BattleTransition,
        openers::{
            Openers,
            trainer::TrainerBattleOpener,
            wild::WildBattleOpener,
        },
        managers::introduction::BattleIntroductionManager
    }
};

pub struct BattleOpenerManager {

    alive: bool,

    current_opener: Openers,
    wild: WildBattleOpener,
    trainer: TrainerBattleOpener,

    pub introduction: BattleIntroductionManager,
}

impl BattleOpenerManager {
    pub fn new() -> Self {
        Self {

            alive: false,

            current_opener: Openers::default(),
            wild: WildBattleOpener::new(),
            trainer: TrainerBattleOpener::new(),

            introduction: BattleIntroductionManager::new(),

        }
    }

    pub fn render_below_panel(&self, battle: &Battle) {
        self.introduction.render_with_offset(battle, self.offset());
        self.get().render_below_panel();
    }

    pub fn spawn_type(&mut self, battle_type: BattleType) {
        self.current_opener = match battle_type {
            BattleType::Wild => Openers::Wild,
            BattleType::Trainer => Openers::Trainer,
            BattleType::GymLeader => Openers::Trainer,
        };
        self.introduction.spawn_type(&self.current_opener);
        self.spawn();
    }

    fn get(&self) -> &dyn BattleOpener {
        match self.current_opener {
            Openers::Wild => &self.wild,
            Openers::Trainer => &self.trainer,
        }
    }

    fn get_mut(&mut self) -> &mut dyn BattleOpener {
        match self.current_opener {
            Openers::Wild => &mut self.wild,
            Openers::Trainer => &mut self.trainer,
        }
    }

}

impl BattleTransition for BattleOpenerManager {

    fn on_start(&mut self) {
        self.get_mut().on_start();
    }

    fn update(&mut self, delta: f32) {
        if self.is_alive() {
            let opener = self.get_mut();
            if opener.is_alive() {
                if opener.is_finished() {
                    opener.despawn();
                    self.introduction.spawn();
                    self.introduction.on_start();
                } else {
                    opener.update(delta
                        // * if macroquad::prelude::is_key_down(macroquad::prelude::KeyCode::Space) {
                        //     8.0
                        // } else {
                        //     1.0
                        // }
                    );
                }
            } else if self.introduction.is_alive() {
                self.introduction.update(delta
                    //  * if macroquad::prelude::is_key_down(macroquad::prelude::KeyCode::Space) {
                    //     8.0
                    // } else {
                    //     1.0
                    // }
                );
            }
        }
    }

    fn render(&self) {
        if self.is_alive() {
            if self.introduction.is_alive() {
                self.introduction.render();
            } else {
                self.get().render();
            }
        }
    }

}

impl BattleOpener for BattleOpenerManager {

    fn offset(&self) -> f32 {
        return self.get().offset();
    }

    fn render_below_panel(&self) {
        macroquad::prelude::warn!("Using wrong render below panel method!");
    }
}

impl Reset for BattleOpenerManager {

    fn reset(&mut self) {
        self.get_mut().reset();
        self.introduction.reset();
    }

}

impl Completable for BattleOpenerManager {

    fn is_finished(&self) -> bool {
        return self.introduction.is_finished();
    }
    
}

impl Entity for BattleOpenerManager {
    fn spawn(&mut self) {
        self.alive = true;
        self.get_mut().spawn();
        self.reset();
    }

    fn despawn(&mut self) {
        self.alive = false;
        self.get_mut().despawn();
        self.introduction.despawn();
        self.reset();
    }

    fn is_alive(&self) -> bool {
        return self.alive;
    }
}
