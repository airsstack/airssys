#!/bin/bash
# CI/CD Gate: DEBT-WASM-004 Verification
# 
# This script MUST PASS before merging Phase 2+, Block 4, or Block 6 work.
# Ensures all Task 1.3 deferred work is completed.
#
# Usage: ./check-debt-wasm-004.sh [phase2|block4|block6|all]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ACTOR_IMPL="airssys-wasm/src/actor/actor_impl.rs"
DEBT_DOC=".memory-bank/sub-projects/airssys-wasm/docs/technical-debt/debt-wasm-004-task-1.3-deferred-implementation.md"

check_mode="${1:-all}"

echo "=========================================="
echo "DEBT-WASM-004 Verification Gate"
echo "Mode: $check_mode"
echo "=========================================="
echo ""

# Function to check for FUTURE WORK comments
check_future_work() {
    local file=$1
    local item_name=$2
    
    if grep -q "FUTURE WORK" "$file"; then
        echo -e "${RED}❌ FAILED: $item_name has unresolved FUTURE WORK${NC}"
        echo "   Location: $file"
        echo "   Count: $(grep -c "FUTURE WORK" "$file") occurrences"
        echo "   See: $DEBT_DOC"
        return 1
    else
        echo -e "${GREEN}✅ PASSED: $item_name resolved${NC}"
        return 0
    fi
}

# Function to check for specific TODO patterns
check_todo_pattern() {
    local file=$1
    local pattern=$2
    local item_name=$3
    
    if grep -q "$pattern" "$file"; then
        echo -e "${RED}❌ FAILED: $item_name has unresolved TODOs${NC}"
        echo "   Pattern: $pattern"
        echo "   Location: $file"
        return 1
    else
        echo -e "${GREEN}✅ PASSED: $item_name TODOs resolved${NC}"
        return 0
    fi
}

# Function to check test coverage
check_test_coverage() {
    local module=$1
    local min_coverage=$2
    local item_name=$3
    
    echo -e "${YELLOW}⏳ Checking test coverage for $item_name...${NC}"
    
    # Run cargo tarpaulin or cargo-llvm-cov (if available)
    if command -v cargo-llvm-cov &> /dev/null; then
        coverage=$(cargo llvm-cov --lib --package airssys-wasm 2>&1 | grep "$module" | awk '{print $NF}' | tr -d '%')
        if [ -z "$coverage" ]; then
            echo -e "${YELLOW}⚠️  WARNING: Could not determine coverage for $item_name${NC}"
            echo "   Manual verification required"
            return 0
        fi
        
        if (( $(echo "$coverage >= $min_coverage" | bc -l) )); then
            echo -e "${GREEN}✅ PASSED: $item_name coverage ${coverage}% ≥ ${min_coverage}%${NC}"
            return 0
        else
            echo -e "${RED}❌ FAILED: $item_name coverage ${coverage}% < ${min_coverage}%${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  WARNING: cargo-llvm-cov not installed, skipping coverage check${NC}"
        echo "   Install: cargo install cargo-llvm-cov"
        return 0
    fi
}

# Phase 2 Task 2.1 Gate
check_phase2() {
    echo "Checking Phase 2 Task 2.1 Requirements..."
    echo "=========================================="
    
    local failed=0
    
    # Check for any FUTURE WORK comments
    check_future_work "$ACTOR_IMPL" "actor_impl.rs deferred work" || failed=1
    
    # Check for test coverage
    check_test_coverage "actor::actor_impl" "90" "Actor message handling" || failed=1
    
    # Check for performance benchmarks
    if [ ! -f "airssys-wasm/benches/actor_message_throughput.rs" ]; then
        echo -e "${RED}❌ FAILED: Performance benchmarks missing${NC}"
        echo "   Required: benches/actor_message_throughput.rs"
        echo "   Target: >10,000 msg/sec"
        failed=1
    else
        echo -e "${GREEN}✅ PASSED: Performance benchmarks present${NC}"
    fi
    
    if [ $failed -eq 0 ]; then
        echo ""
        echo -e "${GREEN}=========================================="
        echo "✅ Phase 2 Task 2.1 Gate: PASSED"
        echo "==========================================${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}=========================================="
        echo "❌ Phase 2 Task 2.1 Gate: FAILED"
        echo "=========================================="
        echo ""
        echo "Review $DEBT_DOC"
        echo "Complete Items #1 and #2 before merging${NC}"
        return 1
    fi
}

