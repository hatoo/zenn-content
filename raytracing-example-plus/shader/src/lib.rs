#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

use crate::rand::DefaultRng;
use camera::Camera;
use material::{EnumMaterial, Material, Scatter};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{
    arch::{ignore_intersection, report_intersection, IndexUnchecked},
    glam::{uvec2, vec3a, UVec3, Vec2, Vec3A, Vec4},
    image::Image,
    ray_tracing::{AccelerationStructure, RayFlags},
};

pub mod camera;
pub mod material;
pub mod math;
pub mod rand;

#[derive(Clone, Copy, Default)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}
#[derive(Clone, Default)]
pub struct RayPayload {
    pub is_miss: u32,
    pub position: Vec3A,
    pub normal: Vec3A,
    pub material: u32,
    pub front_face: u32,
}

impl RayPayload {
    pub fn new_miss(color: Vec3A) -> Self {
        Self {
            is_miss: 1,
            position: color,
            ..Default::default()
        }
    }

    pub fn new_hit(
        position: Vec3A,
        outward_normal: Vec3A,
        ray_direction: Vec3A,
        material: u32,
    ) -> Self {
        let front_face = ray_direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            is_miss: 0,
            position,
            normal,
            material,
            front_face: if front_face { 1 } else { 0 },
        }
    }
}

pub struct PushConstants {
    seed: u32,
}

#[spirv(miss)]
pub fn main_miss(
    #[spirv(world_ray_direction)] world_ray_direction: Vec3A,
    #[spirv(incoming_ray_payload)] out: &mut RayPayload,
) {
    let unit_direction = world_ray_direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    let color = vec3a(1.0, 1.0, 1.0).lerp(vec3a(0.5, 0.7, 1.0), t);

    *out = RayPayload::new_miss(color);
}

#[spirv(ray_generation)]
pub fn main_ray_generation(
    #[spirv(launch_id)] launch_id: UVec3,
    #[spirv(launch_size)] launch_size: UVec3,
    #[spirv(push_constant)] constants: &PushConstants,
    #[spirv(descriptor_set = 0, binding = 0)] top_level_as: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] materials: &[EnumMaterial],
    #[spirv(ray_payload)] payload: &mut RayPayload,
) {
    let rand_seed = (launch_id.y * launch_size.x + launch_id.x) ^ constants.seed;
    let mut rng = DefaultRng::new(rand_seed);

    let camera = Camera::new(
        vec3a(13.0, 2.0, 3.0),
        vec3a(0.0, 0.0, 0.0),
        vec3a(0.0, 1.0, 0.0),
        20.0 / 180.0 * core::f32::consts::PI,
        launch_size.x as f32 / launch_size.y as f32,
        0.1,
        10.0,
    );

    let u = (launch_id.x as f32 + rng.next_f32()) / (launch_size.x - 1) as f32;
    let v = (launch_id.y as f32 + rng.next_f32()) / (launch_size.y - 1) as f32;

    let cull_mask = 0xff;
    let tmin = 0.001;
    let tmax = 100000.0;

    let mut color = vec3a(1.0, 1.0, 1.0);

    let mut ray = camera.get_ray(u, v, &mut rng);

    for _ in 0..50 {
        *payload = RayPayload::default();
        unsafe {
            top_level_as.trace_ray(
                RayFlags::empty(),
                cull_mask,
                0,
                0,
                0,
                ray.origin,
                tmin,
                ray.direction,
                tmax,
                payload,
            );
        }

        if payload.is_miss != 0 {
            color *= payload.position;
            break;
        } else {
            let mut scatter = Scatter::default();
            if unsafe { materials.index_unchecked(payload.material as usize) }.scatter(
                &ray,
                payload,
                &mut rng,
                &mut scatter,
            ) {
                color *= scatter.color;
                ray = scatter.ray;
            } else {
                break;
            }
        }
    }

    let pos = uvec2(launch_id.x, launch_size.y - 1 - launch_id.y);
    let prev: Vec4 = image.read(pos);

    unsafe {
        image.write(pos, prev + color.extend(1.0));
    }
}

