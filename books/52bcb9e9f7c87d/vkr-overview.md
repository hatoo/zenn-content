---
title: "Vulkan Raytracing外観"
---

# なぜVKRなのか

VKRはVulkanのレイトレーシング拡張で、[2020年12月](https://www.khronos.org/blog/vulkan-ray-tracing-final-specification-release)にSDKが配布されたようです。

単にGPUでレイトレーシングしたいならVKR拡張がなくともCompute Shaderを使えば実現できます([筆者が作った例](https://github.com/hatoo/rukako))。CPUよりはだいぶ速いです。

ではなぜわざわざ2018年1月からVKRの仕様を策定してきたかというと、(直接言及している文章を見たことがないので利用者目線から想像すると)Acceleration Structure(BVH)の構築とレイの当たり判定のハードウェアも含めた最適化をしたいからといっていいでしょう(と思っています)。

[Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html#boundingvolumehierarchies)のBounding Volume Hierarchiesをやった方ならわかるようにレイトレーシングのソフトウェアは多くの時間をレイの当たり判定に費やします。そこをGPUベンダに、ハードウェアも含めて最適化してもらえるのは非常にありがたいというわけです。

![Ray Accelerator](https://www.amd.com/system/files/2020-10/579976-hardware-accelerated-raytracing-1920x500.jpg)
[AMDのページ](https://www.amd.com/ja/technologies/rdna-2)にある図。左側はメモリやキャッシュ関係があると考えると(想像)そこそこ大きな面積にレイトレーシング用のハードウェア"Ray Accelerator"があり、すごそう。

# TODO

Add benchmarks