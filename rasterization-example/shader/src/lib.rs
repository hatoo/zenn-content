#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::arch::IndexUnchecked;

use spirv_std::glam::{vec3a, vec4, Vec3A, Vec4};

// vert_id < 3
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    color: &mut Vec3A,
) {
    *out_pos = *unsafe {
        [
            vec4(1.0, 1.0, 0.0, 1.0),
            vec4(0.0, -1.0, 0.0, 1.0),
            vec4(-1.0, 1.0, 0.0, 1.0),
        ]
        .index_unchecked(vert_id as usize)
    };

    *color = *unsafe {
        [
            vec3a(1.0, 0.0, 0.0),
            vec3a(0.0, 1.0, 0.0),
            vec3a(0.0, 0.0, 1.0),
        ]
        .index_unchecked(vert_id as usize)
    };
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4, color: Vec3A) {
    *output = color.extend(1.0);
}
