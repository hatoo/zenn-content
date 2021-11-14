---
title: "はじめに"
---

この文章では[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)と[ash](https://github.com/MaikKlein/ash)を使い、Vulkan Raytracing extension(以下VKR)を用いたレイトレーシングを行います。Vulkan Raytracing extensionを使うことによりCPUより高速にレイトレーシングを行うことができます。また、[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)を使うことでGPUで動くコードも含めすべてRustで書いていきます。

# 対象

この文章は、

- Rust
- Vulkan
    - [Vulkan Tutorial](https://vulkan-tutorial.com/Introduction)のDrawing a Triangleまで
- レイトレーシング
    - [Ray Tracing in One Weekend — The Book Series](https://raytracing.github.io/)のThe Next WeekのBounding Volume Hierarchiesまで

の知識がある方を対象としています。

想定としては[Ray Tracing in One Weekend — The Book Series](https://raytracing.github.io/)をやったけどGPUで動かしたくなった方がターゲットです。
VKR以外のVulkanの説明は複雑すぎてこの文章では扱えませんが触ったことのない方は[Vulkan Tutorial](https://vulkan-tutorial.com/Introduction)のDrawing a Triangleまでやっておけばよいです。

# 動作環境

本文章で使われているコードは

- OS: Windows 11
- GPU: RTX 2080ti
- Vulkan SDK: 1.2.189.2

で動作を確認しました。
