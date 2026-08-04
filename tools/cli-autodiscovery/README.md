# rhwp CLI Autodiscovery Tool

rhwp CLI의 모든 명령어, 옵션, 출력 형식을 자동으로 발견하고 문서화하는 도구입니다.

## 기능

### 1. --help 출력 파싱
- rhwp CLI의 `--help` 출력을 구문 분석
- 모든 명령어와 하위 명령어 자동 발견
- 각 명령어의 옵션, 플래그, 인자 추출
- 설명 및 사용법 파싱

### 2. JSON 스키마 자동 추출
- `--json` 플래그를 지원하는 명령어 식별
- 샘플 HWP 파일로 실제 JSON 출력 생성
- 여러 샘플을 분석하여 JSON Schema 자동 추론
- 타입, 필수 필드, 중첩 구조 파악

### 3. Markdown 문서 자동 생성
- 체계적인 CLI 레퍼런스 문서 생성
- 명령어 카테고리별 분류 (내보내기, 정보 조회, 진단/디버그 등)
- 자동 목차 생성
- 옵션, JSON 스키마, 종료 코드 포함

### 4. OpenAPI/JSON Schema 출력
- OpenAPI 3.0 스펙 자동 생성
- JSON 출력 명령어를 REST API 엔드포인트로 모델링
- 재사용 가능한 컴포넌트 스키마 정의
- API 클라이언트 코드 자동 생성 가능

## 설치 및 요구사항

### Python 요구사항
- Python 3.7 이상
- 표준 라이브러리만 사용 (외부 의존성 없음)

### 필요 파일
- rhwp 바이너리 (빌드되어 있거나 cargo로 빌드 가능)
- 샘플 HWP 파일 (JSON 스키마 추출용, 선택사항)

## 사용법

### 기본 사용

```bash
# rhwp 디렉토리에서 실행
cd /path/to/rhwp
python tools/cli-autodiscovery/autodiscover.py --build --output-dir docs/cli
```

### 옵션

```bash
python autodiscover.py [옵션]

옵션:
  --rhwp-binary PATH    rhwp 바이너리 경로 (기본: target/debug/rhwp)
  --output-dir PATH     출력 디렉토리 (기본: output/cli-docs)
  --build               먼저 cargo build로 바이너리 빌드
  -h, --help            도움말 표시
```

### 예제

#### 1. 자동 빌드 후 문서 생성
```bash
python autodiscover.py --build
```

#### 2. 기존 바이너리로 문서 생성
```bash
python autodiscover.py --rhwp-binary ./target/release/rhwp
```

#### 3. 특정 디렉토리에 출력
```bash
python autodiscover.py --build --output-dir ./mydocs/cli-reference
```

## 출력 파일

실행 후 다음 파일들이 생성됩니다:

### 1. `CLI_REFERENCE.md`
전체 CLI 명령어 레퍼런스 문서
- 명령어별 상세 설명
- 옵션 및 플래그 목록
- JSON 스키마 (해당되는 경우)
- 사용 예제

### 2. `commands.json`
모든 명령어 정보를 담은 JSON 파일
```json
{
  "export-svg": {
    "name": "export-svg",
    "description": "HWP/HWPX/HML 문서를 SVG로 내보내기",
    "usage": "<파일.hwp|파일.hwpx|파일.hml> [옵션]",
    "options": [...],
    "supports_json": true,
    "json_schema": {...}
  },
  ...
}
```

### 3. `openapi.json`
OpenAPI 3.0 스펙 파일
- JSON 출력 명령어를 API로 모델링
- 자동 생성된 스키마 정의
- API 클라이언트 생성에 사용 가능

## 아키텍처

### 핵심 클래스

#### `HelpParser`
`--help` 출력을 파싱하여 명령어 구조 추출
- 명령어 섹션 감지
- 옵션 라인 파싱
- 설명 및 사용법 추출

#### `JSONSchemaExtractor`
JSON 출력 샘플로부터 스키마 추론
- 샘플 HWP 파일 자동 검색
- 여러 샘플에서 JSON 출력 수집
- 타입 및 구조 자동 추론
- JSON Schema 생성

#### `MarkdownGenerator`
체계적인 Markdown 문서 생성
- 카테고리별 명령어 분류
- 자동 목차 생성
- 옵션 및 스키마 포맷팅

#### `OpenAPIGenerator`
OpenAPI 3.0 스펙 생성
- JSON 명령어를 엔드포인트로 변환
- 파라미터 및 응답 스키마 정의
- 컴포넌트 재사용

### 데이터 모델

