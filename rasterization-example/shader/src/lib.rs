#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::arch::IndexUnchecked;

use spirv_std::glam::{vec3, vec4, Vec3, Vec4};

// vert_id < 3
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    color: &mut Vec3,
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
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        ]
        .index_unchecked(vert_id as usize)
    };
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4, color: Vec3) {
    *output = color.extend(1.0);
}
