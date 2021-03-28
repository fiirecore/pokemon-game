use firecore_util::{Entity, Reset, Completable};
use macroquad::prelude::{Texture2D, draw_texture_ex, DrawTextureParams};

use crate::battle::transitions::{BattleTransition, BattleScreenTransition};
use crate::WIDTH_F32;

use crate::util::graphics::{byte_texture, draw_rect};

const DEF_RECT_WIDTH: f32 = -16.0;

pub struct TrainerBattleScreenTransition {

    active: bool,
    finished: bool,

    rect_width: f32,

    texture: Texture2D,

}

impl TrainerBattleScreenTransition { // To - do: Two grey flashes before rectangles scroll through screen

    pub fn new() -> Self {
        Self {
            active: false,
            finished: false,
            rect_width: DEF_RECT_WIDTH,
            texture: byte_texture(include_bytes!("../../../../build/assets/battle/encounter_ball.png")),
        }
    }

    fn draw_lines(&self, y: f32, invert: bool) {
        draw_rect(
            [0.0, 0.0, 0.0, 1.0], 
            if invert {
                WIDTH_F32 - self.rect_width
            } else {
                0.0
            }, 
            y, 
            self.rect_width, 
            32.0
        );
        draw_texture_ex(
            self.texture, 
            if invert {
                WIDTH_F32 - self.rect_width - 16.0
            } else {
                self.rect_width - 16.0
            }, 
            y, 
            macroquad::prelude::WHITE, 
            DrawTextureParams {
                rotation: (self.rect_width * 2.0).to_radians(),
                ..Default::default()
            },
        );
        
    }

}

impl BattleScreenTransition for TrainerBattleScreenTransition {}
impl BattleTransition for TrainerBattleScreenTransition {

    fn on_start(&mut self) {
        
    }

    fn update(&mut self, delta: f32) {
        self.rect_width += 240.0 * delta;
        if self.rect_width >= WIDTH_F32 + 16.0 {
            self.finished = true;
        }
    }

    fn render(&self) {
        self.draw_lines(0.0, false);
        self.draw_lines(32.0, true);
        self.draw_lines(64.0, false);
        self.draw_lines(96.0, true);
        self.draw_lines(128.0, false);
    }

}

impl Reset for TrainerBattleScreenTransition {

    fn reset(&mut self) {
        self.rect_width = DEF_RECT_WIDTH;
        self.finished = false;
    }

}

impl Completable for TrainerBattleScreenTransition {

    fn is_finished(&self) -> bool {
        return self.finished;
    } 

}

impl Entity for TrainerBattleScreenTransition {

    fn spawn(&mut self) {
        self.reset();
        self.active = true;
        self.finished = false;
    }

    fn despawn(&mut self) {
        self.active = false;
        self.finished = false;
    }

    fn is_alive(&self) -> bool {
        self.active
    }


}