---
title: "VKRのシェーダーを書く"
---

さっそくシェーダーを書いていきましょう。
コードは[こちら](https://github.com/hatoo/zenn-content/tree/master/raytracing-example)にあります。

まず、2章と同じセットアップをしてください。
rust-gpuでレイトレーシング拡張を有効にするために`build.rs`を変更します。
`bool`型はRustでは8ビットなので`Int8`も有効にします。

```rust:build.rs
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
```

# 疑似乱数を実装する

疑似乱数はレイトレーシングのいたるところで使用されますがSPIR-Vには[rand(3)](https://linuxjm.osdn.jp/html/LDP_man-pages/man3/rand.3.html)のようなものはありませんし、[randクレート](https://crates.io/crates/rand)もコンパイルできません。
なので自分で実装していきます。

まず、疑似乱数のシードを得ようと思います。
今回は、ピクセルの座標とホストで作った乱数(Push Constantsで渡す)をxorしてシードとします。

```rust
pub struct PushConstants {
    seed: u32,
}

// 前章で書いたようにレイトレーシングのエントリポイントはRay Generation Shader
#[spirv(ray_generation)]
pub fn main_ray_generation(
    // 対象のピクセルの座標
    #[spirv(launch_id)] launch_id: UVec3,
    // 出力のサイズ
    #[spirv(launch_size)] launch_size: UVec3,
    // Push Constants
    #[spirv(push_constant)] constants: &PushConstants,
) {
    let rand_seed = (launch_id.y * launch_size.x + launch_id.x) ^ constants.seed;
}
```

疑似乱数のアルゴリズムはいろいろありますが、今回は[PCGファミリ](https://www.pcg-random.org/index.html)の中から`pcg32si`を使うことにしました、GPUでは基本的に32bitアーキテクチャのようなので内部状態に32bitしか使わないものを選びました。多少、暗号的な耐性が下がると思いますが速さを期待します。

```rust:src/rand.rs
pub struct PCG32si {
    state: u32,
}

impl PCG32si {
    const PCG_DEFAULT_MULTIPLIER_32: u32 = 747796405;
    const PCG_DEFAULT_INCREMENT_32: u32 = 2891336453;

    fn pcg_oneseq_32_step_r(&mut self) {
        self.state = self
            .state
            .wrapping_mul(Self::PCG_DEFAULT_MULTIPLIER_32)
            .wrapping_add(Self::PCG_DEFAULT_INCREMENT_32);
    }

    fn pcg_output_rxs_m_xs_32_32(state: u32) -> u32 {
        let word = ((state >> ((state >> 28).wrapping_add(4))) ^ state).wrapping_mul(277803737);
        (word >> 22) ^ word
    }

    pub fn new(seed: u32) -> Self {
        let mut rng = Self { state: seed };
        rng.pcg_oneseq_32_step_r();
        rng.state = rng.state.wrapping_add(seed);
        rng.pcg_oneseq_32_step_r();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.pcg_oneseq_32_step_r();
        Self::pcg_output_rxs_m_xs_32_32(old_state)
    }

    // 0.0..1.0
    pub fn next_f32(&mut self) -> f32 {
        let float_size = core::mem::size_of::<f32>() as u32 * 8;
        let precision = 23 + 1;
        let scale = 1.0 / ((1 << precision) as f32);

        let value = self.next_u32();
        let value = value >> (float_size - precision);
        scale * value as f32
    }

    pub fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

pub type DefaultRng = PCG32si;
```

実装は[PCGのC実装](https://github.com/imneme/pcg-c)からそのまま持ってきました。
`next_f32`は[randクレート](https://github.com/rust-random/rand/blob/master/src/distributions/float.rs#L107)から持ってきました。
また簡単のために、どうせこれ以上の乱数生成器を作る予定もないのでトレイトで抽象化をせず、`DefaultRng`として公開しています。
