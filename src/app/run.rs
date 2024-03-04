use std::sync::Arc;

use qubicon_vulkan::{commands::command_buffers::{command_buffer_builder::{barrier::{AccessFlags, ImageMemoryBarrier, PipelineBarrierDependencyFlags}, copy::BufferCopy, PipelineBindPoint}, CommandBufferUsageFlags}, descriptors::alloc::descriptor_set::{BufferWriteInfo, DescriptorSet, DescriptorWrite, ImageWriteInfo}, instance::physical_device::memory_properties::MemoryTypeProperties, memory::{alloc::{hollow_device_memory_allocator::HollowDeviceMemoryAllocator, standart_device_memory_allocator::StandartMemoryAllocator}, resources::{buffer::{Buffer, BufferCreateInfo, BufferUsageFlags}, image::ImageLayout, image_view::{ImageAspect, ImageSubresourceRange, ImageViewCreateInfo, ImageViewType}}, BufferRequest, BufferStagingBufferInfo}, queue::{PresentInfo, PresentInfoSwapchainEntry}, shaders::PipelineStageFlags, swapchain::AcquireImageSyncPrimitive, sync};
use qubicon_windowing::x11::WindowEvent;

use super::Application;

impl Application {
    fn instantiate_resources(&mut self) -> (Buffer<StandartMemoryAllocator>, Buffer<StandartMemoryAllocator>, Arc<DescriptorSet>) {
        let voxel_staging_buffer = self.vk_ctx.device.create_buffer(
            Arc::clone(&self.vk_ctx.allocator),
            MemoryTypeProperties::HOST_VISIBLE,
            &BufferCreateInfo {
                usage_flags: BufferUsageFlags::TRANSFER_SRC,
                size: 1024,
                main_owner_queue_family: self.vk_ctx.queue_family,

                ..Default::default()
            }
        ).expect("failed to create staging buffer for voxel data");

        // there should be buffer filling

        let mut order = self.vk_ctx.resource_factory.create_order(Arc::clone(&self.vk_ctx.allocator))
            .unwrap();

        order.request_buffer(
            MemoryTypeProperties::DEVICE_LOCAL,
            BufferRequest {
                usage_flags: BufferUsageFlags::STORAGE_BUFFER,
                create_flags: Default::default(),
                size: 1024,
                main_owner_queue_family: self.vk_ctx.queue_family,
                staging_buffer: Some(
                    BufferStagingBufferInfo {
                        buffer: &voxel_staging_buffer,
                        regions: &[
                            BufferCopy {
                                src_offset: 0,
                                dst_offset: 0,
                                size: 1024
                            }
                        ]
                    }
                )
            }
        ).unwrap();

        let order = order.do_order().unwrap();



        let uniform_buffer = self.vk_ctx.device.create_buffer(
            Arc::clone(&self.vk_ctx.allocator),
            MemoryTypeProperties::HOST_VISIBLE,
            &BufferCreateInfo {
                usage_flags: BufferUsageFlags::UNIFORM_BUFFER,
                size: 1024,
                main_owner_queue_family: self.vk_ctx.queue_family,

                ..Default::default()
            }
        ).expect("failed to create uniform buffer");

        let descriptor_set = unsafe {
            self.descriptor_pool.allocate_descriptor_set_unchecked(Arc::clone(&self.descriptor_set_layout))
        }.expect("failed to allocate descriptor set");

        let voxel_buffer = order.wait().1.swap_remove(0);


        unsafe {
            descriptor_set.update_unchecked(
                &[
                    DescriptorWrite {
                        binding: 0,
                        index: 0,
                        write_info: BufferWriteInfo {
                            buffer: &uniform_buffer,
                            offset: 0,
                            len: 1024
                        }
                    },
                    DescriptorWrite {
                        binding: 1,
                        index: 0,
                        write_info: BufferWriteInfo {
                            buffer: &voxel_buffer,
                            offset: 0,
                            len: 1024
                        }
                    }
                ]
            )
        }

        (uniform_buffer, voxel_buffer, descriptor_set)
    }

