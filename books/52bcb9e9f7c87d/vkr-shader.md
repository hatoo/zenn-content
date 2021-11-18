---
title: "VKRのシェーダーを書く"
---

さっそくシェーダーを書いていきましょう。
コードは[こちら](https://github.com/hatoo/zenn-content/tree/master/raytracing-example)にあります。
文章内にコード片を記載していますが完全なコードではないのでリポジトリを確認してください。

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

```rust:shader/src/lib.rs
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

疑似乱数のアルゴリズムはいろいろありますが、今回は[PCGファミリ](https://www.pcg-random.org/index.html)の中から`pcg32si`を使うことにしました、GPUでは基本的に32bitアーキテクチャのようなので内部状態に32bitしか使わないものを選びました。周期が32bitと短めですが速さを期待します。

```rust:shader/src/rand.rs
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

# カメラを実装する

ピクセル座標から、どの位置からどの方向にレイを飛ばすかを決定します。
[Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)のカメラをそのまま持ってきます。

まず、`Ray`型を定義します。モーションブラーは今回は実装しないので位置と方向だけです。

```rust:shader/src/lib.rs
#[derive(Clone, Copy, Default)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}
```

`random_in_unit_disk`などの数学系の関数は`shader/src/math.rs`に実装することにします。

```rust:shader/src/math.rs
pub fn random_in_unit_disk(rng: &mut DefaultRng) -> Vec3 {
    loop {
        let p = vec3(
            rng.next_f32_range(-1.0, 1.0),
            rng.next_f32_range(-1.0, 1.0),
            0.0,
        );
        if p.length_squared() < 1.0 {
            break p;
        }
    }
}
```

```rust:shader/src/camera.rs
#[derive(Copy, Clone)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    // w: Vec3,
    lens_radius: f32,
}

impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        vup: Vec3,
        vfov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let theta = vfov;
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            // w,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32, rng: &mut DefaultRng) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(rng);
        let offset = self.u * rd.x + self.v * rd.y;

        Ray {
            origin: self.origin + offset,
            direction: (self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset),
        }
    }
}
```

先ほど実装した乱数生成器を使ってデフォーカス・ブラーを実装しています。

# RayPayload型を作成

Closest-Hit ShaderとMiss Shaderの返り値の型をここで定義します。レイを飛ばしたとき、{Closest-Hit, Miss} Shaderのどちらからも値が返ってくる可能性があるため、両方の返り値はもちろん同じ型でなければなりません。
その型を`RayPayload`型とします。

`RayPayload`型にどのような情報が欲しいかというと...

- レイは当たったのか? (Closest-HitかMissか？)
- Missだった場合
    - その色 (もう二度と反射したりしないのでこれだけでよい)
- Closest-Hitだった場合
    - 衝突位置
    - 法線の方向
    - マテリアルのindex (マテリアルのリストをStorage Bufferで渡してindexで参照するようにすることにします)
    - レイは表からあたったのか?裏からあたったのか?

の情報があれば十分です。

素直に考えれば上記は`enum`で表現できますが、ここで**注意点**があります。
Vulkanのメモリモデルは基本的にロジカルポインタです。つまり、ポインタに数値を足したり引いたりすることはできないしキャストすることもできません。rust-gpuにはポインタは存在せず、参照のみ存在すると考えると理解しやすいでしょう。

Rustの`enum`は、各バリアントに対してそれにマッチしてデータが欲しいときにデータ部分に対してキャストをします(上記のようにこれはできません!)。つまり事実上、rust-gpuでは`Option<T>`も含め`enum`を使うことは(現状)できません。

しょうがないので`struct`で表現し、内部の値によって使うメンバを変えることにします。
```rust:shader/src/lib.rs
#[derive(Clone, Default)]
pub struct RayPayload {
    // レイは当たったのか?
    pub is_miss: bool,
    // Missの場合その色。Closest-Hitの場合その位置
    pub position: Vec3,
    // 法線
    pub normal: Vec3,
    // マテリアルの番号
    pub material: u32,
    // 表から例が当たったのかどうか　
    pub front_face: bool,
}
```
