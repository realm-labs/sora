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


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "examples" / "simple" / "schema" / "items.toml"


@dataclass(frozen=True)
class RuntimeCase:
    target: str
    runtime_format: str
    checks: tuple[str, ...] = ()


CASES: tuple[RuntimeCase, ...] = (
    *(
        RuntimeCase(target, runtime_format)
        for target in (
            "rust",
            "kotlin",
            "csharp",
            "java",
            "scala",
            "go",
            "typescript",
        )
        for runtime_format in ("sora", "json", "cbor", "sora-protobuf")
    ),
    *(
        RuntimeCase(target, runtime_format, ("node-check",))
        for target in ("javascript",)
        for runtime_format in ("sora", "json", "cbor", "sora-protobuf")
    ),
    *(
        RuntimeCase(target, runtime_format, ("py-compile",))
        for target in ("python",)
        for runtime_format in ("sora", "json", "cbor", "sora-protobuf")
    ),
    *(
        RuntimeCase("dart", runtime_format, ("dart-analyze",))
        for runtime_format in ("json", "cbor", "sora-protobuf")
    ),
    RuntimeCase("godot", "json"),
    RuntimeCase("c", "sora", ("cc",)),
    RuntimeCase("cpp", "sora", ("cxx",)),
    *(
        RuntimeCase("erlang", runtime_format, ("erlc",))
        for runtime_format in ("sora", "json", "cbor", "sora-protobuf")
    ),
    *(
        RuntimeCase("lua", runtime_format, ("luac",))
        for runtime_format in ("sora", "json", "cbor", "sora-protobuf")
    ),
)


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify generated code for supported runtime formats.")
    parser.add_argument(
        "--sora-bin",
        default=str(ROOT / "target" / "debug" / ("sora.exe" if os.name == "nt" else "sora")),
        help="Path to the compiled sora CLI binary.",
    )
    parser.add_argument(
        "--out",
        type=Path,
        help="Directory for generated matrix output. Defaults to a temporary directory.",
    )
    parser.add_argument(
        "--strict-tools",
        action="store_true",
        help="Fail instead of skipping syntax checks when an optional tool is missing.",
    )
    args = parser.parse_args()

    sora_bin = Path(args.sora_bin)
    if not sora_bin.exists():
        print(f"sora binary not found: {sora_bin}", file=sys.stderr)
        return 2

    if args.out is not None:
        args.out.mkdir(parents=True, exist_ok=True)
        return run_matrix(sora_bin, args.out, args.strict_tools)

    with tempfile.TemporaryDirectory(prefix="sora-runtime-matrix-") as temp:
        return run_matrix(sora_bin, Path(temp), args.strict_tools)


def run_matrix(sora_bin: Path, base: Path, strict_tools: bool) -> int:
    failed = False
    skipped: list[str] = []
    for case in CASES:
        label = f"{case.target}:{case.runtime_format}"
        out_dir = base / case.target / runtime_slug(case.runtime_format)
        project = write_project(base, case)
        print(f"==> {label}", flush=True)
        try:
            run(
                [
                    str(sora_bin),
                    "gen",
                    "--target",
                    case.target,
                    "--project",
                    str(project),
                    "--out",
                    str(out_dir),
                    "--format-code",
                    "never",
                ]
            )
            verify_expected_runtime_marker(case, out_dir)
            for check in case.checks:
                missing = missing_tool(check)
                if missing is not None:
                    message = f"skip {label} {check}: missing {missing}"
                    if strict_tools:
                        print(message, file=sys.stderr)
                        failed = True
                    else:
                        print(message, flush=True)
                        skipped.append(f"{label}:{check}")
                    continue
                run_check(check, out_dir)
        except subprocess.CalledProcessError as error:
            print(f"{label} failed with exit code {error.returncode}", file=sys.stderr)
            failed = True
        except AssertionError as error:
            print(f"{label} failed: {error}", file=sys.stderr)
            failed = True

    if skipped:
        print(f"skipped checks: {', '.join(skipped)}", flush=True)
    return 1 if failed else 0


def write_project(base: Path, case: RuntimeCase) -> Path:
    project_dir = base / "_projects" / case.target / runtime_slug(case.runtime_format)
    project_dir.mkdir(parents=True, exist_ok=True)
    project = project_dir / "project.toml"
    options = [f'runtime_format = "{case.runtime_format}"']
    if case.target == "scala":
        options.append('scala_version = "3"')
    if case.target == "lua":
        options.append('lua_version = "5.4"')
    if case.target == "erlang":
        options.append('enum_repr = "atom"')
    if case.target == "typescript":
        options.append('enum_repr = "string"')
    if case.target == "javascript":
        options.append('enum_repr = "string"')
        options.append("emit_dts = true")
    if case.target == "c":
        options.append('c_standard = "c11"')
    if case.target == "cpp":
        options.append('cpp_standard = "c++17"')

    project.write_text(
        "\n".join(
            [
                'package = "game_config"',
                f'includes = ["{SCHEMA.as_posix()}"]',
                "",
                f"[codegen.{case.target}]",
                *options,
                "",
            ]
        ),
        encoding="utf-8",
    )
    return project


