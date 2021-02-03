use crate::entity::Entity;
use crate::gui::Activatable;
use crate::gui::GuiComponent;
use crate::gui::background::Background;
use crate::gui::dynamic_text::DynamicText;
use crate::io::data::text::Message;
use crate::util::Completable;
use crate::util::Reset;

pub struct MapWindowManager {

    alive: bool,
    background: Background,
    text: DynamicText,

}

impl MapWindowManager {

    pub fn new() -> MapWindowManager {
        let panel_x = 6.0;
        let panel_y = 116.0;
        MapWindowManager {
            alive: false,
            background: Background::new(crate::util::graphics::texture::byte_texture(include_bytes!("../../../build/assets/gui/message.png")), panel_x, panel_y),
            text: DynamicText::new(11.0, 5.0, panel_x, panel_y),
        }
    }

    pub fn set_text(&mut self, message: Vec<Message>) {
        self.text.text = message;
    }

    pub fn update(&mut self, delta: f32) {
        if self.is_alive() {
            self.text.update(delta);
        }
    }

    pub fn render(&self) {
        if self.is_alive() {
            self.background.render();
            self.text.render();
        }
    }

    pub fn input(&mut self, delta: f32) {
        self.text.input(delta);
    }

}

impl Default for MapWindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Entity for MapWindowManager {
    fn spawn(&mut self) {
        self.alive = true;
        self.reset();
        self.text.enable();
        self.text.focus();
    }

    fn despawn(&mut self) {
        self.alive = false;
        self.text.disable();
    }

    fn is_alive(&self) -> bool {
        self.alive
    }
}

impl Reset for MapWindowManager {
    fn reset(&mut self) {
        self.text.reset();
    }
}

impl Completable for MapWindowManager {

    fn is_finished(&self) -> bool {
        self.text.is_finished()
    }

}