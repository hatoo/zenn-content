---
title: "三角形も描画してみる"
---

前章まででVKRで球を描いていきましたが、もうすこしVKRに慣れるために追加で三角形も描画していきます。
SBTの機能を使うことで、球を描く用のシェーダー、三角形を描く用のシェーダーを動的に切り替えることができます。

今までのraytracing-exampleをコピーしてraytracing-example-plusとして続けていきます。
コードは[こちら](https://github.com/hatoo/zenn-content/tree/master/raytracing-example-plus)。

# 三角形用のClosest-Hit Shaderを書く

三角形の当たり判定は標準で用意されているため、Closest-Hit Shaderだけ書けばよいです。
レイと三角形との衝突座標は`#[spirv(hit_attribute)]`から計算できます。

```rust:shader/src/lib.rs
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

#[spirv(closest_hit)]
pub fn triangle_closest_hit(
    #[spirv(hit_attribute)] attribute: &Vec2,
    #[spirv(object_to_world)] object_to_world: Affine3,
    #[spirv(world_ray_direction)] world_ray_direction: Vec3,
    // 各頂点の情報
    // あらかじめ用意しておく
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] vertices: &[Vertex],
    // 各面のindex
    // あらかじめ用意しておく
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] indices: &[u32],
    #[spirv(incoming_ray_payload)] out: &mut RayPayload,
    #[spirv(instance_custom_index)] instance_custom_index: u32,
) {
    // 各頂点の座標
    let v0 = vertices[indices[3 * instance_custom_index as usize + 0] as usize];
    let v1 = vertices[indices[3 * instance_custom_index as usize + 1] as usize];
    let v2 = vertices[indices[3 * instance_custom_index as usize + 2] as usize];

    let barycentrics = vec3(1.0 - attribute.x - attribute.y, attribute.x, attribute.y);

    // 衝突箇所は`hit_attribute`から求められる
    let pos =
        v0.position * barycentrics.x + v1.position * barycentrics.y + v2.position * barycentrics.z;

    // 法線
    let nrm = v0.normal * barycentrics.x + v1.normal * barycentrics.y + v2.normal * barycentrics.z;

    // 座標変換
    // asm!(...)を使えば1命令でできそうだが簡単のために自力で計算している
    let hit_pos = vec3(
        object_to_world.x.dot(pos),
        object_to_world.y.dot(pos),
        object_to_world.z.dot(pos),
    ) + object_to_world.w;

    let normal = vec3(
        object_to_world.x.dot(nrm),
        object_to_world.y.dot(nrm),
        object_to_world.z.dot(nrm),
    )
    .normalize();

    *out = RayPayload::new_hit(hit_pos, normal, world_ray_direction, instance_custom_index);
}
```

# BLASをつくる

三角形を保持したBLASを作ります。VKRの使い方に慣れたいだけなので三角形一個だけです。

```rust:src/main.rs
    let (bottom_as_triangle, bottom_as_triangle_buffer, vertex_buffer, index_buffer) = {
        // 頂点情報
        // シェーダー上では位置と法線のVec3二つだったが、アラインメントの関係上それぞれ4つのf32で配置する必要があることに注意
        const VERTICES: [[f32; 8]; 3] = [
            [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0],
        ];

        const INDICES: [u32; 3] = [0u32, 1, 2];

        let vertex_stride = std::mem::size_of::<f32>() * 8;
        let vertex_buffer_size = vertex_stride * vertices.len();

        let mut vertex_buffer = BufferResource::new(
            vertex_buffer_size as vk::DeviceSize,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device,
            device_memory_properties,
        );

        vertex_buffer.store(&VERTICES, &device);

        let index_buffer_size = std::mem::size_of::<f32>() * 3 * INDICES.len();

        let mut index_buffer = BufferResource::new(
            index_buffer_size as vk::DeviceSize,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device,
            device_memory_properties,
        );

        index_buffer.store(&INDICES, &device);

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                triangles: vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            get_buffer_device_address(&device, vertex_buffer.buffer)
                        },
                    })
                    .max_vertex(VERTICES.len() as u32 - 1)
                    .vertex_stride(vertex_stride as u64)
                    .vertex_format(vk::Format::R32G32B32_SFLOAT)
                    .index_data(vk::DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            get_buffer_device_address(&device, index_buffer.buffer)
                        },
                    })
                    .index_type(vk::IndexType::UINT32)
                    .build(),
            })
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .build();

        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .first_vertex(0)
            .primitive_count(INDICES.len() as u32)
            .primitive_offset(0)
            .transform_offset(0)
            .build();

        let geometries = [geometry];

        let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .build();

        let size_info = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                &[1],
            )
        };

        let bottom_as_buffer = BufferResource::new(
            size_info.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device,
            device_memory_properties,
        );

        let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
            .ty(build_info.ty)
            .size(size_info.acceleration_structure_size)
            .buffer(bottom_as_buffer.buffer)
            .offset(0)
            .build();

        let bottom_as =
            unsafe { acceleration_structure.create_acceleration_structure(&as_create_info, None) }
                .unwrap();

        build_info.dst_acceleration_structure = bottom_as;

        let scratch_buffer = BufferResource::new(
            size_info.build_scratch_size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device,
            device_memory_properties,
        );

        build_info.scratch_data = vk::DeviceOrHostAddressKHR {
            device_address: unsafe { get_buffer_device_address(&device, scratch_buffer.buffer) },
        };

        let build_command_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .build();

            let command_buffers =
                unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap();
            command_buffers[0]
        };

        unsafe {
            device
                .begin_command_buffer(
                    build_command_buffer,
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                        .build(),
                )
                .unwrap();

            let build_infos = [build_info];
            let build_range_infos: &[&[_]] = &[&[build_range_info]];

            acceleration_structure.cmd_build_acceleration_structures(
                build_command_buffer,
                &build_infos,
                build_range_infos,
            );
            device.end_command_buffer(build_command_buffer).unwrap();
            device
                .queue_submit(
                    graphics_queue,
                    &[vk::SubmitInfo::builder()
                        .command_buffers(&[build_command_buffer])
                        .build()],
                    vk::Fence::null(),
                )
                .expect("queue submit failed.");

            device.queue_wait_idle(graphics_queue).unwrap();
            device.free_command_buffers(command_pool, &[build_command_buffer]);
            scratch_buffer.destroy(&device);
        }
        (bottom_as, bottom_as_buffer, vertex_buffer, index_buffer)
    };
```