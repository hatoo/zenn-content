---
title: "ashでシェーダーを実行する"
---

この章では前回rust-gpuで作ったシェーダーを[ash](https://github.com/MaikKlein/ash)でレンダリングします。ashはRustのためのVulkanラッパーです。この例だけなら[wgpu](https://github.com/gfx-rs/wgpu)を使えばよりシンプルになりますが、VKRの機能が使えないので今からashを使います。
といってもVulkanの説明は複雑すぎて自分にはできないので(すいません🙇‍♂️)、ashを使う際のポイントを書くだけにとどめておきます。
また、ネットでよく出てくるVulkanのチュートリアルはウインドウに描画するものが多いですがここでは直接画像ファイルに保存します(オフスクリーンレンダリング)。いくらVKRがリアルタイム用のAPIといっても後々Ray Tracing in One Weekendと同じ処理をするとどうしてもリアルタイムに描画することはできないからです。

# ashの使い方

基本的にashはVukkanの薄いラッパーなのでVulkanを触ったことをあればすぐにわかると思います。

structがBuilderパターンを対応しているので少し楽になります。Builderパターンを使うとVulkanの構造体でよくある`p***`と`***Count`もスライスで入力できます。


ashでRenderPassを作る例
```rust
let render_pass = {
    let color_attachments = [vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: COLOR_FORMAT,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    }];

    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let subpasses = [vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)
        .build()];

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachments)
        .subpasses(&subpasses)
        .build();

    unsafe { device.create_render_pass(&renderpass_create_info, None) }
        .expect("Failed to create render pass!")
};
```

このコードを見て`color_attachments`, `color_attachment_refs`, `SubpassDescription`を配列ではなく単体の中身だけで宣言してbuilderにスライスのリテラルを渡せば少しコードが見やすくなると考えるかもしれませんが、ここで大きな**注意点**があります。

上のコードをたとえば

```rust
let render_pass = {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: COLOR_FORMAT,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .build();

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&[color_attachment])
        .subpasses(&[subpass])
        .build();

    unsafe { device.create_render_pass(&renderpass_create_info, None) }
        .expect("Failed to create render pass!")
};
```

こんな風にすると**Debugビルドでは割と問題なく動きますがリリースビルドで落ちてしまう可能性があります**。
(想像するに、)なぜならashはVulkanのstructをそのままRustで表現しているので一時的に作った配列のライフタイムを保持できないからです。

例。ABIを保つためにポインタで持っている。
```rust
pub struct SubpassDescription {
    pub flags: SubpassDescriptionFlags,
    pub pipeline_bind_point: PipelineBindPoint,
    pub input_attachment_count: u32,
    pub p_input_attachments: *const AttachmentReference,
    pub color_attachment_count: u32,
    pub p_color_attachments: *const AttachmentReference,
    pub p_resolve_attachments: *const AttachmentReference,
    pub p_depth_stencil_attachment: *const AttachmentReference,
    pub preserve_attachment_count: u32,
    pub p_preserve_attachments: *const u32,
}
```

つまりRustコンパイラは一時的に作った`&[color_attachment_ref]`などをスコープから抜けた瞬間破棄する権利を持っています。
これがDebugビルドでは比較的動くけどReleaseビルドでは落ちてしまうことがある理由です。
