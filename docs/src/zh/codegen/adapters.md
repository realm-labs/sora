# 运行时适配器

有些语言并没有适合每个 runtime format 的内置依赖方案。这些目标会使用 adapter hook，而不是在生成代码中嵌入某个第三方 decoder。

生成 runtime 负责 Sora value model 和 table loading 逻辑。应用只需要提供一个小函数，把 bytes 转成 runtime 期望的 decoded value tree。

这让生成代码不绑定具体依赖。游戏可以使用自己已经信任的 CBOR、Protobuf 或压缩库。

## Lua

```lua
local config = SoraConfig.from_cbor(bytes, {
  decode_cbor = function(payload)
    return my_cbor.decode(payload)
  end,
})
```

## Erlang

```erlang
Options = #{
    decode_cbor => fun my_cbor:decode/1
},
Config = sora_config:from_cbor(Bytes, Options).
```

## Dart

```dart
final config = SoraConfig.fromCbor(
  bytes,
  decodeCbor: (payload) => myCborDecode(payload),
);
```

Adapter 让生成代码独立于依赖选择，同时仍能使用相同的导出数据格式。

## Adapter 返回什么

adapter 应该返回生成 runtime 期望的目标语言 Sora value tree。它不负责构造 typed row；解码后的类型化构造由生成代码处理。

如果某个 target 对某个格式有 self-contained decoder，就不需要 adapter。
