use std::error::Error;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("./shader", "spirv-unknown-spv1.3")
        .capability(Capability::RayTracingKHR)
        .capability(Capability::Int8)
        .extension("SPV_KHR_ray_tracing")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
