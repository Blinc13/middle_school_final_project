use std::sync::Arc;

use qubicon_input_server::{keymaps::{Abs, Key}, ActionEventType, ActionInputEntry, LinuxInputServer};
use qubicon_vulkan::{descriptors::{alloc::DescriptorPoolSize, DescriptorBinding, DescriptorPool, DescriptorPoolCreateInfo, DescriptorSetLayout, DescriptorSetLayoutCreateInfo, DescriptorType}, device::{create_info::{DeviceCreateInfo, QueueFamilyUsage}, Device}, instance::{creation_info::InstanceCreateInfo, physical_device::{queue_info::QueueFamilyCapabilities, PhysicalDevice}}, memory::{alloc::standart_device_memory_allocator::StandartMemoryAllocator, resources::{buffer::Buffer, format::Format, image::{Image, ImageUsageFlags}}, ResourceFactory}, queue::Queue, shaders::{compute::{ComputePipeline, ComputePipelineCreateInfo}, pipeline_layout::PipelineLayout, PipelineShaderStageCreateInfo, ShaderStageFlags}, surface::{CompositeAlphaFlags, PresentMode, SurfaceTransformFlags}, Instance};
use qubicon_windowing::{x11::{WindowId, WindowingServer}, AssociatedSwapchainCreateInfo};

const SHADER_SRC: &[u8] = include_bytes!("shader/rendering_shader.spv");

mod run;

pub struct Application {
    vk_ctx: VulkanContext,
    input_server: LinuxInputServer,
    windowing_server: WindowingServer,

    descriptor_pool: DescriptorPool,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    pipeline_layout: Arc<PipelineLayout>,

    window_id: WindowId,
    rendering_pipeline: Arc<ComputePipeline>
}


impl Application {
    // constructs input server and adds input actions
    fn init_input_server() -> LinuxInputServer {
        let mut input_server = LinuxInputServer::new()
            .expect("failed to init input server");

        
        // movement with mouse and keyboard
        input_server.add_input_action(
            "move_left",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::LX, range: -1.01..0.0 },
                },
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Key { key: Key::A, pressed: true }
                }
            ]
        );
        input_server.add_input_action(
            "move_right",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::LX, range: 0.0..1.01 },
                },
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Key { key: Key::D, pressed: true }
                }
            ]
        );
        input_server.add_input_action(
            "move_forward",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::LY, range: -1.01..0.0 },
                },
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Key { key: Key::W, pressed: true }
                }
            ]
        );
        input_server.add_input_action(
            "move_backward",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::LY, range: 0.0..1.01 },
                },
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Key { key: Key::S, pressed: true }
                }
            ]
        );


        //camera controls with gamepad
        input_server.add_input_action(
            "rotate_left",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::RX, range: -1.01..0.0 }
                }
            ]
        );
        input_server.add_input_action(
            "rotate_right",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::RX, range: 0.0..1.01 }
                }
            ]
        );
        input_server.add_input_action(
            "rotate_up",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::RY, range: -1.01..0.0 }
                }
            ]
        );
        input_server.add_input_action(
            "rotate_down",
            [
                ActionInputEntry {
                    device_id: None,
                    r#type: ActionEventType::Abs { abs: Abs::RY, range: 0.0..1.01 }
                }
            ]
        );


        input_server
    }

    // inits window server and creates window with swapchain
    fn init_windowing_server(vk_ctx: &VulkanContext, width: u32, height: u32) -> (WindowId, WindowingServer) {
        let mut windowing_server = WindowingServer::init();

        let window_id = windowing_server.create_window_vulkan(
            &vk_ctx.device,
            width,
            height,
            &AssociatedSwapchainCreateInfo {
                min_image_count: 3,
                image_array_layers: 1,
                image_usage: ImageUsageFlags::TRANSFER_DST | /* tmp */ ImageUsageFlags::STORAGE,

                pre_transform: SurfaceTransformFlags::IDENTITY,
                composite_alpha: CompositeAlphaFlags::OPAQUE,
               
                clipped: false,
                image_main_owner_queue_family: vk_ctx.queue_family
            },
            | mode | mode == PresentMode::FIFO /* V-Sync */,
            // dont care
            | f | {
                println!("{f:?}");

                true
            }
        ).expect("failed to create window");

        (window_id, windowing_server)
    }

    fn create_vulkan_objects(vk_ctx: &VulkanContext, buffered_frames_count: u32) -> (Arc<DescriptorSetLayout>, Arc<PipelineLayout>, Arc<ComputePipeline>, DescriptorPool) {
        let descriptor_set_layout = unsafe {
            vk_ctx.device.create_descriptor_set_layout_unchecked(
                DescriptorSetLayoutCreateInfo {
                    bindings: [
                        DescriptorBinding {
                            shader_stage_flags: ShaderStageFlags::COMPUTE,
                            r#type: DescriptorType::UniformBuffer,
                            count: 1
                        },
                        DescriptorBinding {
                            shader_stage_flags: ShaderStageFlags::COMPUTE,
                            r#type: DescriptorType::StorageBuffer,
                            count: 1
                        },
                        DescriptorBinding {
                            shader_stage_flags: ShaderStageFlags::COMPUTE,
                            r#type: DescriptorType::StorageImage,
                            count: 1
                        }
                    ]
                }
            )
        }.expect("failed to create descriptor set layout");

        let pipeline_layout = vk_ctx.device.create_pipeline_layout(
            [Arc::clone(&descriptor_set_layout)]
        ).expect("failed to create pipeline layout");

        let rendering_pipeline = unsafe {
            let shader_module = {
                let mut binary = Vec::<u32>::with_capacity(SHADER_SRC.len() / 4);
                
                // :)
                core::ptr::copy_nonoverlapping(
                    SHADER_SRC.as_ptr(),
                    binary.as_mut_ptr().cast(),
                    SHADER_SRC.len()
                );
                binary.set_len(SHADER_SRC.len() / 4);


                vk_ctx.device.create_shader_module_from_binary(&binary)
            }.expect("failed to create shader module");

            vk_ctx.device.create_compute_pipeline_unchecked(
                ComputePipelineCreateInfo {
                    create_flags: Default::default(),
                    stage: PipelineShaderStageCreateInfo {
                        stage: ShaderStageFlags::COMPUTE,
                        module: &shader_module,
                        entry_name: "main"
                    },
                    layout: Arc::clone(&pipeline_layout),
                    base_pipeline: None
                }
            )
        }.expect("failed to create rendering pipeline");


        
        let descriptor_pool = vk_ctx.device.create_descriptor_pool(
            DescriptorPoolCreateInfo {
                max_sets: buffered_frames_count,
                pool_sizes: [
                    DescriptorPoolSize {
                        r#type: DescriptorType::UniformBuffer,
                        count: buffered_frames_count
                    },
                    DescriptorPoolSize {
                        r#type: DescriptorType::StorageBuffer,
                        count: buffered_frames_count
                    },
                    DescriptorPoolSize {
                        r#type: DescriptorType::StorageImage,
                        count: buffered_frames_count
                    }
                ]
            }
        ).expect("failed to create descriptor pool");


        return (descriptor_set_layout, pipeline_layout, rendering_pipeline, descriptor_pool);
    }
    
    pub fn init() -> Self {
        let vk_ctx = VulkanContext::init();

        let input_server = Self::init_input_server();
        let (window_id, windowing_server) = Self::init_windowing_server(&vk_ctx, 600, 400);

        let (descriptor_set_layout, pipeline_layout, rendering_pipeline, descriptor_pool) = Self::create_vulkan_objects(&vk_ctx, 1);
        
        Self {
            vk_ctx,
            input_server,
            windowing_server,

            descriptor_pool,
            descriptor_set_layout,
            pipeline_layout,

            window_id,
            rendering_pipeline
        }
    }
}


