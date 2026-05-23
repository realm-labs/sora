# Sora Showcase Example

This example is intentionally much larger than `examples/simple`.

It keeps real Excel workbooks in `data/` and generated outputs in `generated/`
so the whole pipeline is easy to inspect. The showcase currently covers 34
tables across core, battle, economy, quest, and complex-data domains, with
hundreds of rows and mixed map/list/singleton table modes.

`data/Complex.xlsx` is the stress case for Sora's Excel projection. It shows
single `union<T>` values edited through `tagged_columns`, non-JSON
`list<union<T>>` values assembled from child rows, derived fields that point at
other derived groups, and a nested tuple cell that combines struct, tuple-list,
split, and map parsers.

`data/Core.xlsx` also includes smaller coverage examples for singleton tables,
`f64`, fixed-size arrays, and an optional struct derived from a child table.

- schema: `project.toml` and `schema/game.toml`
- Excel data: `data/Core.xlsx`, `data/Battle.xlsx`, `data/Economy.xlsx`, `data/Quest.xlsx`, `data/Complex.xlsx`
- Rust Cargo project: `rust`
- Kotlin Gradle project: `kotlin`
- C# .NET project: `csharp`
- Java Gradle project: `java`
- Scala sbt project: `scala`
- Go module: `go`
- Lua generated modules with EmmyLua annotations: `lua/generated` (configured for Lua 5.4)
- binary bundle: `generated/config.sora`
- debug JSON: `generated/debug-json`
- schema lock: `generated/schema.lock`

The Kotlin project uses Gradle Wrapper 9.5.1, Kotlin JVM plugin 2.3.20, and a
JDK 21 toolchain.

The Scala project uses sbt and Scala 3.3.3.

Regenerate everything:

```powershell
cargo run -p sora-showcase-builder
```

Verify every showcase runtime supported by the tools installed on the machine:

```powershell
uv run python scripts/verify_showcase.py
```

CI runs the same verifier in strict mode for Rust, Kotlin, C#, Java, Scala, Go,
Python, TypeScript, JavaScript, Dart, Lua, Erlang, C, and C++.

Run the Rust smoke example:

```powershell
cargo run -p sora-showcase-rust
```

Run the Kotlin smoke example with the checked-in Gradle wrapper:

```powershell
Push-Location examples/showcase/kotlin
.\gradlew.bat run
Pop-Location
```

Run the C# smoke example:

```powershell
dotnet run --project examples/showcase/csharp/SoraShowcase.csproj
```

Run the Java smoke example with the checked-in Gradle wrapper:

```powershell
Push-Location examples/showcase/java
.\gradlew.bat run
Pop-Location
```

Run the Scala smoke example:

```powershell
Push-Location examples/showcase/scala
sbt run
Pop-Location
```

Run the Go smoke example:

```powershell
Push-Location examples/showcase/go
go run ./cmd/showcase
Pop-Location
```

Run the Python smoke example:

```powershell
python3 examples/showcase/python/main.py
```

Check generated Lua syntax with a local Lua 5.3+ runtime:

```powershell
Get-ChildItem examples/showcase/lua/generated -Filter *.lua | ForEach-Object { luac -p $_.FullName }
```