def verify_expected_runtime_marker(case: RuntimeCase, out_dir: Path) -> None:
    runtime = read_runtime(case.target, out_dir)
    if case.runtime_format == "sora":
        expected = (
            "SoraBundle",
            "sora_bundle",
            "ParseSoraBundle",
            "parse_bundle",
            "sora_bundle_parse",
        )
        if not any(marker in runtime for marker in expected):
            raise AssertionError("runtime does not contain marker for sora")
        return
    expected = {
        "json": (
            "parseJson",
            "parse_json",
            "parse_json_bundle",
            "JsonBundle",
            "ParseJson",
            "ParseJsonBundle",
            "parseJsonBundle",
        ),
        "cbor": (
            "parseCbor",
            "parse_cbor",
            "parse_cbor_bundle",
            "CborBundle",
            "ParseCbor",
            "ParseCborBundle",
            "parseCborBundle",
            "decode_cbor",
        ),
        "sora-protobuf": (
            "parseProtobuf",
            "parse_protobuf",
            "parse_protobuf_bundle",
            "ProtobufBundle",
            "ParseProtobuf",
            "ParseProtobufBundle",
            "parseProtobufBundle",
            "decode_protobuf",
        ),
    }[case.runtime_format]
    if not any(marker in runtime for marker in expected):
        raise AssertionError(f"runtime does not contain marker for {case.runtime_format}")


def read_runtime(target: str, out_dir: Path) -> str:
    candidates = {
        "rust": ("runtime.rs", "mod.rs"),
        "kotlin": ("Runtime.kt",),
        "csharp": ("Runtime.cs",),
        "java": ("Runtime.java",),
        "scala": ("SoraRuntime.scala",),
        "go": ("runtime.go",),
        "typescript": ("runtime.ts", "sora_runtime.ts"),
        "javascript": ("runtime.js", "sora_runtime.js"),
        "python": ("runtime.py", "sora_runtime.py"),
        "dart": ("runtime.dart",),
        "godot": ("runtime.gd", "sora_runtime.gd"),
        "c": ("sora_runtime.c", "runtime.c"),
        "cpp": ("sora_runtime.hpp", "runtime.hpp"),
        "erlang": ("sora_runtime.erl",),
        "lua": ("sora_runtime.lua",),
    }[target]
    for name in candidates:
        matches = list(out_dir.rglob(name))
        if matches:
            return matches[0].read_text(encoding="utf-8")
    raise AssertionError(f"runtime file not found in {out_dir}")


def missing_tool(check: str) -> str | None:
    tool = {
        "node-check": "node",
        "py-compile": sys.executable,
        "dart-analyze": "dart",
        "erlc": "erlc",
        "luac": "luac",
        "cc": c_compiler(),
        "cxx": cxx_compiler(),
    }[check]
    return None if shutil.which(tool) else tool


def run_check(check: str, out_dir: Path) -> None:
    if check == "node-check":
        for path in sorted(out_dir.rglob("*.js")):
            run(["node", "--check", str(path)])
        return
    if check == "py-compile":
        run([sys.executable, "-m", "compileall", "-q", str(out_dir)])
        return
    if check == "dart-analyze":
        (out_dir / "pubspec.yaml").write_text(
            "name: sora_runtime_matrix\n"
            "publish_to: none\n"
            "environment:\n"
            "  sdk: ^3.10.0\n",
            encoding="utf-8",
        )
        (out_dir / "analysis_options.yaml").write_text(
            "analyzer:\n"
            "  errors:\n"
            "    unused_import: ignore\n",
            encoding="utf-8",
        )
        run(["dart", "analyze", str(out_dir)])
        return
    if check == "erlc":
        beam_dir = out_dir / "_beam"
        beam_dir.mkdir(exist_ok=True)
        run(["erlc", "-o", str(beam_dir), *[str(path) for path in sorted(out_dir.rglob("*.erl"))]])
        return
    if check == "luac":
        for path in sorted(out_dir.rglob("*.lua")):
            run(["luac", "-p", str(path)])
        return
    if check == "cc":
        compiler = c_compiler()
        obj_dir = out_dir / "_obj"
        obj_dir.mkdir(exist_ok=True)
        for source in sorted(out_dir.rglob("*.c")):
            output = obj_dir / f"{source.stem}.o"
            run([compiler, "-std=c11", "-Wall", "-Wextra", "-I", str(out_dir), "-c", str(source), "-o", str(output)])
        return
    if check == "cxx":
        compiler = cxx_compiler()
        source = out_dir / "_verify.cpp"
        source.write_text('#include "sora_config.hpp"\nint main() { return 0; }\n', encoding="utf-8")
        run([compiler, "-std=c++17", "-Wall", "-Wextra", "-I", str(out_dir), "-c", str(source), "-o", str(out_dir / "_verify.o")])
        return
    raise AssertionError(f"unknown check {check}")


def c_compiler() -> str:
    for candidate in ("cc", "gcc", "clang"):
        if shutil.which(candidate):
            return candidate
    return "cc"


def cxx_compiler() -> str:
    for candidate in ("c++", "g++", "clang++"):
        if shutil.which(candidate):
            return candidate
    return "c++"


def runtime_slug(runtime_format: str) -> str:
    return runtime_format.replace("-", "_")


def run(command: list[str]) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


if __name__ == "__main__":
    raise SystemExit(main())
