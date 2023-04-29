#!/bin/bash
# CSTNSystems/minotaur smoke test
# Verifies basic solver operation and output generation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$ROOT_DIR/src/rust"
RESULTS_DIR="$ROOT_DIR/results"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo " CSTNSystems/minotaur smoke test"
echo "=========================================="
echo ""

# Check prerequisites
echo -n "[1/6] Checking prerequisites... "
command -v fpm >/dev/null 2>&1 || { echo -e "${RED}FAIL${NC} - fpm not found"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}FAIL${NC} - cargo not found"; exit 1; }
command -v gfortran >/dev/null 2>&1 || { echo -e "${RED}FAIL${NC} - gfortran not found"; exit 1; }
echo -e "${GREEN}OK${NC}"

# Build
echo -n "[2/6] Building project... "
cd "$ROOT_DIR"
make build > /tmp/minotaur_build.log 2>&1 || {
    echo -e "${RED}FAIL${NC}"
    echo "Build log:"
    cat /tmp/minotaur_build.log
    exit 1
}
echo -e "${GREEN}OK${NC}"

# Run baseline
echo -n "[3/6] Running baseline config... "
BASELINE_OUT="$RESULTS_DIR/smoke_baseline.csv"
cd "$RUST_DIR"
cargo run --release -- \
    --config ../../configs/baseline.toml \
    --out "$BASELINE_OUT" \
    --mode single > /tmp/minotaur_baseline.log 2>&1 || {
    echo -e "${RED}FAIL${NC}"
    echo "Run log:"
    cat /tmp/minotaur_baseline.log
    exit 1
}
echo -e "${GREEN}OK${NC}"

# Verify baseline output
echo -n "[4/6] Verifying baseline output... "
if [ ! -f "$BASELINE_OUT" ]; then
    echo -e "${RED}FAIL${NC} - output file not created"
    exit 1
fi
LINE_COUNT=$(wc -l < "$BASELINE_OUT")
if [ "$LINE_COUNT" -lt 2 ]; then
    echo -e "${RED}FAIL${NC} - output file too short ($LINE_COUNT lines)"
    exit 1
fi
# Check for convergence (status=0)
if ! grep -q ",0," "$BASELINE_OUT"; then
    echo -e "${YELLOW}WARN${NC} - baseline did not converge"
else
    echo -e "${GREEN}OK${NC}"
fi

# Run mini sweep
echo -n "[5/6] Running mini sweep... "
SWEEP_OUT="$RESULTS_DIR/smoke_sweep.csv"
cd "$RUST_DIR"
cargo run --release -- \
    --config ../../configs/sweep.toml \
    --out "$SWEEP_OUT" \
    --mode sweep > /tmp/minotaur_sweep.log 2>&1 || {
    echo -e "${RED}FAIL${NC}"
    echo "Run log:"
    cat /tmp/minotaur_sweep.log
    exit 1
}
echo -e "${GREEN}OK${NC}"

# Verify sweep output
echo -n "[6/6] Verifying sweep output... "
if [ ! -f "$SWEEP_OUT" ]; then
    echo -e "${RED}FAIL${NC} - sweep output file not created"
    exit 1
fi
SWEEP_LINES=$(wc -l < "$SWEEP_OUT")
# sweep.toml has 21x21 = 441 points + 1 header = 442 lines
if [ "$SWEEP_LINES" -lt 100 ]; then
    echo -e "${RED}FAIL${NC} - sweep output too short ($SWEEP_LINES lines)"
    exit 1
fi
echo -e "${GREEN}OK${NC}"

# Cleanup
rm -f "$BASELINE_OUT" "$SWEEP_OUT"
rm -f /tmp/minotaur_*.log

echo ""
echo "=========================================="
echo -e " ${GREEN}All smoke tests passed${NC}"
echo "=========================================="
