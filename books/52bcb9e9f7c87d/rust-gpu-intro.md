---
title: "rust-gpuを使う"
---

[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)はRustのコードを[SPIR-V](https://en.wikipedia.org/wiki/Standard_Portable_Intermediate_Representation)にコンパイルするツールです。Vulkanを使いSPIR-VをGPU上で動かすことができます。
この文章ではrust-gpuでレイトレーシングを行うことを目的としていますが、この章ではまずrust-gpuで簡単なラスタライズ用のシェーダーを作っていきます。

# セットアップ

さっそくrust-gpuをやっていきましょう。
rust-gpuを使ったシェーダー用のプロジェクトとそれを使うアプリケーション用のプロジェクトの二つを作ります。シェーダーはアプリケーションの`build.rs`でコンパイルします。

```
mkdir rasterization-example
cd rasterization-example
# アプリケーション用のプロジェクト
cargo new rasterization-example
# シェーダー用のプロジェクト
cargo new rasterization-example-shader --lib
```

Cargo Workspaceの設定をします。

```toml:Cargo.toml
[workspace]
members = [
	"rasterization-example",
	"rasterization-example-shader"
]
```

rust-gpuは特定のRustのバージョンで動くため、rust-toolchainを[ここ](https://github.com/EmbarkStudios/rust-gpu/blob/main/rust-toolchain)からコピーします。

```toml:rust-toolchain
# If you see this, run `rustup self update` to get rustup 1.23 or newer.

# NOTE: above comment is for older `rustup` (before TOML support was added),
# which will treat the first line as the toolchain name, and therefore show it
# to the user in the error, instead of "error: invalid channel name '[toolchain]'".

[toolchain]
channel = "nightly-2021-10-26"
components = ["rust-src", "rustc-dev", "llvm-tools-preview"]
```

rasterization-example-shaderをSPIR-Vでコンパイルするために設定してます。

```toml:rasterization-example/Cargo.toml
...
[build-dependencies]
spirv-builder = { git = "https://github.com/EmbarkStudios/rust-gpu" }
```

```rust:rasterization-example/build.rs
use std::error::Error;

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../rasterization-example-shader", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
```

# シェーダーを書く

ここからシェーダーを書いていきます。
Cargo.tomlにlibを設定しspirv-stdをdependenciesに加えます。

```toml:rasterization-example-shader/Cargo.toml
...
[lib]
crate-type = ["lib", "dylib"]

[dependencies]
spirv-std = { git="https://github.com/EmbarkStudios/rust-gpu.git", features = ["glam"] }
```

vertexシェーダーとfragmentシェーダーを書いていきます。
vertexシェーダーで大きな三角形を描き、fragment シェーダーで色を付けます。

```rust:rasterization-example-shader/src/lib.rs
#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

// features = ["glam"]を指定したのでglamの型をベクトルとして使える
use spirv_std::glam::{vec3, vec4, Vec3, Vec4};

// vert_id < 3
// vertex shaderであることを指定
#[spirv(vertex)]
pub fn main_vs(
    // 頂点番号、ここでは頂点番号が0, 1, 2の範囲であることを仮定している
    // 他にどのような #[sprv(...)]が使えるかは https://github.com/EmbarkStudios/rust-gpu/blob/main/crates/rustc_codegen_spirv/src/symbols.rs を見るとよい
    #[spirv(vertex_index)] vert_id: i32,
    // 頂点の場所
    #[spirv(position)] out_pos: &mut Vec4,
    // 何も指定せずに &mut したのでLocation 0の出力だと解釈される
    // これがmain_fsのcolorに渡される
    color: &mut Vec3,
) {
    *out_pos = [
        vec4(1.0, 1.0, 0.0, 1.0),
        vec4(0.0, -1.0, 0.0, 1.0),
        vec4(-1.0, 1.0, 0.0, 1.0),
    ][vert_id as usize];

    *color = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ][vert_id as usize];
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4, color: Vec3) {
    *output = color.extend(1.0);
}

```

# シェーダーを確認する

アプリケーション側からビルドされたシェーダーのバイナリのパスをコンパイル時に`env!("rasterization_example_shader.spv")`で取得できます。

```rust:rasterization-example/src/main.rs
fn main() {
    const SHADER_PATH: &str = env!("rasterization_example_shader.spv");
    const SHADER: &[u8] = include_bytes!(env!("rasterization_example_shader.spv"));

    dbg!(SHADER_PATH);
    dbg!(SHADER.len());
}
```

```
> cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target\debug\rasterization-example.exe`
[rasterization-example\src\main.rs:5] SHADER_PATH = "c:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\rasterization_example_shader.spv.dir\\module"
[rasterization-example\src\main.rs:6] SHADER.len() = 1580
```

ここでSPIR-Vの実行は次章に回してここではSPIR-Vのディスアセンブルした結果を確認して終わりにします。
まず、[SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools)をインストールします。
SPIRR-Toolsのspirv-disでディスアセンブルします。

```
> spirv-dis.exe "c:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\rasterization_example_shader.spv.dir\\module"
; SPIR-V
; Version: 1.5
; Generator: Embark Studios Rust GPU Compiler Backend; 0
; Bound: 97
; Schema: 0
               OpCapability Shader
               OpCapability VulkanMemoryModel
               OpMemoryModel Logical Vulkan
               OpEntryPoint Vertex %1 "main_vs" %gl_VertexIndex %gl_Position %4
               OpEntryPoint Fragment %5 "main_fs" %6 %7
               OpExecutionMode %5 OriginUpperLeft
               OpDecorate %gl_VertexIndex BuiltIn VertexIndex
               OpDecorate %gl_Position BuiltIn Position
               OpDecorate %4 Location 0
               OpDecorate %6 Location 0
               OpDecorate %7 Location 0
               OpDecorate %_arr_v4float_uint_3 ArrayStride 16
               OpDecorate %_arr_v3float_uint_3 ArrayStride 16
      %float = OpTypeFloat 32
    %v4float = OpTypeVector %float 4
    %v3float = OpTypeVector %float 3
%_ptr_Input_v3float = OpTypePointer Input %v3float
%_ptr_Output_v3float = OpTypePointer Output %v3float
%_ptr_Function_v3float = OpTypePointer Function %v3float
       %void = OpTypeVoid
        %int = OpTypeInt 32 1
%_ptr_Output_v4float = OpTypePointer Output %v4float
%_ptr_Function_v4float = OpTypePointer Function %v4float
         %24 = OpTypeFunction %void
%_ptr_Input_int = OpTypePointer Input %int
%gl_VertexIndex = OpVariable %_ptr_Input_int Input
%gl_Position = OpVariable %_ptr_Output_v4float Output
          %4 = OpVariable %_ptr_Output_v3float Output
          %6 = OpVariable %_ptr_Output_v4float Output
          %7 = OpVariable %_ptr_Input_v3float Input
       %uint = OpTypeInt 32 0
     %uint_0 = OpConstant %uint 0
     %uint_1 = OpConstant %uint 1
     %uint_2 = OpConstant %uint 2
     %uint_3 = OpConstant %uint 3
%_arr_v4float_uint_3 = OpTypeArray %v4float %uint_3
%_ptr_Function__arr_v4float_uint_3 = OpTypePointer Function %_arr_v4float_uint_3
%_arr_v3float_uint_3 = OpTypeArray %v3float %uint_3
%_ptr_Function__arr_v3float_uint_3 = OpTypePointer Function %_arr_v3float_uint_3
    %float_1 = OpConstant %float 1
   %float_n1 = OpConstant %float -1
    %float_0 = OpConstant %float 0
       %bool = OpTypeBool
         %91 = OpConstantComposite %v4float %float_1 %float_n1 %float_0 %float_1
         %92 = OpConstantComposite %v4float %float_0 %float_1 %float_0 %float_1
         %93 = OpConstantComposite %v4float %float_n1 %float_n1 %float_0 %float_1
         %94 = OpConstantComposite %v3float %float_1 %float_0 %float_0
         %95 = OpConstantComposite %v3float %float_0 %float_1 %float_0
         %96 = OpConstantComposite %v3float %float_0 %float_0 %float_1
          %1 = OpFunction %void None %24
         %38 = OpLabel
         %39 = OpVariable %_ptr_Function__arr_v4float_uint_3 Function
         %40 = OpVariable %_ptr_Function__arr_v3float_uint_3 Function
               OpSelectionMerge %86 None
               OpSwitch %uint_0 %87
         %87 = OpLabel
         %41 = OpLoad %int %gl_VertexIndex
         %45 = OpAccessChain %_ptr_Function_v4float %39 %uint_0
               OpStore %45 %91
         %46 = OpAccessChain %_ptr_Function_v4float %39 %uint_1
               OpStore %46 %92
         %47 = OpAccessChain %_ptr_Function_v4float %39 %uint_2
               OpStore %47 %93
         %48 = OpBitcast %uint %41
         %49 = OpULessThan %bool %48 %uint_3
               OpSelectionMerge %50 None
               OpBranchConditional %49 %51 %52
         %52 = OpLabel
               OpBranch %86
         %51 = OpLabel
         %53 = OpInBoundsAccessChain %_ptr_Function_v4float %39 %48
         %54 = OpLoad %v4float %53
               OpStore %gl_Position %54
         %58 = OpAccessChain %_ptr_Function_v3float %40 %uint_0
               OpStore %58 %94
         %59 = OpAccessChain %_ptr_Function_v3float %40 %uint_1
               OpStore %59 %95
         %60 = OpAccessChain %_ptr_Function_v3float %40 %uint_2
               OpStore %60 %96
               OpSelectionMerge %63 None
               OpBranchConditional %49 %64 %65
         %65 = OpLabel
               OpBranch %86
         %64 = OpLabel
         %66 = OpInBoundsAccessChain %_ptr_Function_v3float %40 %48
         %67 = OpLoad %v3float %66
               OpStore %4 %67
               OpBranch %86
         %63 = OpLabel
               OpUnreachable
         %50 = OpLabel
               OpUnreachable
         %86 = OpLabel
               OpReturn
               OpFunctionEnd
          %5 = OpFunction %void None %24
         %80 = OpLabel
         %81 = OpLoad %v3float %7
         %82 = OpCompositeExtract %float %81 0
         %83 = OpCompositeExtract %float %81 1
         %84 = OpCompositeExtract %float %81 2
         %85 = OpCompositeConstruct %v4float %82 %83 %84 %float_1
               OpStore %6 %85
               OpReturn
               OpFunctionEnd
```

それっぽい結果が出ているのを確認できました。