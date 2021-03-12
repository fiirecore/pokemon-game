use firecore_util::text::TextColor;
use super::GuiText;

pub struct StaticText {
	
	alive: bool,
	x: f32,
	y: f32,
	panel_x: f32,
	panel_y: f32,

	pub text: Vec<String>,
	pub color: TextColor,
	pub font_id: usize,

	direction: bool,
	
}

impl StaticText {
	
	pub fn new(text: Vec<String>, text_color: TextColor, font_id: usize, from_right: bool, x: f32, y: f32, panel_x: f32, panel_y: f32) -> Self {
		
		Self {
			
			alive: false,
			x: x,
			y: y,
			panel_x: panel_x,
			panel_y: panel_y,

			text: text,
			color: text_color,
			font_id: font_id,

			direction: from_right,
			
		}
		
	}
	
}

impl super::GuiComponent for StaticText {
	
	fn update_position(&mut self, x: f32, y: f32) {
		self.panel_x = x;
		self.panel_y = y;
	}
	
	fn render(&self) {
		for line_index in 0..self.get_text().len() {
			if self.direction {
				crate::util::graphics::draw_text_right_color(self.get_font_id(), self.get_line(line_index), self.color, self.panel_x + self.x, self.panel_y + self.y + (line_index << 4) as f32);
			} else {
				crate::util::graphics::draw_text_left_color(self.get_font_id(), self.get_line(line_index), self.color, self.panel_x + self.x, self.panel_y + self.y + (line_index << 4) as f32);
			}
		}		
	}
	
}

impl GuiText for StaticText {
	
	fn get_line(&self, index: usize) -> &String {
		&self.get_text()[index]
	}

	fn get_text(&self) -> &Vec<String> {
		&self.text
	}

	fn get_font_id(&self) -> usize {
		self.font_id
	}
	
}

impl firecore_util::Entity for StaticText {

	fn spawn(&mut self) {
		self.alive = true;		
	}
	
	fn despawn(&mut self) {
		self.alive = false;
	}
	
	fn is_alive(& self) -> bool {
		self.alive
	}

}