#[spirv(intersection)]
pub fn sphere_intersection(
    #[spirv(object_ray_origin)] ray_origin: Vec3A,
    #[spirv(object_ray_direction)] ray_direction: Vec3A,
    #[spirv(ray_tmin)] t_min: f32,
    #[spirv(ray_tmax)] t_max: f32,
) {
    let oc = ray_origin;
    let a = ray_direction.length_squared();
    let half_b = oc.dot(ray_direction);
    let c = oc.length_squared() - 1.0;

    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return;
    }

    let sqrtd = discriminant.sqrt();

    let root0 = (-half_b - sqrtd) / a;
    let root1 = (-half_b + sqrtd) / a;

    if root0 >= t_min && root0 <= t_max {
        unsafe {
            report_intersection(root0, 0);
        }
    }

    if root1 >= t_min && root1 <= t_max {
        unsafe {
            report_intersection(root1, 0);
        }
    }
}

#[derive(Clone, Copy)]
#[spirv(matrix)]
#[repr(C)]
pub struct Affine3 {
    pub x: Vec3A,
    pub y: Vec3A,
    pub z: Vec3A,
    pub w: Vec3A,
}

#[spirv(closest_hit)]
pub fn sphere_closest_hit(
    #[spirv(ray_tmax)] t: f32,
    #[spirv(object_to_world)] object_to_world: Affine3,
    #[spirv(world_ray_origin)] world_ray_origin: Vec3A,
    #[spirv(world_ray_direction)] world_ray_direction: Vec3A,
    #[spirv(incoming_ray_payload)] out: &mut RayPayload,
    #[spirv(instance_custom_index)] instance_custom_index: u32,
) {
    let hit_pos = world_ray_origin + t * world_ray_direction;
    let normal = (hit_pos - object_to_world.w).normalize();
    *out = RayPayload::new_hit(hit_pos, normal, world_ray_direction, instance_custom_index);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3A,
    pub normal: Vec3A,
}

#[spirv(closest_hit)]
pub fn triangle_closest_hit(
    #[spirv(hit_attribute)] attribute: &Vec2,
    #[spirv(object_to_world)] object_to_world: Affine3,
    #[spirv(world_to_object)] world_to_object: Affine3,
    #[spirv(world_ray_direction)] world_ray_direction: Vec3A,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] vertices: &[Vertex],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] indices: &[u32],
    #[spirv(incoming_ray_payload)] out: &mut RayPayload,
    #[spirv(primitive_id)] primitive_id: u32,
    #[spirv(instance_custom_index)] instance_custom_index: u32,
) {
    let v0 = *unsafe {
        vertices.index_unchecked(*indices.index_unchecked(3 * primitive_id as usize + 0) as usize)
    };
    let v1 = *unsafe {
        vertices.index_unchecked(*indices.index_unchecked(3 * primitive_id as usize + 1) as usize)
    };
    let v2 = *unsafe {
        vertices.index_unchecked(*indices.index_unchecked(3 * primitive_id as usize + 2) as usize)
    };

    let barycentrics = vec3a(1.0 - attribute.x - attribute.y, attribute.x, attribute.y);

    let pos =
        v0.position * barycentrics.x + v1.position * barycentrics.y + v2.position * barycentrics.z;

    let nrm = v0.normal * barycentrics.x + v1.normal * barycentrics.y + v2.normal * barycentrics.z;

    let hit_pos = pos.x * object_to_world.x
        + pos.y * object_to_world.y
        + pos.z * object_to_world.z
        + object_to_world.w;

    let normal = vec3a(
        world_to_object.x.dot(nrm),
        world_to_object.y.dot(nrm),
        world_to_object.z.dot(nrm),
    )
    .normalize();

    *out = RayPayload::new_hit(hit_pos, normal, world_ray_direction, instance_custom_index);
}

#[spirv(any_hit)]
pub fn triangle_any_hit(
    #[spirv(ray_tmax)] t: f32,
    #[spirv(object_ray_origin)] object_ray_origin: Vec3A,
    #[spirv(object_ray_direction)] object_ray_direction: Vec3A,
) {
    let pos = object_ray_origin + t * object_ray_direction;

    if pos.length_squared() < 0.2 {
        unsafe { ignore_intersection() };
    }
}
