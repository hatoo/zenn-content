---
title: "Vulkan Raytracing外観"
---

# なぜVKRなのか

VKRはVulkanのレイトレーシング拡張で、[2020年12月](https://www.khronos.org/blog/vulkan-ray-tracing-final-specification-release)にSDKが配布されたようです。

:::message
Vulkanのレイトレーシング関係のチュートリアルをネットで探すと`VK_NV_ray_tracing`などのNvidia独自の拡張を使ったものが見つかるかもしれません。これは古いAPIなので無視したほうがよいでしょう。
といっても現在標準のKHR拡張とそこまで大きな違いがあるわけではありません。
:::

単にGPUでレイトレーシングしたいならVKR拡張がなくともCompute Shaderを使えば実現できます([筆者が作った例](https://github.com/hatoo/rukako))。CPUよりはだいぶ速いです。

ではなぜわざわざ2018年1月からVKRの仕様を策定してきたかというと、(直接言及している文章を見たことがないので利用者目線から想像すると)Acceleration Structure(BVH)の構築とレイの当たり判定をハードウェアも含めて最適化したいからといっていいでしょう(と思っています)。

[Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html#boundingvolumehierarchies)のBounding Volume Hierarchiesをやった方ならわかるように、レイトレーシングのソフトウェアは多くの時間をレイの当たり判定に費やします。そこをGPUベンダに、ハードウェアも含めて最適化してもらえるのは非常にありがたいというわけです。

![Ray Accelerator](https://www.amd.com/system/files/2020-10/579976-hardware-accelerated-raytracing-1920x500.jpg)
[AMDのページ](https://www.amd.com/ja/technologies/rdna-2)にある図。左側はメモリやキャッシュ関係があると考えると(想像)そこそこ大きな面積にレイトレーシング用のハードウェア"Ray Accelerator"があり、すごそう。

## ベンチマーク

大体同じシーンをCPU, GPU(Compute Shader), GPU(VKR)でレンダリングして時間を計測してみました。計測時間はプログラムの最初から最後までなのでセットアップにかかった時間なども含んでいます。

1200x800ピクセル1000サンプルです

- CPU: 3950x
- GPU: RTX 2080ti

:::message alert
ベンチマークに使用したプログラムは筆者が作ったものですが全然最適化されていません。
あくまで参考程度にとらえてください。
:::

|   | CPU | GPU(Compute Shader) | GPU(VKR) |
| - | --- | ------------------- | -------- |
| 時間(秒) | 55.7 | 17.3 | 3.2 |

![bench](/images/bench.png)

VKRだと速い。BVHの最適化を丸投げできるのがとても良い。