```python
@dataclass
class CommandOption:
    name: str
    short_flag: Optional[str]
    long_flag: Optional[str]
    argument: Optional[str]
    description: str
    is_flag: bool
    choices: List[str]

@dataclass
class Command:
    name: str
    description: str
    usage: str
    options: List[CommandOption]
    supports_json: bool
    json_schema: Optional[Dict[str, Any]]
    exit_codes: Dict[int, str]
    examples: List[str]
```

## 작동 원리

### 1단계: Help 텍스트 파싱
1. `rhwp --help` 실행하여 전체 도움말 획득
2. 정규표현식으로 명령어 섹션 탐지
3. 각 명령어의 옵션 라인 파싱
4. `Command` 및 `CommandOption` 객체 생성

### 2단계: JSON 스키마 추출
1. `--json` 플래그 지원 명령어 필터링
2. 샘플 HWP/HWPX 파일 탐색 (samples/, pdf/, tests/fixtures/)
3. 각 샘플로 명령어 실행하여 JSON 출력 수집
4. 여러 샘플의 구조를 분석하여 스키마 추론
5. 공통 필드를 required로, 타입을 자동 결정

### 3단계: 문서 생성
1. 명령어를 카테고리별로 분류
2. Markdown 목차 자동 생성
3. 각 명령어의 상세 문서 작성
4. JSON 스키마를 코드 블록으로 포함

### 4단계: OpenAPI 생성
1. JSON 출력 명령어를 API 엔드포인트로 변환
2. 옵션을 쿼리 파라미터로 매핑
3. JSON 스키마를 응답 스키마로 사용
4. 재사용 가능한 컴포넌트 정의

## 확장 가능성

### 새로운 명령어 자동 감지
도구는 `--help` 출력을 파싱하므로, rhwp에 새 명령어를 추가하면 자동으로 감지됩니다.

### 커스텀 스키마 힌트
향후 명령어 코드에 스키마 힌트를 주석으로 추가하여 더 정확한 스키마 생성 가능:
```rust
/// JSON Schema: { "type": "object", "properties": { "pageCount": { "type": "integer" } } }
fn export_text(args: &[String]) -> i32 {
    // ...
}
```

### 다른 형식 지원
- Swagger UI HTML 생성
- GraphQL 스키마 변환
- TypeScript 타입 정의 생성

## 제한사항

### 스키마 추론의 한계
- 샘플 파일이 없으면 스키마를 추출할 수 없음
- 선택적 필드는 샘플에 나타나지 않으면 누락될 수 있음
- 복잡한 조건부 스키마는 자동 추론 불가

### 해결책
1. 다양한 샘플 파일 제공
2. 명령어 코드에 명시적 스키마 정의 추가
3. 수동으로 스키마를 보완

## 트러블슈팅

### "rhwp 바이너리를 찾을 수 없습니다"
```bash
# 먼저 빌드
cargo build --bin rhwp

# 또는 --build 플래그 사용
python autodiscover.py --build
```

### "샘플 파일을 찾을 수 없습니다"
```bash
# samples, pdf, tests/fixtures 디렉토리에 HWP 파일 추가
# 또는 샘플 없이 실행 (스키마 추출은 건너뜀)
```

### JSON 스키마가 생성되지 않음
- 명령어가 `--json` 플래그를 지원하는지 확인
- 샘플 파일이 올바른 형식인지 확인
- 명령어 실행 시 타임아웃 발생 여부 확인

## 활용 사례

### 1. API 서버 개발
OpenAPI 스펙을 사용하여 rhwp를 래핑하는 REST API 서버 구축

### 2. 클라이언트 라이브러리 생성
OpenAPI 스펙에서 Python, TypeScript, Go 등의 클라이언트 코드 자동 생성

### 3. 문서 유지보수 자동화
CI/CD 파이프라인에 통합하여 코드 변경 시 문서 자동 업데이트

### 4. MCP 도구 정의 검증
생성된 스키마를 MCP (Model Context Protocol) 도구 정의와 비교

## 관련 문서

- [rhwp CLI 사용 가이드](../../mydocs/manual/cli_commands.md)
- [MCP 서버 구현](../mcp-server/README.md)
- [개발 환경 가이드](../../mydocs/manual/dev_environment_guide.md)

## 라이선스

이 도구는 rhwp 프로젝트의 일부로 MIT 라이선스 하에 배포됩니다.

## 기여

버그 리포트, 기능 제안, 풀 리퀘스트를 환영합니다!

### 개선 아이디어
- [ ] 명령어 실행 예제 자동 생성
- [ ] 종료 코드 자동 감지 및 문서화
- [ ] 대화형 모드로 누락된 정보 입력
- [ ] GraphQL 스키마 생성
- [ ] TypeScript 타입 정의 생성
- [ ] Swagger UI HTML 생성
