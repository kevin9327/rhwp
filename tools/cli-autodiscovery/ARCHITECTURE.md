# CLI Autodiscovery Tool Architecture

## 개요

이 도구는 rhwp CLI의 모든 명령어와 옵션을 자동으로 발견하고 문서화하는 Python 기반 도구입니다.

## 설계 원칙

### 1. Zero External Dependencies
- Python 표준 라이브러리만 사용
- 배포 및 실행이 간단
- 의존성 충돌 없음

### 2. Automated Discovery
- `--help` 출력 파싱으로 명령어 자동 감지
- 실제 JSON 출력 분석으로 스키마 자동 추론
- 새 명령어 추가 시 자동 반영

### 3. Multiple Output Formats
- Markdown (인간 친화적)
- JSON (기계 가독)
- OpenAPI (API 통합)

## 아키텍처 다이어그램

```
┌─────────────────────────────────────────────────────────┐
│                   CLIAutodiscovery                      │
│                    (Main Controller)                     │
└──────────────┬──────────────────────────────────────────┘
               │
               ├─► HelpParser
               │   ├─ parse() → Dict[str, Command]
               │   └─ _parse_command_options()
               │
               ├─► JSONSchemaExtractor
               │   ├─ extract_schema() → Dict[str, Any]
               │   ├─ _infer_schema()
               │   ├─ _infer_field_schema()
               │   └─ _find_sample_files()
               │
               ├─► MarkdownGenerator
               │   ├─ generate() → str
               │   ├─ _generate_header()
               │   ├─ _generate_toc()
               │   └─ _generate_commands_section()
               │
               └─► OpenAPIGenerator
                   ├─ generate() → Dict[str, Any]
                   └─ _generate_path_item()
```

## 핵심 컴포넌트

### 1. HelpParser

**책임:** `--help` 출력을 구조화된 데이터로 변환

**입력:**
```
명령:
  export-svg <파일.hwp> [옵션]
      HWP 문서를 SVG로 내보내기

      -o, --output <폴더>     출력 폴더
      --json                  JSON 출력
```

**출력:**
```python
Command(
    name='export-svg',
    description='HWP 문서를 SVG로 내보내기',
    options=[
        CommandOption(name='output', short_flag='-o', ...),
        CommandOption(name='json', long_flag='--json', ...)
    ],
    supports_json=True
)
```

**파싱 전략:**
1. 정규표현식으로 명령어 라인 감지
2. 들여쓰기로 계층 구조 파악
3. 옵션 패턴 매칭 (short/long flag, argument)
4. 여러 줄 설명 병합

### 2. JSONSchemaExtractor

**책임:** 실제 JSON 출력으로부터 스키마 추론

**작동 방식:**
1. 샘플 HWP 파일 탐색 (samples/, pdf/, tests/)
2. 각 샘플로 명령어 실행: `rhwp <command> <file> --json`
3. JSON 출력 수집 (최대 3개 샘플)
4. 여러 샘플 분석하여 공통 구조 추론

**스키마 추론 알고리즘:**
```python
def infer_schema(samples):
    # 1. 공통 필드 찾기
    common_fields = set(samples[0].keys())
    for sample in samples[1:]:
        common_fields &= set(sample.keys())

    # 2. 타입 추론
    for field in all_fields:
        values = [s[field] for s in samples if field in s]
        infer_type(values)  # str, int, array, object 등

    # 3. 재귀적으로 중첩 객체/배열 처리
    for nested_object:
        infer_schema(nested_samples)
```

**타입 매핑:**
- `str` → `{"type": "string"}`
- `int` → `{"type": "integer"}`
- `float` → `{"type": "number"}`
- `bool` → `{"type": "boolean"}`
- `list` → `{"type": "array", "items": {...}}`
- `dict` → `{"type": "object", "properties": {...}}`

### 3. MarkdownGenerator

**책임:** 구조화된 명령어 데이터를 Markdown 문서로 변환

**문서 구조:**
```markdown
# rhwp CLI Reference

## 목차
### 내보내기
- export-svg
- export-pdf
...

## 개요
- 총 명령어: N개
- JSON 지원: M개

## 명령어
### export-svg
설명...
**사용법:** ...
**옵션:** ...
**JSON 스키마:** ...
```

**카테고리 분류 규칙:**
- `export-*` → 내보내기
- `info`, `capabilities`, `fields`, `search` → 정보 조회
- `hwp5-*`, `dump*`, `diag` → 진단/디버그
- `convert`, `batch`, `edit` → 변환
- 나머지 → 기타

### 4. OpenAPIGenerator

**책임:** JSON 명령어를 OpenAPI 3.0 스펙으로 변환

