#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parents[1]
SHOWCASE = ROOT / "examples" / "showcase"


@dataclass(frozen=True)
class Check:
    name: str
    tools: tuple[str, ...]
    run: Callable[[], None]


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify generated showcase runtimes.")
    parser.add_argument(
        "--only",
        help="Comma-separated check names to run. Defaults to every known check.",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when a selected check is missing its required tool.",
    )
    args = parser.parse_args()

    requested = set(args.only.split(",")) if args.only else None
    checks = all_checks()
    selected = [check for check in checks if requested is None or check.name in requested]
    unknown = sorted(requested.difference(check.name for check in checks)) if requested else []
    if unknown:
        print(f"unknown showcase checks: {', '.join(unknown)}", file=sys.stderr)
        return 2

    failed = False
    skipped: list[str] = []
    for check in selected:
        missing = [tool for tool in check.tools if shutil.which(tool) is None]
        if missing:
            message = f"skip {check.name}: missing {', '.join(missing)}"
            if args.strict:
                print(message, file=sys.stderr)
                failed = True
            else:
                print(message, flush=True)
                skipped.append(check.name)
            continue

        print(f"==> {check.name}", flush=True)
        try:
            check.run()
        except subprocess.CalledProcessError as error:
            print(f"{check.name} failed with exit code {error.returncode}", file=sys.stderr)
            failed = True

    if skipped:
        print(f"skipped showcase checks: {', '.join(skipped)}", flush=True)
    return 1 if failed else 0


def all_checks() -> list[Check]:
    return [
        Check("rust", ("cargo",), check_rust),
        Check("kotlin", gradle_tools("kotlin"), check_kotlin),
        Check("csharp", ("dotnet",), check_csharp),
        Check("java", gradle_tools("java"), check_java),
        Check("scala", ("sbt",), check_scala),
        Check("go", ("go",), check_go),
        Check("python", (python_tool(),), check_python),
        Check("typescript", ("tsc",), check_typescript),
        Check("javascript", ("node",), check_javascript),
        Check("dart", ("dart",), check_dart),
        Check("lua", ("luac",), check_lua),
        Check("erlang", ("erlc",), check_erlang),
        Check("c", (c_compiler(),), check_c),
        Check("cpp", (cpp_compiler(),), check_cpp),
        Check("godot", ("godot",), check_godot),
    ]


def gradle_tools(project: str) -> tuple[str, ...]:
    wrapper = SHOWCASE / project / ("gradlew.bat" if os.name == "nt" else "gradlew")
    return (str(wrapper),)


def python_tool() -> str:
    return sys.executable


def c_compiler() -> str:
    for candidate in ("cc", "gcc", "clang", "cl"):
        if shutil.which(candidate):
            return candidate
    return "cc"


def cpp_compiler() -> str:
    for candidate in ("c++", "g++", "clang++", "cl"):
        if shutil.which(candidate):
            return candidate
    return "c++"


def run(command: list[str], *, cwd: Path | None = None) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd or ROOT, check=True)


def check_rust() -> None:
    run(["cargo", "run", "-p", "sora-showcase-rust"])


def check_kotlin() -> None:
    run([str(SHOWCASE / "kotlin" / gradle_name()), "run", "--no-daemon"], cwd=SHOWCASE / "kotlin")


def check_csharp() -> None:
    run(["dotnet", "run", "--project", str(SHOWCASE / "csharp" / "SoraShowcase.csproj")])


def check_java() -> None:
    run([str(SHOWCASE / "java" / gradle_name()), "run", "--no-daemon"], cwd=SHOWCASE / "java")


def check_scala() -> None:
    run(["sbt", "run"], cwd=SHOWCASE / "scala")


def check_go() -> None:
    run(["go", "run", "./cmd/showcase"], cwd=SHOWCASE / "go")


def check_python() -> None:
    run([python_tool(), str(SHOWCASE / "python" / "main.py")])


def check_typescript() -> None:
    run(["tsc", "--noEmit", "-p", str(SHOWCASE / "typescript" / "tsconfig.json")])


def check_javascript() -> None:
    generated = sorted((SHOWCASE / "javascript" / "generated").glob("*.js"))
    for path in generated:
        run(["node", "--check", str(path)])
    run(["node", str(SHOWCASE / "javascript" / "main.mjs")])


def check_dart() -> None:
    run(["dart", "pub", "get"], cwd=SHOWCASE / "dart")
    run(["dart", "analyze"], cwd=SHOWCASE / "dart")
    run(["dart", "run", "bin/showcase.dart"], cwd=SHOWCASE / "dart")


def check_lua() -> None:
    for path in sorted((SHOWCASE / "lua" / "generated").glob("*.lua")):
        run(["luac", "-p", str(path)])


def check_erlang() -> None:
    with tempfile.TemporaryDirectory(prefix="sora-erlang-") as temp:
        command = ["erlc", "-o", temp]
        command.extend(str(path) for path in sorted((SHOWCASE / "erlang" / "generated").glob("*.erl")))
        run(command)


def check_c() -> None:
    compiler = c_compiler()
    sources = sorted((SHOWCASE / "c" / "generated").glob("*.c"))
    with tempfile.TemporaryDirectory(prefix="sora-c-") as temp:
        for source in sources:
            output = Path(temp) / f"{source.stem}.o"
            if compiler == "cl":
                run(["cl", "/nologo", "/std:c11", "/I", str(source.parent), "/c", str(source), f"/Fo{output}"])
            else:
                run([compiler, "-std=c11", "-Wall", "-Wextra", "-I", str(source.parent), "-c", str(source), "-o", str(output)])


def check_cpp() -> None:
    compiler = cpp_compiler()
    with tempfile.TemporaryDirectory(prefix="sora-cpp-") as temp:
        source = Path(temp) / "verify.cpp"
        source.write_text('#include "sora_config.hpp"\nint main() { return 0; }\n', encoding="utf-8")
        output = Path(temp) / ("verify.obj" if compiler == "cl" else "verify.o")
        include = SHOWCASE / "cpp" / "generated"
        if compiler == "cl":
            run(["cl", "/nologo", "/std:c++17", "/I", str(include), "/c", str(source), f"/Fo{output}"])
        else:
            run([compiler, "-std=c++17", "-Wall", "-Wextra", "-I", str(include), "-c", str(source), "-o", str(output)])


def check_godot() -> None:
    run(["godot", "--headless", "--path", str(SHOWCASE / "godot"), "--quit"])


def gradle_name() -> str:
    return "gradlew.bat" if os.name == "nt" else "./gradlew"


if __name__ == "__main__":
    raise SystemExit(main())
