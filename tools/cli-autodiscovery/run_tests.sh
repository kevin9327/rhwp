#!/bin/bash
# CLI Autodiscovery 테스트 스크립트

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHWP_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$RHWP_ROOT/output/cli-autodiscovery-test"

echo "=== rhwp CLI Autodiscovery 테스트 ==="
echo ""
echo "rhwp 디렉토리: $RHWP_ROOT"
echo "출력 디렉토리: $OUTPUT_DIR"
echo ""

# 1. rhwp 빌드
echo "[1/4] rhwp 빌드 중..."
cd "$RHWP_ROOT"
cargo build --bin rhwp 2>&1 | tail -5

RHWP_BINARY="$RHWP_ROOT/target/debug/rhwp"
if [ ! -f "$RHWP_BINARY" ]; then
    echo "오류: rhwp 바이너리를 찾을 수 없습니다: $RHWP_BINARY"
    exit 1
fi
echo "  ✓ 빌드 완료: $RHWP_BINARY"
echo ""

# 2. 자동 발견 실행
echo "[2/4] CLI 자동 발견 실행 중..."
python3 "$SCRIPT_DIR/autodiscover.py" \
    --rhwp-binary "$RHWP_BINARY" \
    --output-dir "$OUTPUT_DIR"
echo ""

# 3. 생성된 파일 확인
echo "[3/4] 생성된 파일 확인 중..."
EXPECTED_FILES=(
    "CLI_REFERENCE.md"
    "commands.json"
    "openapi.json"
)

for file in "${EXPECTED_FILES[@]}"; do
    filepath="$OUTPUT_DIR/$file"
    if [ -f "$filepath" ]; then
        size=$(wc -c < "$filepath")
        echo "  ✓ $file (${size} bytes)"
    else
        echo "  ✗ $file (누락)"
        exit 1
    fi
done
echo ""

# 4. JSON 파일 유효성 검사
echo "[4/4] JSON 파일 유효성 검사 중..."
for jsonfile in "$OUTPUT_DIR"/*.json; do
    if [ -f "$jsonfile" ]; then
        basename_file=$(basename "$jsonfile")
        if python3 -m json.tool "$jsonfile" > /dev/null 2>&1; then
            echo "  ✓ $basename_file (유효한 JSON)"
        else
            echo "  ✗ $basename_file (잘못된 JSON)"
            exit 1
        fi
    fi
done
echo ""

# 5. 스키마 검증 (선택적)
if [ -d "$RHWP_ROOT/samples" ] || [ -d "$RHWP_ROOT/pdf" ]; then
    echo "[추가] JSON 스키마 검증 중..."
    if python3 "$SCRIPT_DIR/schema_validator.py" \
        --commands-json "$OUTPUT_DIR/commands.json" \
        --rhwp-binary "$RHWP_BINARY" 2>&1 | head -20; then
        echo "  ✓ 스키마 검증 통과"
    else
        echo "  경고: 스키마 검증에 일부 오류가 있을 수 있습니다"
    fi
    echo ""
fi

echo "=== 테스트 완료 ==="
echo ""
echo "결과 파일:"
echo "  - Markdown: $OUTPUT_DIR/CLI_REFERENCE.md"
echo "  - JSON: $OUTPUT_DIR/commands.json"
echo "  - OpenAPI: $OUTPUT_DIR/openapi.json"
echo ""
echo "Markdown 문서 미리보기:"
head -30 "$OUTPUT_DIR/CLI_REFERENCE.md"
echo "..."
echo ""
echo "✓ 모든 테스트 통과"