# Block 4 Gate (Security)
check_block4() {
    echo "Checking Block 4 (Security) Requirements..."
    echo "=========================================="
    
    local failed=0
    
    # Check for any FUTURE WORK comments
    check_future_work "$ACTOR_IMPL" "actor_impl.rs deferred work" || failed=1
    
    # Check for security tests
    if ! grep -q "test.*capability.*enforcement" "airssys-wasm/src/actor/actor_impl.rs"; then
        echo -e "${RED}❌ FAILED: Capability enforcement tests missing${NC}"
        echo "   Required: Tests verifying capability checks"
        failed=1
    else
        echo -e "${GREEN}✅ PASSED: Capability enforcement tests present${NC}"
    fi
    
    # Check for test coverage (security-critical: 95%)
    check_test_coverage "actor::actor_impl" "95" "Actor security (capability checks)" || failed=1
    
    # Check for security audit documentation
    if [ ! -f ".memory-bank/sub-projects/airssys-wasm/docs/security-audit-block-4.md" ]; then
        echo -e "${YELLOW}⚠️  WARNING: Security audit documentation not found${NC}"
        echo "   Expected: docs/security-audit-block-4.md"
        echo "   Manual security review REQUIRED"
    else
        echo -e "${GREEN}✅ PASSED: Security audit documented${NC}"
    fi
    
    if [ $failed -eq 0 ]; then
        echo ""
        echo -e "${GREEN}=========================================="
        echo "🔒 Block 4 (Security) Gate: PASSED"
        echo "==========================================${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}=========================================="
        echo "❌ Block 4 (Security) Gate: FAILED"
        echo "=========================================="
        echo ""
        echo "🔒 SECURITY CRITICAL: System vulnerable"
        echo "Review $DEBT_DOC Item #3"
        echo "Complete capability enforcement before merging${NC}"
        return 1
    fi
}

# Block 6 Gate (Storage/Registry)
check_block6() {
    echo "Checking Block 6 (Storage) Requirements..."
    echo "=========================================="
    
    local failed=0
    
    # Check for any FUTURE WORK comments
    check_future_work "$ACTOR_IMPL" "actor_impl.rs deferred work" || failed=1
    
    # Check for registry integration
    if ! grep -q "ctx.registry" "airssys-wasm/src/actor/actor_impl.rs"; then
        echo -e "${RED}❌ FAILED: Registry integration not implemented${NC}"
        echo "   Required: ctx.registry calls in pre_start/post_stop"
        failed=1
    else
        echo -e "${GREEN}✅ PASSED: Registry integration present${NC}"
    fi
    
    # Check for memory leak tests
    if [ ! -f "airssys-wasm/tests/actor_memory_leak_tests.rs" ]; then
        echo -e "${RED}❌ FAILED: Memory leak tests missing${NC}"
        echo "   Required: tests/actor_memory_leak_tests.rs"
        failed=1
    else
        echo -e "${GREEN}✅ PASSED: Memory leak tests present${NC}"
    fi
    
    # Check for test coverage
    check_test_coverage "actor::actor_impl" "90" "Actor lifecycle (registry)" || failed=1
    
    if [ $failed -eq 0 ]; then
        echo ""
        echo -e "${GREEN}=========================================="
        echo "💾 Block 6 (Storage) Gate: PASSED"
        echo "==========================================${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}=========================================="
        echo "❌ Block 6 (Storage) Gate: FAILED"
        echo "=========================================="
        echo ""
        echo "💾 MEMORY LEAK RISK: Registry cleanup missing"
        echo "Review $DEBT_DOC Item #5"
        echo "Complete registry integration before merging${NC}"
        return 1
    fi
}

# Main execution
case "$check_mode" in
    phase2)
        check_phase2
        exit $?
        ;;
    block4)
        check_block4
        exit $?
        ;;
    block6)
        check_block6
        exit $?
        ;;
    all)
        failed=0
        check_phase2 || failed=1
        echo ""
        check_block4 || failed=1
        echo ""
        check_block6 || failed=1
        
        if [ $failed -eq 0 ]; then
            echo ""
            echo -e "${GREEN}=========================================="
            echo "✅ ALL DEBT-WASM-004 GATES: PASSED"
            echo "=========================================="
            echo ""
            echo "All Task 1.3 deferred work completed!"
            echo "Safe to merge${NC}"
            exit 0
        else
            echo ""
            echo -e "${RED}=========================================="
            echo "❌ DEBT-WASM-004 VERIFICATION: FAILED"
            echo "=========================================="
            echo ""
            echo "⚠️  DO NOT MERGE - Deferred work incomplete"
            echo ""
            echo "Review: $DEBT_DOC"
            echo "Complete all required items before merging${NC}"
            exit 1
        fi
        ;;
    *)
        echo "Usage: $0 [phase2|block4|block6|all]"
        echo ""
        echo "Examples:"
        echo "  $0 phase2  # Check Phase 2 Task 2.1 requirements"
        echo "  $0 block4  # Check Block 4 security requirements"
        echo "  $0 block6  # Check Block 6 storage requirements"
        echo "  $0 all     # Check all requirements (default)"
        exit 1
        ;;
esac
