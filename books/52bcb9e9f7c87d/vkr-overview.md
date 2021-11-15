---
title: "Vulkan Raytracing外観"
---

# なぜVKRなのか

VKRはVulkanのレイトレーシング拡張で、[2020年12月](https://www.khronos.org/blog/vulkan-ray-tracing-final-specification-release)にSDKが配布されたようです。

:::message
Vulkanのレイトレーシング関係のチュートリアルをネットで探すと`VK_NV_ray_tracing`などのNvidia独自の拡張を使ったものが見つかるかもしれません。これは古いAPIなので避けたほうが良いかしれません。
といっても現在標準のKHR拡張とそこまで大きな違いがあるわけではありません。
:::

単にGPUでレイトレーシングしたいならVKR拡張がなくてもCompute Shaderを使えば実現できます([筆者が作った例](https://github.com/hatoo/rukako))。CPUよりはだいぶ速いです。

ではなぜわざわざ2018年1月からVKRの仕様を策定してきたかというと、(直接言及している文章を見たことがないので利用者目線から想像すると)BVH(VKRではAcceleration Structureと言います)の構築とレイの当たり判定をハードウェアも含めて最適化したいからといっていいでしょう(と思っています)。

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

# Acceleration Structure

前節で、VKRの主なモチベーションはAcceleration Structure(以下AS)にある(と思う)と述べました。
ASとはBVHと同じ働きをします。APIを呼ぶことでGPU上で構築したりシェーダーからレイの当たり判定をすることができます。(実際にはASの実装はGPUベンダの裁量によるので中でBVHとは違うすごいアルゴリズムが使われているかもしれませんが)
またASは、特定の条件を満たした際に低コストで再構築したり、シリアライズして例えば他のGPUで構築したASを他のGPUで使ったりする面白い機能もありますがこの文章では触れません。

## Top Level Acceleration StructureとBottom Level Acceleration Structure

ASはTop Level Acceleration Structure(以下TLAS)とBottom Level Acceleration Structure(BLAS)の二層構造です。
シェーダーからはTLASのみが見え、TLASは複数のBLASをその変換行列とともに持ちます。BLASは複数のポリゴンもしくはAABBを持ちます。AABBの場合はその内部での当たり判定は独自のシェーダーを定義して計算します。TLASがポリゴン/AABBを保持することもBLASが他のBLASやTLASを持つこともありません。

![Acceleration Structure](/images/acceleration_structure.png)


