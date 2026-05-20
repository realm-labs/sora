# Sora Showcase Example

This example is intentionally much larger than `examples/simple`.

It keeps real Excel workbooks in `data/` and generated outputs in `generated/`
so the whole pipeline is easy to inspect. The showcase currently covers 27
tables across core, battle, economy, and quest domains, with hundreds of rows
and mixed map/list/singleton table modes.

- schema: `project.toml` and `schema/game.toml`
- Excel data: `data/Core.xlsx`, `data/Battle.xlsx`, `data/Economy.xlsx`, `data/Quest.xlsx`
- Rust Cargo project: `rust`
- Kotlin Gradle project: `kotlin`
- C# .NET project: `csharp`
- Java Gradle project: `java`
- Go module: `go`
- Lua generated modules with EmmyLua annotations: `lua/generated`
- binary bundle: `generated/config.sora`
- debug JSON: `generated/debug-json`
- schema lock: `generated/schema.lock`

The Kotlin project uses Gradle Wrapper 9.5.1, Kotlin JVM plugin 2.3.20, and a
JDK 21 toolchain.

Regenerate everything:

```powershell
cargo run -p sora-showcase-builder
```

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

Run the Go smoke example:

```powershell
Push-Location examples/showcase/go
go run ./cmd/showcase
Pop-Location
```

Check generated Lua syntax with a local Lua 5.3+ runtime:

```powershell
Get-ChildItem examples/showcase/lua/generated -Filter *.lua | ForEach-Object { luac -p $_.FullName }
```
