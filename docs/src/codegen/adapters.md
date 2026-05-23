# Runtime Adapters

Some languages do not have a built-in dependency story for every runtime format. Those targets use adapter hooks instead of embedding a third-party decoder.

The generated runtime owns the Sora value model and table loading logic. The application supplies a small function that turns bytes into the decoded value tree expected by the runtime.

This keeps generated code independent from dependency choices. A game can use the CBOR, Protobuf, or compression library it already trusts.

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

Adapters keep generated code independent from dependency choices while still allowing the same exported data formats to be used.

## What the Adapter Must Return

The adapter should return the language-specific Sora value tree expected by the generated runtime. It is not responsible for constructing typed rows; generated code handles that after decoding.

If a target has a self-contained decoder for a format, no adapter is needed.
