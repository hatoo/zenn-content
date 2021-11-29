use std::{
    collections::HashSet,
    ffi::{c_void, CStr, CString},
    fs::File,
    io::Write,
    ptr,
};

use ash::{prelude::VkResult, vk};

fn main() {
    const ENABLE_VALIDATION_LAYER: bool = true;
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const COLOR_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;

    let extent = vk::Extent2D::builder().width(WIDTH).height(HEIGHT).build();

    let validation_layers: Vec<CString> = if ENABLE_VALIDATION_LAYER {
        vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()]
    } else {
        Vec::new()
    };
    let validation_layers_ptr: Vec<*const i8> = validation_layers
        .iter()
        .map(|c_str| c_str.as_ptr())
        .collect();

    let entry = unsafe { ash::Entry::new() }.unwrap();

    assert_eq!(
        check_validation_layer_support(
            &entry,
            validation_layers.iter().map(|cstring| cstring.as_c_str())
        ),
        Ok(true)
    );

    let instance = {
        let application_name = CString::new("Hello Triangle").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let mut debug_utils_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(default_vulkan_debug_utils_callback))
            .build();

        let application_info = vk::ApplicationInfo::builder()
            .application_name(application_name.as_c_str())
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_2)
            .build();

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(validation_layers_ptr.as_slice());

        let instance_create_info = if ENABLE_VALIDATION_LAYER {
            instance_create_info.push_next(&mut debug_utils_create_info)
        } else {
            instance_create_info
        }
        .build();

        unsafe { entry.create_instance(&instance_create_info, None) }
            .expect("failed to create instance!")
    };

    let (physical_device, queue_family_index) =
        pick_physical_device_and_queue_family_indices(&instance)
            .unwrap()
            .unwrap();

    let device: ash::Device = {
        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])
            .build()];

        let mut physical_device_vulkan_memory_model_features =
            vk::PhysicalDeviceVulkanMemoryModelFeatures::builder()
                .vulkan_memory_model(true)
                .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .push_next(&mut physical_device_vulkan_memory_model_features)
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(validation_layers_ptr.as_slice())
            .build();

        unsafe { instance.create_device(physical_device, &device_create_info, None) }
            .expect("Failed to create logical Device!")
    };

    let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    let pipeline_layout = {
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default();

        unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None) }
            .expect("Failed to create pipeline layout!")
    };

    let device_memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };

    let image = {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(COLOR_FORMAT)
            .extent(
                vk::Extent3D::builder()
                    .width(WIDTH)
                    .height(HEIGHT)
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
            .build();

        unsafe { device.create_image(&image_create_info, None) }.unwrap()
    };

    let device_memory = {
        let mem_reqs = unsafe { device.get_image_memory_requirements(image) };
        let mem_alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_reqs.size)
            .memory_type_index(get_memory_type_index(
                device_memory_properties,
                mem_reqs.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ));

        unsafe { device.allocate_memory(&mem_alloc_info, None) }.unwrap()
    };

    unsafe { device.bind_image_memory(image, device_memory, 0) }.unwrap();

    let image_view = {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(COLOR_FORMAT)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image)
            .build();

        unsafe { device.create_image_view(&image_view_create_info, None) }.unwrap()
    };

    // render pass

    let render_pass = {
        let color_attachments = [vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: COLOR_FORMAT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        }];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses)
            .build();

        unsafe { device.create_render_pass(&renderpass_create_info, None) }
            .expect("Failed to create render pass!")
    };

    let graphics_pipeline = {
        const SHADER: &[u8] = include_bytes!(env!("shader.spv"));

        let shader_module = unsafe { create_shader_module(&device, SHADER).unwrap() };

        let main_vs = CString::new("main_vs").unwrap();
        let main_fs = CString::new("main_fs").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .name(main_vs.as_c_str())
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .name(main_fs.as_c_str())
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::default();
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }];

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports)
            .build();

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL)
            .rasterizer_discard_enable(false)
            .depth_bias_enable(false)
            .build();

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
            .max_depth_bounds(1.0)
            .min_depth_bounds(0.0)
            .build();

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::all(),
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_blend_attachment_states)
            .build();
        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_create_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_statue_create_info)
            .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_state_create_info)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_index(-1)
            .build()];

        let graphics_pipeline = unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphic_pipeline_create_infos,
                None,
            )
        }
        .expect("Failed to create Graphics Pipeline!.")[0];

        unsafe {
            device.destroy_shader_module(shader_module, None);
        }

        graphics_pipeline
    };

    let framebuffer = {
        let image_views = [image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&image_views)
            .width(WIDTH)
            .height(HEIGHT)
            .layers(1)
            .build();

        unsafe { device.create_framebuffer(&framebuffer_create_info, None) }
            .expect("Failed to create Framebuffer!")
    };

    let command_pool = {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .build();

        unsafe { device.create_command_pool(&command_pool_create_info, None) }
            .expect("Failed to create Command Pool!")
    };

    let command_buffer = {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        unsafe { device.allocate_command_buffers(&command_buffer_allocate_info) }
            .expect("Failed to allocate Command Buffers!")[0]
    };

    {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE)
            .build();

        unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info) }
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values)
            .build();

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    {
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .build()];

        unsafe {
            device
                .queue_submit(graphics_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to execute queue submit.");

            device.queue_wait_idle(graphics_queue).unwrap();
        }
    }

    // transfer to host

    let dst_image = {
        let dst_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(COLOR_FORMAT)
            .extent(
                vk::Extent3D::builder()
                    .width(WIDTH)
                    .height(HEIGHT)
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::LINEAR)
            .usage(vk::ImageUsageFlags::TRANSFER_DST)
            .build();

        unsafe { device.create_image(&dst_image_create_info, None) }.unwrap()
    };

    let dst_device_memory = {
        let dst_mem_reqs = unsafe { device.get_image_memory_requirements(dst_image) };
        let dst_mem_alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(dst_mem_reqs.size)
            .memory_type_index(get_memory_type_index(
                device_memory_properties,
                dst_mem_reqs.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            ));

        unsafe { device.allocate_memory(&dst_mem_alloc_info, None) }.unwrap()
    };
    unsafe { device.bind_image_memory(dst_image, dst_device_memory, 0) }.unwrap();

    let copy_cmd = {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();

        unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap()[0]
    };

    {
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder().build();

        unsafe { device.begin_command_buffer(copy_cmd, &cmd_begin_info) }.unwrap();
    }

    {
        let image_barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(dst_image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        unsafe {
            device.cmd_pipeline_barrier(
                copy_cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );
        }
    }

    {
        let copy_region = vk::ImageCopy::builder()
            .src_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .build(),
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .build(),
            )
            .extent(
                vk::Extent3D::builder()
                    .width(WIDTH)
                    .height(HEIGHT)
                    .depth(1)
                    .build(),
            )
            .build();

        unsafe {
            device.cmd_copy_image(
                copy_cmd,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy_region],
            );
        }
    }

    {
        let image_barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::GENERAL)
            .image(dst_image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        unsafe {
            device.cmd_pipeline_barrier(
                copy_cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );
        }
    }

    {
        let command_buffers = [copy_cmd];
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            device.end_command_buffer(copy_cmd).unwrap();

            device
                .queue_submit(graphics_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to execute queue submit.");

            device.queue_wait_idle(graphics_queue).unwrap();
        }
    }

    let subresource_layout = {
        let subresource = vk::ImageSubresource::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build();

        unsafe { device.get_image_subresource_layout(dst_image, subresource) }
    };

    let data: *const u8 = unsafe {
        device
            .map_memory(
                dst_device_memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap() as _
    };

    let mut data = unsafe { data.offset(subresource_layout.offset as isize) };

    let mut png_encoder = png::Encoder::new(File::create("out.png").unwrap(), WIDTH, HEIGHT);

    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_color(png::ColorType::Rgba);

    let mut png_writer = png_encoder
        .write_header()
        .unwrap()
        .into_stream_writer_with_size((4 * WIDTH) as usize)
        .unwrap();

    for _ in 0..HEIGHT {
        let row = unsafe { std::slice::from_raw_parts(data, 4 * WIDTH as usize) };
        png_writer.write_all(row).unwrap();
        data = unsafe { data.offset(subresource_layout.row_pitch as isize) };
    }

    png_writer.finish().unwrap();

    unsafe {
        device.unmap_memory(dst_device_memory);
        device.free_memory(dst_device_memory, None);
        device.destroy_image(dst_image, None);
    }

    // clean up

    unsafe {
        device.destroy_command_pool(command_pool, None);
    }

    unsafe { device.destroy_framebuffer(framebuffer, None) };

    unsafe {
        device.destroy_pipeline(graphics_pipeline, None);
    }

    unsafe {
        device.destroy_pipeline_layout(pipeline_layout, None);
    }

    unsafe {
        device.destroy_render_pass(render_pass, None);
    }

    unsafe {
        device.destroy_image_view(image_view, None);
        device.destroy_image(image, None);
        device.free_memory(device_memory, None);
    }

    unsafe {
        device.destroy_device(None);
    }
}

