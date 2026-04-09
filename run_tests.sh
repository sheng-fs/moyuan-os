#!/bin/bash

# 自动化测试脚本
# 运行所有测试用例并生成测试报告
# 如果所有测试通过，则清理测试文件

set -e

# 获取项目根目录
PROJECT_ROOT=$(pwd)

echo "=== 开始运行测试 ==="

# 创建测试报告目录
TEST_REPORT_DIR="$PROJECT_ROOT/.test_reports"
mkdir -p "$TEST_REPORT_DIR"

# 测试结果文件
TEST_RESULT="$TEST_REPORT_DIR/test_results.txt"
TEST_SUMMARY="$TEST_REPORT_DIR/test_summary.txt"

# 初始化测试结果
echo "测试执行时间: $(date)" > "$TEST_RESULT"
echo "====================================" >> "$TEST_RESULT"

# 运行核心微内核测试
echo "\n=== 运行核心微内核测试 ==="
echo "\n=== 核心微内核测试 ===" >> "$TEST_RESULT"
(cd "$PROJECT_ROOT/kernel/core_microkernel" && cargo test --lib) >> "$TEST_RESULT" 2>&1

# 运行音频服务测试
echo "\n=== 运行音频服务测试 ==="
echo "\n=== 音频服务测试 ===" >> "$TEST_RESULT"
(cd "$PROJECT_ROOT/services/audio" && cargo test --lib) >> "$TEST_RESULT" 2>&1

# 运行输入法服务测试
echo "\n=== 运行输入法服务测试 ==="
echo "\n=== 输入法服务测试 ===" >> "$TEST_RESULT"
(cd "$PROJECT_ROOT/services/input_method" && cargo test --lib) >> "$TEST_RESULT" 2>&1

# 运行浏览器测试
echo "\n=== 运行浏览器测试 ==="
echo "\n=== 浏览器测试 ===" >> "$TEST_RESULT"
(cd "$PROJECT_ROOT/userland/browser" && cargo test --lib) >> "$TEST_RESULT" 2>&1

# 生成测试摘要
echo "\n=== 测试摘要 ===" > "$TEST_SUMMARY"
echo "测试执行时间: $(date)" >> "$TEST_SUMMARY"
echo "====================================" >> "$TEST_SUMMARY"

# 统计测试结果
PASSED=$(grep -o "test result: ok" "$TEST_RESULT" | wc -l)
FAILED=$(grep -o "test result: FAILED" "$TEST_RESULT" | wc -l)
TOTAL=$((PASSED + FAILED))

echo "总测试套件数: $TOTAL" >> "$TEST_SUMMARY"
echo "通过测试套件数: $PASSED" >> "$TEST_SUMMARY"
echo "失败测试套件数: $FAILED" >> "$TEST_SUMMARY"

if [ $FAILED -eq 0 ]; then
    echo "\n🎉 所有测试通过！" >> "$TEST_SUMMARY"
    echo "\n🎉 所有测试通过！正在清理测试文件..."

    # 清理测试文件
    find "$PROJECT_ROOT" -name "tests" -type d -exec rm -rf {} \;
    rm -rf "$TEST_REPORT_DIR"

    echo "测试文件已清理完成。"
else
    echo "\n❌ 存在测试失败，请查看测试报告了解详情。" >> "$TEST_SUMMARY"
    echo "\n❌ 存在测试失败，请查看测试报告了解详情。"
    echo "测试报告位置: $TEST_REPORT_DIR"
fi

echo "\n=== 测试完成 ==="
echo "测试摘要已保存到: $TEST_SUMMARY"
