# Agent Workflow Validator

자동화된 에이전트 워크플로우 검증 도구 — 플레이북의 명령 시퀀스를 자동으로 실행하고 검증합니다.

## 개요

Agent Workflow Validator는 [에이전트 실무 플레이북](../../mydocs/manual/agent_task_playbook.md)의 명령 시퀀스를 YAML/JSON으로 정의하고, 자동으로 실행하여 각 단계의 성공 여부를 검증하는 도구입니다.

## 설치

```bash
pip install -r requirements.txt
```

## 사용법

```bash
python validator.py examples/01-form-autofill.yaml
```

## 주요 기능

1. **워크플로우 정의** - YAML/JSON 형식으로 명령 시퀀스 작성
2. **자동 실행** - 각 단계를 순서대로 실행하고 출력 수집
3. **다양한 검증** - 종료 코드, JSON 출력, 파일 존재, NDJSON 레코드 수 등
4. **상세한 오류 보고** - 실패 시 어느 단계에서 무엇이 잘못되었는지 명확히 표시
5. **CI 통합** - GitHub Actions 등 CI 시스템에서 자동 실행 가능

## 파일 구조

```
workflow-validator/
├── validator.py              # 메인 검증기
├── requirements.txt          # Python 의존성
├── schemas/
│   └── workflow.schema.json # 워크플로우 JSON Schema
├── examples/                 # 플레이북 기반 예제
│   ├── 01-form-autofill.yaml
│   ├── 02-table-export.yaml
│   ├── 03-search-and-render.yaml
│   ├── 04-batch-sweep.yaml
│   └── 05-format-conversion-verify.yaml
└── tests/                    # 테스트 워크플로우
    └── test-simple.yaml
```

## 워크플로우 예제

### 기본 구조

```yaml
name: "워크플로우 이름"
version: "1.0.0"
environment:
  dependencies: [rhwp, jq]
  variables:
    SAMPLE_DIR: "samples"
steps:
  - id: step1
    name: "단계 이름"
    command: 'rhwp info file.hwp --json'
    validations:
      - type: exit_code
        exit_code: 0
      - type: json_valid
```

## 검증 타입

- `exit_code` - 종료 코드 확인
- `json_valid` / `json_path` - JSON 출력 검증
- `ndjson_valid` / `ndjson_count_min/max` - NDJSON 검증
- `file_exists` / `file_size_min/max` - 파일 검증
- `stdout_contains` / `stdout_regex` - 출력 패턴
- `custom_script` - 커스텀 검증

자세한 문서는 validator.py의 docstring을 참조하세요.

## CI 통합

`.github/workflows/validate-agent-workflows.yml` 참조

## 라이선스

MIT License - rhwp 프로젝트와 동일