    pub fn run(mut self) {
        let command_pool = self.vk_ctx.compute_queue.create_command_pool().unwrap();
        let (uniform_buffer, voxel_data_buffer, descriptor_set) = self.instantiate_resources();

        self.windowing_server.window_mut(self.window_id)
            .unwrap()
            .show();

        'event_loop: loop {
            self.windowing_server.update();

            let mut window = self.windowing_server.window_mut(self.window_id)
                .unwrap();

            { // event handling
                let mut resize_required = false;

                for event in window.events() {
                    match event {
                        WindowEvent::Close => break 'event_loop,
                        WindowEvent::Resize { .. } => resize_required = true,

                        _ => {}
                    }
                }

                if resize_required {
                    window.force_swapchain_resize().unwrap();
                }
            }

            {
                let swapchain = unsafe { window.swapchain_mut() }.unwrap();
                let image = loop {
                    let fence = self.vk_ctx.device.create_fence(Default::default()).unwrap();

                    let res = swapchain.acquare_next_image(
                        AcquireImageSyncPrimitive::<sync::semaphore_types::Binary>::Fence(&fence),
                        u64::MAX
                    );

                    fence.wait(u64::MAX);

                    if let Ok(img) = res {
                        break img;
                    }
                };


                unsafe {
                    let image_view = image.create_image_view_unchecked(
                        &ImageViewCreateInfo {
                            view_type: ImageViewType::Type2D,
                            format: image.format(),
                            components: Default::default(),
                            subresource_range: ImageSubresourceRange {
                                aspect_mask: ImageAspect::COLOR,
                                mip_levels: 0..1,
                                array_layers: 0..1
                            }
                        }
                    ).unwrap();

                    descriptor_set.update_unchecked(
                        &[
                            DescriptorWrite {
                                binding: 2,
                                index: 0,
                                write_info: ImageWriteInfo {
                                    sampler: None,
                                    image_view: &image_view,
                                    image_layout: ImageLayout::General
                                }
                            }
                        ]
                    );

                    let command_buffer = command_pool.create_primary_command_buffer(
                        CommandBufferUsageFlags::ONE_TIME_SUBMIT
                    ).unwrap()
                        .cmd_bind_descriptor_set_unchecked(PipelineBindPoint::Compute, 0, &self.pipeline_layout, &descriptor_set)
                        .cmd_bind_compute_pipeline_unchecked(&self.rendering_pipeline)
                        .cmd_pipeline_barrier_unchecked::<_, HollowDeviceMemoryAllocator>(
                            PipelineStageFlags::TOP_OF_PIPE,
                            PipelineStageFlags::COMPUTE_SHADER,
                            PipelineBarrierDependencyFlags::empty(),
                            &[],
                            &[
                                ImageMemoryBarrier {
                                    src_access_mask: AccessFlags::empty(),
                                    dst_access_mask: AccessFlags::SHADER_WRITE,

                                    old_layout: ImageLayout::Undefined,
                                    new_layout: ImageLayout::General,
                                    src_queue_family_index: u32::MAX,
                                    dst_queue_family_index: u32::MAX,
                                    image: &image,
                                    subresource_range: ImageSubresourceRange {
                                        aspect_mask: ImageAspect::COLOR,
                                        mip_levels: 0..1,
                                        array_layers: 0..1
                                    }
                                }
                            ],
                            &[]
                        )
                        .cmd_dispatch_unchecked(100, 100, 100)
                        .cmd_pipeline_barrier_unchecked::<_, HollowDeviceMemoryAllocator>(
                            PipelineStageFlags::COMPUTE_SHADER,
                            PipelineStageFlags::BOTTOM_OF_PIPE,
                            PipelineBarrierDependencyFlags::empty(),
                            &[],
                            &[
                                ImageMemoryBarrier {
                                    src_access_mask: AccessFlags::SHADER_WRITE,
                                    dst_access_mask: AccessFlags::empty(),

                                    old_layout: ImageLayout::General,
                                    new_layout: ImageLayout::PresentSrc,

                                    src_queue_family_index: u32::MAX,
                                    dst_queue_family_index: u32::MAX,

                                    image: &image,
                                    subresource_range: ImageSubresourceRange {
                                        aspect_mask: ImageAspect::COLOR,
                                        mip_levels: 0..1,
                                        array_layers: 0..1
                                    }
                                }
                            ],
                            &[]
                        )
                        .build()
                        .unwrap();

                    
                    let op_semaphore = self.vk_ctx.device.create_semaphore::<sync::semaphore_types::Binary>()
                        .unwrap();
                    let op_semaphore = Arc::new(op_semaphore);

                    
                    let submission = self.vk_ctx.compute_queue.submit::<_, sync::semaphore_types::Binary>(
                        core::iter::once(Arc::clone(&op_semaphore)),
                        core::iter::empty(),
                        core::iter::once(command_buffer)
                    ).unwrap();


                    let mut present_entries = PresentInfoSwapchainEntry {
                        swapchain: &swapchain,
                        swapchain_image: &image,
                        result: Ok(())
                    };

                    let _present = self.vk_ctx.compute_queue.present(
                        PresentInfo {
                            wait_semaphores: &[&op_semaphore],
                            entries: core::slice::from_mut(&mut present_entries)
                        }
                    );
                }
            }
        }
    }
}