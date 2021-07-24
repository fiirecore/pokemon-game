use crate::{
    engine::{gui::TextDisplay, tetra::Context, util::Completable},
    game::battle_glue::BattleTrainerEntry,
    pokedex::trainer::{TrainerData, TrainerId},
};

pub mod managers;

pub mod closers;
pub mod transitions;

pub(crate) trait BattleTransition: Completable {
    fn update(&mut self, ctx: &mut Context, delta: f32);

    fn draw(&self, ctx: &mut Context);

    // fn render_below_player(&self);
}

pub(crate) trait BattleCloser: Completable {
    fn spawn(
        &mut self,
        winner: Option<&TrainerId>,
        trainer: Option<&TrainerData>,
        trainer_entry: Option<&BattleTrainerEntry>,
        text: &mut TextDisplay,
    );

    fn update(&mut self, ctx: &mut Context, delta: f32, text: &mut TextDisplay);

    fn draw(&self, ctx: &mut Context);

    fn draw_battle(&self, ctx: &mut Context);

    fn world_active(&self) -> bool;
}