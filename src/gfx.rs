pub mod gui;
pub mod texture;
pub mod window;

use crate::fs::{read_all_u32, read_all_u8};
use gui::font::Font;
use memoffset::offset_of;
use nalgebra::Vector2;
use std::{iter::once, mem::size_of, sync::Arc};
use typenum::{B0, B1};
use vulkan::{
	buffer::Buffer,
	command::CommandPool,
	descriptor::{DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorType},
	device::{BufferUsageFlags, Device, Queue},
	image::{
		Format, Image, ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageType, ImageUsageFlags, ImageView,
		Sampler,
	},
	instance::{Instance, Version},
	physical_device::PhysicalDevice,
	pipeline::{PipelineLayout, VertexDesc, VertexInputAttributeDescription},
	shader::{ShaderModule, ShaderStageFlags},
	sync::GpuFuture,
	Vulkan,
};

pub struct Gfx {
	instance: Arc<Instance>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	layout: Arc<PipelineLayout>,
	verts: Arc<Buffer<[TriangleVertex]>>,
	desc_set: Arc<DescriptorSet>,
	vshader: Arc<ShaderModule>,
	fshader: Arc<ShaderModule>,
}
impl Gfx {
	pub async fn new() -> Arc<Self> {
		// start reading files now to use later
		let img = read_all_u8("assets/colors.png");
		let vert_spv = read_all_u32("build/shader.vert.spv");
		let frag_spv = read_all_u32("build/shader.frag.spv");

		let vulkan = Vulkan::new().unwrap();

		let name = env!("CARGO_PKG_NAME");
		let version = Version::new(
			env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
			env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
			env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
		);
		let instance = Instance::new(vulkan, name, version);

		let (device, mut queue) = {
			let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

			let queue_family = physical_device
				.get_queue_family_properties()
				.filter(|props| props.queue_flags().graphics())
				.next()
				.unwrap()
				.family();

			let (device, mut queues) = Device::new(physical_device, vec![(queue_family, &[1.0][..])]);
			(device, queues.next().unwrap())
		};

		let cmdpool = CommandPool::new(device.clone(), queue.family().clone(), true);

		let img = image::load_from_memory(&img.await.unwrap()).unwrap().into_rgba();
		let (width, height) = (img.width(), img.height());
		let data: Vec<_> = img.pixels().cloned().collect();
		let pixels = Buffer::init_slice(device.clone(), data.len() as _, B1, BufferUsageFlags::TRANSFER_SRC)
			.copy_from_slice(&data);
		let format = Format::R8G8B8A8_UNORM;
		let usage = ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED;
		let (image, image_future) = Image::init(device.clone(), ImageType::TYPE_2D, width, height, 1, format, usage)
			.copy_from_buffer(&queue, &cmdpool, pixels);
		let subresource =
			ImageSubresourceRange::builder().aspect_mask(ImageAspectFlags::COLOR).level_count(1).layer_count(1).build();
		let image_view = ImageView::new(image, format, subresource);

		let data = [
			TriangleVertex { pos: [0.0, 0.0].into() },
			TriangleVertex { pos: [1.0, 0.0].into() },
			TriangleVertex { pos: [1.0, 1.0].into() },
			TriangleVertex { pos: [1.0, 1.0].into() },
			TriangleVertex { pos: [0.0, 1.0].into() },
			TriangleVertex { pos: [0.0, 0.0].into() },
		];
		let verts = Buffer::init_slice(device.clone(), data.len() as _, B1, BufferUsageFlags::TRANSFER_SRC)
			.copy_from_slice(&data);
		let (verts, verts_future) = Buffer::init_slice(
			device.clone(),
			data.len() as _,
			B0,
			BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
		)
		.copy_from_buffer(&mut queue, &cmdpool, verts);

		let future = image_future.join(verts_future).then_signal_fence();

		let sampler = Sampler::new(device.clone());

		let desc_layout = DescriptorSetLayout::builder(device.clone())
			.desc(
				DescriptorType::COMBINED_IMAGE_SAMPLER,
				1,
				ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX,
				once(sampler),
			)
			.build();

		let layout = PipelineLayout::new(
			device.clone(),
			vec![desc_layout.clone()],
			once((ShaderStageFlags::VERTEX, 0, size_of::<[Vector2<f32>; 2]>() as _)),
		);

		let desc_pool =
			DescriptorPool::new(device.clone(), 1, vec![(DescriptorType::COMBINED_IMAGE_SAMPLER, 1).into()]);

		let desc_set = DescriptorSet::alloc(desc_pool, vec![desc_layout]).next().unwrap();
		DescriptorSet::update_builder(&device)
			.write(
				&desc_set,
				0,
				DescriptorType::COMBINED_IMAGE_SAMPLER,
				once((None, image_view, ImageLayout::SHADER_READ_ONLY_OPTIMAL)),
			)
			.submit();

		let _font = Font::new(queue.clone(), cmdpool.clone());

		future.wait();

		let vshader = unsafe { ShaderModule::new(device.clone(), &vert_spv.await.unwrap()) };
		let fshader = unsafe { ShaderModule::new(device.clone(), &frag_spv.await.unwrap()) };

		Arc::new(Self { instance, device, queue, layout, verts, desc_set, vshader, fshader })
	}
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TriangleVertex {
	pub pos: Vector2<f32>,
}
impl VertexDesc for TriangleVertex {
	fn attribute_descs() -> Vec<VertexInputAttributeDescription> {
		vec![
			VertexInputAttributeDescription::builder()
				.binding(0)
				.location(0)
				.format(Format::R32G32_SFLOAT)
				.offset(offset_of!(Self, pos) as _)
				.build(),
		]
	}
}
