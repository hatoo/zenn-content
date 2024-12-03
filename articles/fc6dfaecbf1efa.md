---
title: "Rustの`std::io::Write::write`が`Ok(0)`を返すことについて"
emoji: "🦀"
type: "tech" # tech: 技術記事 / idea: アイデア
topics: ["rust"]
published: false
---

# はじめに

先日、[`hyper`](https://github.com/hyperium/hyper)のストリーム^[具体的には`hyper`のHTTP/2サーバーでUpgradeされたストリーム]を[`tokio-rustls`](https://github.com/rustls/tokio-rustls)でラップした際に、`tokio::io::AsyncWrite::poll_write`が返す`Ok(0)`の解釈の違いによって問題が出たので、そのときに調べたことをまとめます
こちらは厳密には`tokio`の`AsyncWrite`の話ですが、実質同じなので`std::io::Write`の話として記事を続けます

そのときのissue
- https://github.com/rustls/tokio-rustls/issues/92
- https://github.com/hyperium/hyper/issues/3801

# 起きたこと

こちらのコードは`tokio-rustls`にあるTLSストリームの実装の一部です

https://github.com/rustls/tokio-rustls/blob/66fb0ae98fbc9e71d5aa855d45e88ca8d53f95f3/src/common/mod.rs#L330-L334

ここではshutdownの際にwhileループでなにやら今まで書ききれてなかったデータを書き込もうとしています
しかし、`tokio-rustls`が`hyper`のストリームをラップしていた場合、この段階でこの`write`に対して常に`Ok(0)`を返すので`self.session.wants_write()`が一生`true`のままになり無限ループに陥ってしまいます

結局、https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.write には

> A return value of Ok(0) typically means that the underlying object is no longer able to accept bytes and will likely not be able to in the future as well, or that the buffer provided is empty.

と書かれているので、`hyper`側が`Ok(0)`を返すのは正しい挙動で、`rust-tls`側が受け取った`Ok(0)`を特別扱いする必要があるということでまとまりました

# `std::io::Write::write`が`Ok(0)`を返すことについての第一印象

しかし https://github.com/rustls/tokio-rustls/issues/92#issuecomment-2507878251 のコメントの通り、ストリームが`もうこれ以上書き込めないよ`ということを表明するために`Ok(0)`を返すのはあんまりなデザインだと思わざるを得ません
POSIXの`write(2)`にもそのような話はないっぽい
普通に考えて、仮に[`std::io::Write::write_all`](https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all)を自分で実装するとしたら、自分も上記の`tokio-rustls`のコードみたいに書く自信があります

できれば`もうこれ以上書き込めないよ`というときには https://doc.rust-lang.org/std/io/enum.ErrorKind.html から`WriteZero`とかそれっぽいやつを選んで返してほしいものです

ちなみに[`std::io::Write::write_all`](https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all)は`Ok(0)`を特別に扱っているのでそれが原因で無限ループすることはありません
https://doc.rust-lang.org/src/std/io/mod.rs.html#1703-1715

# rustc内の議論

上記のデザイン上の疑問について調べていたところ、目を引くissueが見つかりました
https://github.com/rust-lang/rust/issues/56889

主題は`std::io::Write::write_all`が`Ok(0)`をどう扱うかについてですが

https://github.com/rust-lang/rust/issues/56889#issuecomment-740110530

> IMHO a sane writer should return an error when it is already at the end. This would be in line with eg. a block device (a typical size-limited object to write to...). Ok(0) doesn't really make any sense to me. If you can't write anything yet, return EWOULDBLOCK (or, well, block), if you can't write anything because of some error, return the error. But don't just do nothing (or purely some unrelated internal stuff) and return 0, that makes no sense.
So in that sense, treating Ok(0) as an error condition is acceptable, because it's something that shouldn't happen anyway. (And yes, IMO the slice impl could just as well be changed)

というコメントがあり、個人的に納得しました。かなりまとめると`write`に`Ok(0)`を返すのはそもそも異常なので、受け取った側がそれをエラーとして扱うのは妥当だということです

# まとめ

個人的には以下のような結論に至りました。

- `std::io::Write::write`に対して`Ok(0)`が帰ってきた場合、受け取った側はそれをエラーとして扱うべき
- だからといって`std::io::Write::write`実装側が`もうこれ以上書き込めないよ`というときに`Ok(0)`を返すのは良くない。普通に`std::io::Error`を返すべき
- `Result<std::num::NonZeroUsize>`返せよ。Breaking Changeなのを差し引いてもさすがにこれはオタクすぎるか
- `std::io::Write::write_all`は`Ok(0)`をエラーとして扱う。無限ループするわけにはいかないので^[関連するLibs-API Meeting https://hackmd.io/@rust-libs/SJUBKd-lK]これは正しい
- `std::io::Write::write_all`がそういう挙動をしている以上 https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.write に *A return value of Ok(0) typically means that the underlying object is no longer able to accept bytes and will likely not be able to in the future as well, or that the buffer provided is empty.* と書くのは妥当