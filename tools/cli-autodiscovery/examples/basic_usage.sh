#!/bin/bash
# CLI Autodiscovery 기본 사용 예제

# 예제 1: 자동 빌드 후 문서 생성
echo "=== 예제 1: 자동 빌드 후 문서 생성 ==="
python3 autodiscover.py --build --output-dir ./output/example1
echo ""

# 예제 2: 기존 바이너리로 문서 생성
echo "=== 예제 2: 기존 바이너리 사용 ==="
python3 autodiscover.py \
    --rhwp-binary ../../target/debug/rhwp \
    --output-dir ./output/example2
echo ""

# 예제 3: 프로젝트 문서 디렉토리에 직접 생성
echo "=== 예제 3: 프로젝트 문서 디렉토리에 생성 ==="
python3 autodiscover.py \
    --build \
    --output-dir ../../docs/cli-reference
echo ""

# 생성된 파일 확인
echo "=== 생성된 파일 확인 ==="
find ./output -name "*.md" -o -name "*.json" | sort
echo ""

# JSON 명령어 개수 세기
echo "=== 통계 ==="
total_commands=$(jq 'length' ./output/example1/commands.json 2>/dev/null || echo "N/A")
json_commands=$(jq '[.[] | select(.supports_json == true)] | length' ./output/example1/commands.json 2>/dev/null || echo "N/A")

echo "총 명령어: $total_commands"
echo "JSON 지원: $json_commands"
