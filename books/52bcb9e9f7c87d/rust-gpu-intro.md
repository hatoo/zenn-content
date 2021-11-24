---
title: "rust-gpu入門"
---

[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)はRustのコードを[SPIR-V](https://en.wikipedia.org/wiki/Standard_Portable_Intermediate_Representation)にコンパイルするツールです。
この文章はrust-gpuでレイトレーシングを行うことを目的としていますが、この章ではまずrust-gpuで簡単なラスタライズ用のシェーダーをつくっていきます。

コードは[こちら](https://github.com/hatoo/zenn-content/tree/master/rasterization-example)にあります。`src/main.rs`が次章の内容になっていますがそれ以外は同じです。

# セットアップ

さっそくrust-gpuをやっていきましょう。
rust-gpuを使ったシェーダー用のプロジェクトとそれを使うアプリケーション用のプロジェクトの二つをつくります。シェーダーはアプリケーションの`build.rs`でコンパイルします。

```
# アプリケーション用のプロジェクト
cargo new rasterization-example
cd rasterization-example
# シェーダー用のプロジェクト
cargo new shader --lib
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

shaderをrust-gpuでコンパイルするために設定します。

```toml:Cargo.toml
...
[build-dependencies]
spirv-builder = { git = "https://github.com/EmbarkStudios/rust-gpu" }
```

```rust:build.rs
use std::error::Error;

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("shader", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
```

# シェーダーを書く

ここからシェーダーを書いていきます。
Cargo.tomlにlibを設定し[spirv-std](https://embarkstudios.github.io/rust-gpu/api/spirv_std/)をdependenciesに加えます。
spirv-stdはSPIR-Vターゲットでのstdみたいなものに相当します。


```toml:shader/Cargo.toml
...
[lib]
crate-type = ["lib", "dylib"]

[dependencies]
spirv-std = { git = "https://github.com/EmbarkStudios/rust-gpu.git", features = ["glam"] }
```

vertexシェーダーとfragmentシェーダーを書いていきます。
vertexシェーダーで大きな三角形を描き、fragment シェーダーで色を付けます。

```rust:shader/src/lib.rs
// ここら辺はテンプレ
// 気になる方は一つ一つ調べれば割とすぐに把握できるでしょう
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
    // gl_VertexIndex相当がここに入る
    #[spirv(vertex_index)] vert_id: i32,
    // gl_Position相当の変数
    #[spirv(position)] out_pos: &mut Vec4,
    // 何も指定せずに &mut したのでlayout(location = 0) outだと解釈される
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
pub fn main_fs(
    // layout(location = 0) out
    output: &mut Vec4, 
    // layout(location = 0) in
    color: Vec3) {
    *output = color.extend(1.0);
}

```

# シェーダーを確認する

アプリケーション側からビルドされたシェーダーのバイナリのパスをコンパイル時に`env!("<シェーダープロジェクトのディレクトリ名>.spv")`で取得できます。

```rust:src/main.rs
fn main() {
    const SHADER_PATH: &str = env!("shader.spv");
    const SHADER: &[u8] = include_bytes!(env!("shader.spv"));

    dbg!(SHADER_PATH);
    dbg!(SHADER.len());
}
```

```
> cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target\debug\rasterization-example.exe`
[src\main.rs:5] SHADER_PATH = "C:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\shader.spv.dir\\module"
[src\main.rs:6] SHADER.len() = 1580
```

このSPIR-Vの実行は次章に回してここではSPIR-Vのバイナリを確認して終わりにします。
[SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools)のspirv-disでディスアセンブルします。

```
> spirv-dis "C:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\shader.spv.dir\\module"
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
    %v3float = OpTypeVector %float 3
    %v4float = OpTypeVector %float 4
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
    %float_0 = OpConstant %float 0
   %float_n1 = OpConstant %float -1
       %bool = OpTypeBool
         %91 = OpConstantComposite %v4float %float_1 %float_1 %float_0 %float_1
         %92 = OpConstantComposite %v4float %float_0 %float_n1 %float_0 %float_1
         %93 = OpConstantComposite %v4float %float_n1 %float_1 %float_0 %float_1
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

また、[SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross)でSPIR-VをGLSLに変換した結果を見ることもできます。

```glsl
> spirv-cross "C:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\shader.spv.dir\\module" --entry main_vs
#version 450

layout(location = 0) out vec3 _4;

void main()
{
    do
    {
        vec4 _39[3];
        _39[0u] = vec4(1.0, 1.0, 0.0, 1.0);
        _39[1u] = vec4(0.0, -1.0, 0.0, 1.0);
        _39[2u] = vec4(-1.0, 1.0, 0.0, 1.0);
        uint _48 = uint(gl_VertexID);
        bool _49 = _48 < 3u;
        if (_49)
        {
            gl_Position = _39[_48];
            vec3 _40[3];
            _40[0u] = vec3(1.0, 0.0, 0.0);
            _40[1u] = vec3(0.0, 1.0, 0.0);
            _40[2u] = vec3(0.0, 0.0, 1.0);
            if (_49)
            {
                _4 = _40[_48];
                break;
            }
            else
            {
                break;
            }
        }
        else
        {
            break;
        }
    } while(false);
}

> spirv-cross "C:\\Users\\hato2\\Desktop\\zenn-content\\rasterization-example\\target\\spirv-builder\\spirv-unknown-vulkan1.2\\release\\deps\\shader.spv.dir\\module" --entry main_fs
#version 450

layout(location = 0) out vec4 _6;
layout(location = 0) in vec3 _7;

void main()
{
    _6 = vec4(_7, 1.0);
}
```

:::message
この例ではあまり問題になりませんがrust-gpuで生成されたSPIR-Vは最適化が甘い可能性があります。
ただ、ドライバはSPIR-VをさらにGPU用のコードに変換して実行するのでそこで最適化されることに期待しましょう。
:::

:::message
`#[spirv(...)]`で他にどのような機能が使えるかを知りたい方は[ソース](https://github.com/EmbarkStudios/rust-gpu/blob/main/crates/rustc_codegen_spirv/src/symbols.rs)をみてエスパーするとよいでしょう。
:::