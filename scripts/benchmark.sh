#!/bin/bash
set -e

CLOV="$(cd "$(dirname ./target/release/clov)" && pwd)/$(basename ./target/release/clov)"
BENCH_DIR="./scripts/benchmark"

# Mode local : générer les fichiers debug
if [ -z "$CI" ]; then
  rm -rf "$BENCH_DIR"
  mkdir -p "$BENCH_DIR/unix" "$BENCH_DIR/clov" "$BENCH_DIR/diff"
fi

# Nom de fichier safe
safe_name() {
  echo "$1" | tr ' /' '_-' | tr -cd 'a-zA-Z0-9_-'
}

# Fonction pour compter les tokens (~4 chars = 1 token)
count_tokens() {
  local input="$1"
  local len=${#input}
  echo $(( (len + 3) / 4 ))
}

# Compteurs globaux
TOTAL_UNIX=0
TOTAL_CLOV=0
TOTAL_TESTS=0
GOOD_TESTS=0
FAIL_TESTS=0
SKIP_TESTS=0

# Fonction de benchmark — une ligne par test
bench() {
  local name="$1"
  local unix_cmd="$2"
  local clov_cmd="$3"

  unix_out=$(eval "$unix_cmd" 2>/dev/null || true)
  clov_out=$(eval "$clov_cmd" 2>/dev/null || true)

  unix_tokens=$(count_tokens "$unix_out")
  clov_tokens=$(count_tokens "$clov_out")

  TOTAL_TESTS=$((TOTAL_TESTS + 1))

  local icon=""
  local tag=""

  if [ -z "$clov_out" ]; then
    icon="❌"
    tag="FAIL"
    FAIL_TESTS=$((FAIL_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_CLOV=$((TOTAL_CLOV + unix_tokens))
  elif [ "$clov_tokens" -ge "$unix_tokens" ] && [ "$unix_tokens" -gt 0 ]; then
    icon="⚠️"
    tag="SKIP"
    SKIP_TESTS=$((SKIP_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_CLOV=$((TOTAL_CLOV + unix_tokens))
  else
    icon="✅"
    tag="GOOD"
    GOOD_TESTS=$((GOOD_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_CLOV=$((TOTAL_CLOV + clov_tokens))
  fi

  if [ "$tag" = "FAIL" ]; then
    printf "%s %-24s │ %-40s │ %-40s │ %6d → %6s (--)\n" \
      "$icon" "$name" "$unix_cmd" "$clov_cmd" "$unix_tokens" "-"
  else
    if [ "$unix_tokens" -gt 0 ]; then
      local pct=$(( (unix_tokens - clov_tokens) * 100 / unix_tokens ))
    else
      local pct=0
    fi
    printf "%s %-24s │ %-40s │ %-40s │ %6d → %6d (%+d%%)\n" \
      "$icon" "$name" "$unix_cmd" "$clov_cmd" "$unix_tokens" "$clov_tokens" "$pct"
  fi

  # Fichiers debug en local uniquement
  if [ -z "$CI" ]; then
    local filename=$(safe_name "$name")
    local prefix="GOOD"
    [ "$tag" = "FAIL" ] && prefix="FAIL"
    [ "$tag" = "SKIP" ] && prefix="BAD"

    local ts=$(date "+%d/%m/%Y %H:%M:%S")

    printf "# %s\n> %s\n\n\`\`\`bash\n$ %s\n\`\`\`\n\n\`\`\`\n%s\n\`\`\`\n" \
      "$name" "$ts" "$unix_cmd" "$unix_out" > "$BENCH_DIR/unix/${filename}.md"

    printf "# %s\n> %s\n\n\`\`\`bash\n$ %s\n\`\`\`\n\n\`\`\`\n%s\n\`\`\`\n" \
      "$name" "$ts" "$clov_cmd" "$clov_out" > "$BENCH_DIR/clov/${filename}.md"

    {
      echo "# Diff: $name"
      echo "> $ts"
      echo ""
      echo "| Metric | Unix | CLOV |"
      echo "|--------|------|-----|"
      echo "| Tokens | $unix_tokens | $clov_tokens |"
      echo ""
      echo "## Unix"
      echo "\`\`\`"
      echo "$unix_out"
      echo "\`\`\`"
      echo ""
      echo "## CLOV"
      echo "\`\`\`"
      echo "$clov_out"
      echo "\`\`\`"
    } > "$BENCH_DIR/diff/${prefix}-${filename}.md"
  fi
}

# Section header
section() {
  echo ""
  echo "── $1 ──"
}

# ═══════════════════════════════════════════
echo "CLOV Benchmark"
echo "═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════"
printf "   %-24s │ %-40s │ %-40s │ %s\n" "TEST" "SHELL" "CLOV" "TOKENS"
echo "───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────"

# ===================
# ls
# ===================
section "ls"
bench "ls" "ls -la" "$CLOV ls"
bench "ls src/" "ls -la src/" "$CLOV ls src/"
bench "ls -l src/" "ls -l src/" "$CLOV ls -l src/"
bench "ls -la src/" "ls -la src/" "$CLOV ls -la src/"
bench "ls -lh src/" "ls -lh src/" "$CLOV ls -lh src/"
bench "ls src/ -l" "ls -l src/" "$CLOV ls src/ -l"
bench "ls -a" "ls -la" "$CLOV ls -a"
bench "ls multi" "ls -la src/ scripts/" "$CLOV ls src/ scripts/"

# ===================
# read
# ===================
section "read"
bench "read" "cat src/main.rs" "$CLOV read src/main.rs"
bench "read -l minimal" "cat src/main.rs" "$CLOV read src/main.rs -l minimal"
bench "read -l aggressive" "cat src/main.rs" "$CLOV read src/main.rs -l aggressive"
bench "read -n" "cat -n src/main.rs" "$CLOV read src/main.rs -n"

# ===================
# find
# ===================
section "find"
bench "find *" "find . -type f" "$CLOV find '*'"
bench "find *.rs" "find . -name '*.rs' -type f" "$CLOV find '*.rs'"
bench "find --max 10" "find . -not -path './target/*' -not -path './.git/*' -type f | head -10" "$CLOV find '*' --max 10"
bench "find --max 100" "find . -not -path './target/*' -not -path './.git/*' -type f | head -100" "$CLOV find '*' --max 100"

# ===================
# git
# ===================
section "git"
bench "git status" "git status" "$CLOV git status"
bench "git log -n 10" "git log -10" "$CLOV git log -n 10"
bench "git log -n 5" "git log -5" "$CLOV git log -n 5"
bench "git diff" "git diff HEAD~1 2>/dev/null || echo ''" "$CLOV git diff"

# ===================
# grep
# ===================
section "grep"
bench "grep fn" "grep -rn 'fn ' src/ || true" "$CLOV grep 'fn ' src/"
bench "grep struct" "grep -rn 'struct ' src/ || true" "$CLOV grep 'struct ' src/"
bench "grep -l 40" "grep -rn 'fn ' src/ || true" "$CLOV grep 'fn ' src/ -l 40"
bench "grep --max 20" "grep -rn 'fn ' src/ | head -20 || true" "$CLOV grep 'fn ' src/ --max 20"
bench "grep -c" "grep -ron 'fn ' src/ || true" "$CLOV grep 'fn ' src/ -c"

# ===================
# json
# ===================
section "json"
cat > /tmp/clov_bench.json << 'JSONEOF'
{
  "name": "clov",
  "version": "0.2.1",
  "config": {
    "debug": false,
    "max_depth": 10,
    "filters": ["node_modules", "target", ".git"]
  },
  "dependencies": {
    "serde": "1.0",
    "clap": "4.0",
    "anyhow": "1.0"
  }
}
JSONEOF
bench "json" "cat /tmp/clov_bench.json" "$CLOV json /tmp/clov_bench.json"
bench "json -d 2" "cat /tmp/clov_bench.json" "$CLOV json /tmp/clov_bench.json -d 2"
rm -f /tmp/clov_bench.json

# ===================
# deps
# ===================
section "deps"
bench "deps" "cat Cargo.toml" "$CLOV deps"

# ===================
# env
# ===================
section "env"
bench "env" "env" "$CLOV env"
bench "env -f PATH" "env | grep PATH" "$CLOV env -f PATH"
bench "env --show-all" "env" "$CLOV env --show-all"

# ===================
# err
# ===================
section "err"
bench "err cargo build" "cargo build 2>&1 || true" "$CLOV err cargo build"

# ===================
# test
# ===================
section "test"
bench "test cargo test" "cargo test 2>&1 || true" "$CLOV test cargo test"

# ===================
# log
# ===================
section "log"
LOG_FILE="/tmp/clov_bench_sample.log"
cat > "$LOG_FILE" << 'LOGEOF'
2024-01-15 10:00:01 INFO  Application started
2024-01-15 10:00:02 INFO  Loading configuration
2024-01-15 10:00:03 ERROR Connection failed: timeout
2024-01-15 10:00:04 ERROR Connection failed: timeout
2024-01-15 10:00:05 ERROR Connection failed: timeout
2024-01-15 10:00:06 ERROR Connection failed: timeout
2024-01-15 10:00:07 ERROR Connection failed: timeout
2024-01-15 10:00:08 WARN  Retrying connection
2024-01-15 10:00:09 INFO  Connection established
2024-01-15 10:00:10 INFO  Processing request
2024-01-15 10:00:11 INFO  Processing request
2024-01-15 10:00:12 INFO  Processing request
2024-01-15 10:00:13 INFO  Request completed
LOGEOF
bench "log" "cat $LOG_FILE" "$CLOV log $LOG_FILE"
rm -f "$LOG_FILE"

# ===================
# summary
# ===================
section "summary"
bench "summary cargo --help" "cargo --help" "$CLOV summary cargo --help"
bench "summary rustc --help" "rustc --help 2>/dev/null || echo 'rustc not found'" "$CLOV summary rustc --help"

# ===================
# Modern JavaScript Stack (skip si pas de package.json)
# ===================
if [ -f "package.json" ]; then
  section "modern JS stack"

  if command -v tsc &> /dev/null || [ -f "node_modules/.bin/tsc" ]; then
    bench "tsc" "tsc --noEmit 2>&1 || true" "$CLOV tsc --noEmit"
  fi

  if command -v prettier &> /dev/null || [ -f "node_modules/.bin/prettier" ]; then
    bench "prettier --check" "prettier --check . 2>&1 || true" "$CLOV prettier --check ."
  fi

  if command -v eslint &> /dev/null || [ -f "node_modules/.bin/eslint" ]; then
    bench "lint" "eslint . 2>&1 || true" "$CLOV lint ."
  fi

  if [ -f "next.config.js" ] || [ -f "next.config.mjs" ] || [ -f "next.config.ts" ]; then
    if command -v next &> /dev/null || [ -f "node_modules/.bin/next" ]; then
      bench "next build" "next build 2>&1 || true" "$CLOV next build"
    fi
  fi

  if [ -f "playwright.config.ts" ] || [ -f "playwright.config.js" ]; then
    if command -v playwright &> /dev/null || [ -f "node_modules/.bin/playwright" ]; then
      bench "playwright test" "playwright test 2>&1 || true" "$CLOV playwright test"
    fi
  fi

  if [ -f "prisma/schema.prisma" ]; then
    if command -v prisma &> /dev/null || [ -f "node_modules/.bin/prisma" ]; then
      bench "prisma generate" "prisma generate 2>&1 || true" "$CLOV prisma generate"
    fi
  fi

  if command -v vitest &> /dev/null || [ -f "node_modules/.bin/vitest" ]; then
    bench "vitest run" "vitest run --reporter=json 2>&1 || true" "$CLOV vitest run"
  fi

  if command -v pnpm &> /dev/null; then
    bench "pnpm list" "pnpm list --depth 0 2>&1 || true" "$CLOV pnpm list --depth 0"
    bench "pnpm outdated" "pnpm outdated 2>&1 || true" "$CLOV pnpm outdated"
  fi
fi

# ===================
# gh (skip si pas dispo ou pas dans un repo)
# ===================
if command -v gh &> /dev/null && git rev-parse --git-dir &> /dev/null; then
  section "gh"
  bench "gh pr list" "gh pr list 2>&1 || true" "$CLOV gh pr list"
  bench "gh run list" "gh run list 2>&1 || true" "$CLOV gh run list"
fi

# ===================
# docker (skip si pas dispo)
# ===================
if command -v docker &> /dev/null; then
  section "docker"
  bench "docker ps" "docker ps 2>/dev/null || true" "$CLOV docker ps"
  bench "docker images" "docker images 2>/dev/null || true" "$CLOV docker images"
fi

# ===================
# kubectl (skip si pas dispo)
# ===================
if command -v kubectl &> /dev/null; then
  section "kubectl"
  bench "kubectl pods" "kubectl get pods 2>/dev/null || true" "$CLOV kubectl pods"
  bench "kubectl services" "kubectl get services 2>/dev/null || true" "$CLOV kubectl services"
fi

# ===================
# Python (avec fixtures temporaires)
# ===================
if command -v python3 &> /dev/null && command -v ruff &> /dev/null && command -v pytest &> /dev/null; then
  section "python"

  PYTHON_FIXTURE=$(mktemp -d)
  cd "$PYTHON_FIXTURE"

  # pyproject.toml
  cat > pyproject.toml << 'PYEOF'
[project]
name = "clov-bench"
version = "0.25.0"

[tool.ruff]
line-length = 88
PYEOF

  # sample.py avec quelques issues ruff
  cat > sample.py << 'PYEOF'
import os
import sys
import json


def process_data(x):
    if x == None:  # E711: comparison to None
        return []
    result = []
    for i in range(len(x)):  # C416: unnecessary list comprehension
        result.append(x[i] * 2)
    return result

def unused_function():  # F841: local variable assigned but never used
    temp = 42
    return None
PYEOF

  # test_sample.py
  cat > test_sample.py << 'PYEOF'
from sample import process_data

def test_process_data():
    assert process_data([1, 2, 3]) == [2, 4, 6]

def test_process_data_none():
    assert process_data(None) == []
PYEOF

  bench "ruff check" "ruff check . 2>&1 || true" "$CLOV test ruff check ."
  bench "pytest" "pytest -v 2>&1 || true" "$CLOV test pytest -v"

  cd - > /dev/null
  rm -rf "$PYTHON_FIXTURE"
fi

# ===================
# Go (avec fixtures temporaires)
# ===================
if command -v go &> /dev/null && command -v golangci-lint &> /dev/null; then
  section "go"

  GO_FIXTURE=$(mktemp -d)
  cd "$GO_FIXTURE"

  # go.mod
  cat > go.mod << 'GOEOF'
module bench

go 1.21
GOEOF

  # main.go
  cat > main.go << 'GOEOF'
package main

import "fmt"

func Add(a, b int) int {
    return a + b
}

func Multiply(a, b int) int {
    return a * b
}

func main() {
    fmt.Println(Add(2, 3))
    fmt.Println(Multiply(4, 5))
}
GOEOF

  # main_test.go
  cat > main_test.go << 'GOEOF'
package main

import "testing"

func TestAdd(t *testing.T) {
    result := Add(2, 3)
    if result != 5 {
        t.Errorf("Add(2, 3) = %d; want 5", result)
    }
}

func TestMultiply(t *testing.T) {
    result := Multiply(4, 5)
    if result != 20 {
        t.Errorf("Multiply(4, 5) = %d; want 20", result)
    }
}
GOEOF

  bench "golangci-lint" "golangci-lint run 2>&1 || true" "$CLOV test golangci-lint run"
  bench "go test" "go test -v 2>&1 || true" "$CLOV test go test -v"

  cd - > /dev/null
  rm -rf "$GO_FIXTURE"
fi

# ===================
# Résumé global
# ===================
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════"

if [ "$TOTAL_TESTS" -gt 0 ]; then
  GOOD_PCT=$((GOOD_TESTS * 100 / TOTAL_TESTS))
  if [ "$TOTAL_UNIX" -gt 0 ]; then
    TOTAL_SAVED=$((TOTAL_UNIX - TOTAL_CLOV))
    TOTAL_SAVE_PCT=$((TOTAL_SAVED * 100 / TOTAL_UNIX))
  else
    TOTAL_SAVED=0
    TOTAL_SAVE_PCT=0
  fi

  echo ""
  echo "  ✅ $GOOD_TESTS good  ⚠️ $SKIP_TESTS skip  ❌ $FAIL_TESTS fail    $GOOD_TESTS/$TOTAL_TESTS ($GOOD_PCT%)"
  echo "  Tokens: $TOTAL_UNIX → $TOTAL_CLOV  (-$TOTAL_SAVE_PCT%)"
  echo ""

  # Fichiers debug en local
  if [ -z "$CI" ]; then
    echo "  Debug: $BENCH_DIR/{unix,clov,diff}/"
  fi
  echo ""

  # Exit code non-zero si moins de 80% good
  if [ "$GOOD_PCT" -lt 80 ]; then
    echo "  BENCHMARK FAILED: $GOOD_PCT% good (minimum 80%)"
    exit 1
  fi
fi
