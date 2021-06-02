use crate::{
    util::{
        Reset, 
        Completable,
    },
    storage::data,
    text::MessagePage,
    gui::DynamicText,
    graphics::draw_o_bottom,
    tetra::{
        Context,
        graphics::Texture,
    }
};

use crate::battle::{
    Battle,
    ui::transitions::{
        BattleIntroduction,
        introductions::basic::BasicBattleIntroduction,
    }
};

pub struct TrainerBattleIntroduction {

    introduction: BasicBattleIntroduction,

    texture: Option<Texture>,
    offset: f32,
    leaving: bool,

}

impl TrainerBattleIntroduction {

    const FINAL_TRAINER_OFFSET: f32 = 126.0;

    pub fn new(ctx: &mut Context) -> Self {
        Self {
            introduction: BasicBattleIntroduction::new(ctx),
            texture: None,
            offset: 0.0,
            leaving: false,
        }
    }

}

impl BattleIntroduction for TrainerBattleIntroduction {

    fn spawn(&mut self, battle: &Battle, text: &mut DynamicText) {
        text.clear();

        if let Some(trainer) = battle.data.trainer.as_ref() {
            self.texture = Some(trainer.texture.clone());

            let name = format!("{} {}", trainer.prefix, trainer.name);

            text.push(MessagePage::new(
                vec![
                    name.clone(), 
                    String::from("would like to battle!")
                ], 
                None
            ));

            text.push(MessagePage::new(
                vec![
                    name + " sent", 
                    format!("out {}", BasicBattleIntroduction::concatenate(&battle.opponent.active))
                ],
                Some(0.5),
            ));
        } else {
            text.push(MessagePage::new(
                vec![String::from("No trainer data found!")],
                None,
            ));
        }

        text.process_messages(data());

        self.introduction.common_setup(text, &battle.player.active);
        
    }

    fn update(&mut self, ctx: &mut Context, delta: f32, battle: &mut Battle, text: &mut DynamicText) {
        self.introduction.update(ctx, delta, battle, text);
        if text.can_continue() {
            if text.current() == text.len() - 2 {
                self.leaving = true;
            }           
        }
        if self.leaving && self.offset < Self::FINAL_TRAINER_OFFSET {
            self.offset += 300.0 * delta;
        }
    }

    fn draw(&self, ctx: &mut Context, battle: &Battle) {
        if self.offset < Self::FINAL_TRAINER_OFFSET {
            draw_o_bottom(ctx, self.texture.as_ref(), 144.0 + self.offset, 74.0);
        } else {
            self.introduction.draw_opponent(ctx, battle);
        }
        self.introduction.draw_player(ctx, battle);  
    }
}

impl Completable for TrainerBattleIntroduction {
    fn finished(&self) -> bool {
        self.introduction.finished()
    }
}

impl Reset for TrainerBattleIntroduction {
    fn reset(&mut self) {
        self.introduction.reset();
        self.offset = 0.0;
        self.leaving = false;
    }
}