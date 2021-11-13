---
title: "rust-gpuを使う"
---

[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)はRustのコードを[SPIR-V](https://en.wikipedia.org/wiki/Standard_Portable_Intermediate_Representation)にコンパイルするツールです。Vulkanを使いSPIR-VをGPU上で動かすことができます。
この文章ではrust-gpuでレイトレーシングを行うことを目的としていますが、この章ではまずrust-gpuで簡単なラスタライズ用のシェーダーを作っていきます。

# セットアップ