**변환 전략:**
- 명령어 → API 엔드포인트 (`/commands/{command-name}`)
- 옵션 → 쿼리 파라미터
- JSON 스키마 → 응답 스키마
- 공통 스키마 → 컴포넌트로 추출

**예제 변환:**
```
rhwp export-text file.hwp --json --page 0
```
↓
```yaml
GET /commands/export-text?page=0
responses:
  200:
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/export_text_response'
```

## 데이터 흐름

```
1. Input Phase
   rhwp --help → HelpParser → Dict[str, Command]

2. Schema Extraction Phase
   For each JSON command:
     samples/*.hwp → rhwp cmd --json → JSON output
     → JSONSchemaExtractor → JSON Schema

3. Documentation Phase
   Commands + Schemas → MarkdownGenerator → CLI_REFERENCE.md
                      → JSON serializer → commands.json
                      → OpenAPIGenerator → openapi.json
```

## 에러 처리

### HelpParser
- 알 수 없는 형식: 건너뛰고 계속 진행
- 빈 설명: 빈 문자열로 처리

### JSONSchemaExtractor
- 샘플 파일 없음: 스키마 추출 건너뜀 (경고 로그)
- 명령어 실행 실패: 다음 샘플로 진행
- JSON 파싱 실패: 해당 샘플 무시
- 타임아웃: 30초 후 중단

### 문서 생성
- 스키마 없음: "스키마 정보 없음" 표시
- 빈 명령어: 빈 문서 생성하지 않음

## 확장성

### 새 출력 형식 추가

1. Generator 클래스 작성:
```python
class GraphQLGenerator:
    def __init__(self, commands: Dict[str, Command]):
        self.commands = commands

    def generate(self) -> str:
        # GraphQL 스키마 생성
        pass
```

2. CLIAutodiscovery에 통합:
```python
def _generate_graphql(self):
    gen = GraphQLGenerator(self.commands)
    schema = gen.generate()
    (self.output_dir / 'schema.graphql').write_text(schema)
```

### 커스텀 스키마 힌트 지원

명령어 코드에 메타데이터 주석 추가:
```rust
/// @cli-schema: {"type": "object", "properties": {"pageCount": {"type": "integer"}}}
fn export_text(args: &[String]) -> i32 {
    // ...
}
```

파서 확장:
```python
def extract_schema_hints(source_file: Path) -> Dict[str, Any]:
    # Rust 소스 파싱하여 @cli-schema 주석 추출
    pass
```

## 성능 최적화

### 병렬 스키마 추출
```python
from concurrent.futures import ThreadPoolExecutor

with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [
        executor.submit(extract_schema, cmd)
        for cmd in json_commands
    ]
    results = [f.result() for f in futures]
```

### 캐싱
```python
import pickle
from pathlib import Path

CACHE_FILE = Path('.cli-discovery-cache.pkl')

def load_cache():
    if CACHE_FILE.exists():
        return pickle.load(CACHE_FILE.open('rb'))
    return {}

def save_cache(data):
    pickle.dump(data, CACHE_FILE.open('wb'))
```

## 테스트 전략

### 단위 테스트
```python
def test_help_parser():
    help_text = """
    명령:
      export-svg <파일.hwp>
          SVG 내보내기
          --json    JSON 출력
    """
    parser = HelpParser(help_text)
    commands = parser.parse()
    assert 'export-svg' in commands
    assert commands['export-svg'].supports_json
```

### 통합 테스트
```bash
# run_tests.sh
1. cargo build --bin rhwp
2. python autodiscover.py --rhwp-binary target/debug/rhwp
3. 출력 파일 존재 확인
4. JSON 유효성 검사
5. 스키마 검증
```

### 회귀 테스트
- 기존 생성 문서와 신규 생성 문서 비교
- 명령어 개수 변화 감지
- 스키마 구조 변경 감지

## 보안 고려사항

### 명령어 실행
- 타임아웃 설정 (30초)
- subprocess 격리
- 출력 크기 제한

### 파일 시스템
- 출력 디렉토리 검증
- 경로 순회 공격 방지
- 권한 확인

## 유지보수

### 코드 스타일
- PEP 8 준수
- 타입 힌트 사용 (`typing` 모듈)
- Docstring 작성 (Google 스타일)

### 버전 관리
- 도구 버전: `__version__ = "1.0.0"`
- rhwp 버전과 별도 관리
- CHANGELOG.md 유지

## 참고 자료

- [JSON Schema Specification](https://json-schema.org/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Python argparse](https://docs.python.org/3/library/argparse.html)
- [Markdown Specification](https://spec.commonmark.org/)
