use crate::gfx::texture::TexAtlas;
use font_kit::{
	canvas::{Canvas, Format, RasterizationOptions},
	hinting::HintingOptions,
	loaders::default::Font as KitFont,
	source::SystemSource,
};
use pathfinder_geometry::{
	transform2d::Transform2F,
	vector::{Vector2F, Vector2I},
};
use std::sync::Arc;
use vulkan::{command::CommandPool, device::Queue};

pub struct Font {
	font: KitFont,
	atlas: TexAtlas,
}
impl Font {
	pub fn new(queue: Arc<Queue>, pool: Arc<CommandPool>) -> Self {
		let font = SystemSource::new().select_by_postscript_name("ArialMT").unwrap().load().unwrap();
		Self { font, atlas: TexAtlas::new(queue, pool) }
	}

	pub fn load_char(&mut self, ch: char) {
		let mut canvas = Canvas::new(Vector2I::splat(32), Format::A8);

		let glyph = self.font.glyph_for_char(ch).unwrap();
		let transform = Transform2F::from_translation(Vector2F::new(0.0, 32.0));
		let rasterization = RasterizationOptions::GrayscaleAa;
		let hinting = HintingOptions::None;
		self.font.rasterize_glyph(&mut canvas, glyph, 32.0, transform, hinting, rasterization).unwrap();

		let (tex, future) = self.atlas.alloc(32, 32);
	}
}