fn check_validation_layer_support<'a>(
    entry: &ash::Entry,
    required_validation_layers: impl IntoIterator<Item = &'a CStr>,
) -> VkResult<bool> {
    let supported_layers: HashSet<CString> = entry
        .enumerate_instance_layer_properties()?
        .into_iter()
        .map(|layer_property| unsafe {
            CStr::from_ptr(layer_property.layer_name.as_ptr()).to_owned()
        })
        .collect();

    Ok(required_validation_layers
        .into_iter()
        .all(|l| supported_layers.contains(l)))
}

fn pick_physical_device_and_queue_family_indices(
    instance: &ash::Instance,
) -> VkResult<Option<(vk::PhysicalDevice, u32)>> {
    Ok(unsafe { instance.enumerate_physical_devices() }?
        .into_iter()
        .find_map(|physical_device| {
            let graphics_family =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) }
                    .into_iter()
                    .enumerate()
                    .find(|(_, device_properties)| {
                        device_properties.queue_count > 0
                            && device_properties
                                .queue_flags
                                .contains(vk::QueueFlags::GRAPHICS)
                    });

            graphics_family.map(|(i, _)| (physical_device, i as u32))
        }))
}

unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> VkResult<vk::ShaderModule> {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    device.create_shader_module(&shader_module_create_info, None)
}

fn get_memory_type_index(
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    mut type_bits: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..device_memory_properties.memory_type_count {
        if (type_bits & 1) == 1 {
            if (device_memory_properties.memory_types[i as usize].property_flags & properties)
                == properties
            {
                return i;
            }
        }
        type_bits >>= 1;
    }
    0
}

pub unsafe extern "system" fn default_vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}
