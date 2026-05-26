#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

rm -rf book

mdbook build .

MDBOOK_BOOK__SRC=src/zh \
MDBOOK_BOOK__LANGUAGE=zh-CN \
MDBOOK_BUILD__BUILD_DIR=book/zh \
MDBOOK_OUTPUT__HTML__SITE_URL=/sora/zh/ \
  mdbook build .