struct VulkanContext {
    instance: Instance,
    device: Device,

    queue_family: u32,
    compute_queue: Queue,
    
    resource_factory: ResourceFactory,
    allocator: Arc<StandartMemoryAllocator>
}

impl VulkanContext {
    fn find_family(dev: &PhysicalDevice, capabilities: QueueFamilyCapabilities, count: u32) -> Option<u32> {
        dev.get_queue_family_infos()
            .iter()
            .enumerate()
            .find(| (_, family) | family.capabilities.contains(capabilities) && family.queue_count > count)
            .map(| (idx, _) | idx as u32)
    }

    pub fn init() -> Self {
        let capabilitites: QueueFamilyCapabilities = QueueFamilyCapabilities::COMPUTE | QueueFamilyCapabilities::TRANSFER;

        let instance = Instance::create(
            &InstanceCreateInfo {
                enable_windowing: true
            }
        ).expect("failed to create Vulkan instance");

        let (queue_family, device) = instance.enumerate_devices()
            .expect("failed to enumerate devices")
            .filter(| dev | dev.get_properties().device_name.starts_with("AMD"))
            .filter_map(| dev | Some((Self::find_family(&dev, capabilitites, 2)?, dev)))
            .next()
            .expect("no matching Vulkan devices found");

        let device = device.create_logical_device(
            DeviceCreateInfo {
                features: Default::default(),
                enable_swapchain: true,

                queues: [
                    QueueFamilyUsage {
                        family_index: queue_family,
                        queue_count: 2
                    }
                ]
            }
        ).expect("failed to create logical device");


        let compute_queue = device.get_queue(queue_family, 0)
            .expect("no compute queue found");
        let transfer_queue = device.get_queue(queue_family, 1)
            .expect("no transfer queue found");

        let resource_factory = ResourceFactory::init(
            &device,
            transfer_queue
        ).expect("failed to init resource factory");

        let allocator = StandartMemoryAllocator::new(&device);

        Self {
            instance,
            device,

            queue_family,
            compute_queue,

            resource_factory,
            allocator
        }
    }
}