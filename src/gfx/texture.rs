use std::sync::Arc;
use vulkan::{
	command::CommandPool,
	device::Queue,
	image::{
		ClearColorValue, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageType, ImageUsageFlags, ImageView,
	},
	sync::{GpuFuture, NowFuture},
};

pub struct TexAtlas {
	queue: Arc<Queue>,
	pool: Arc<CommandPool>,
	images: Vec<(Arc<ImageView>, Vec<Rect>)>,
}
impl TexAtlas {
	pub fn new(queue: Arc<Queue>, pool: Arc<CommandPool>) -> Self {
		Self { queue, pool, images: vec![] }
	}

	pub fn alloc(&mut self, w: u32, h: u32) -> (Subtex, Box<dyn GpuFuture>) {
		let mut target = None;
		'outer: for (image_view, rects) in &mut self.images {
			for i in (0..rects.len()).rev() {
				if rects[i].w > w && rects[i].h > h {
					let rect = rects.remove(i);
					rects.extend(rect.sub_corner(w, h));
					target = Some((image_view.clone(), Rect::new(rect.x, rect.y, w, h)));
					break 'outer;
				}
			}
		}

		let (image_view, rect, future) = if let Some((image_view, rect)) = target {
			(image_view, rect, Box::new(NowFuture::new(self.queue.device().clone())) as Box<dyn GpuFuture>)
		} else {
			let format = Format::R8G8B8A8_UNORM;
			let (image, future) = Image::init(
				self.queue.device().clone(),
				ImageType::TYPE_2D,
				2048,
				2048,
				1,
				format,
				ImageUsageFlags::SAMPLED,
			)
			.clear(&self.queue, &self.pool, ClearColorValue { uint32: [0, 0, 0, 0] });

			let subresource = ImageSubresourceRange::builder()
				.aspect_mask(ImageAspectFlags::COLOR)
				.level_count(1)
				.layer_count(1)
				.build();
			let image_view = ImageView::new(image, format, subresource);

			let rect = Rect::new(0, 0, w, h);
			let rects = rect.sub_corner(w, h);
			self.images.push((image_view.clone(), rects));
			(image_view, rect, Box::new(future) as _)
		};

		(Subtex { image_view, rect }, future)
	}
}

pub struct Subtex {
	image_view: Arc<ImageView>,
	rect: Rect,
}

// TODO: move to a math module?
struct Rect {
	x: u32,
	y: u32,
	w: u32,
	h: u32,
}
impl Rect {
	fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
		Self { x, y, w, h }
	}

	fn zero() -> Self {
		Self { x: 0, y: 0, w: 0, h: 0 }
	}

	// TODO: maybe replace with a generator function once they're stable, to avoid heap allocation
	fn sub(&self, rhs: &Rect) -> Vec<Rect> {
		let mut ret = vec![];

		let self_right = self.right();
		let self_bottom = self.bottom();
		let rhs_right = rhs.right();
		let rhs_bottom = rhs.bottom();

		if rhs.x > self.x {
			ret.push(Rect::new(self.x, self.y, rhs.x - self.x, self.h));
		}
		if rhs_right < self_right {
			ret.push(Rect::new(rhs_right, self.y, self_right - rhs_right, self.h));
		}
		if rhs.y > self.y {
			ret.push(Rect::new(self.x, self.y, self.w, rhs.y - self.y));
		}
		if rhs_bottom < self_bottom {
			ret.push(Rect::new(self.x, rhs_bottom, self.w, self_bottom - rhs_bottom));
		}

		ret
	}

	// TODO: maybe replace with a generator function once they're stable, to avoid heap allocation
	fn sub_corner(&self, w: u32, h: u32) -> Vec<Rect> {
		let mut ret = vec![];

		if w < self.w {
			ret.push(Rect::new(self.x + w, self.y, self.w - w, self.h));
		}
		if h < self.h {
			ret.push(Rect::new(self.x, self.y + h, self.w, self.h - h));
		}

		ret
	}

	fn right(&self) -> u32 {
		self.x + self.w
	}

	fn bottom(&self) -> u32 {
		self.y + self.h
	}
}
