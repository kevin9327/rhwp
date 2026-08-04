#!/usr/bin/env python3
"""
JSON Schema Validator

autodiscover.py로 생성된 JSON 스키마를 실제 명령어 출력과 비교하여 검증합니다.

사용법:
    python schema_validator.py --commands-json commands.json --rhwp-binary rhwp
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any, List, Tuple


class SchemaValidator:
    """JSON 스키마 검증기"""

    def __init__(self, commands_json: Path, rhwp_binary: Path):
        self.commands_json = commands_json
        self.rhwp_binary = rhwp_binary
        self.commands = self._load_commands()

    def _load_commands(self) -> Dict[str, Any]:
        """commands.json 로드"""
        with open(self.commands_json, 'r', encoding='utf-8') as f:
            return json.load(f)

    def validate_all(self, sample_files: List[Path]) -> Dict[str, List[str]]:
        """모든 명령어 스키마 검증"""
        results = {}

        for cmd_name, cmd_data in self.commands.items():
            if not cmd_data.get('supports_json'):
                continue

            schema = cmd_data.get('json_schema')
            if not schema:
                continue

            print(f"\n검증 중: {cmd_name}")
            errors = self._validate_command(cmd_name, schema, sample_files)

            if errors:
                results[cmd_name] = errors
                print(f"  ✗ {len(errors)}개 오류 발견")
                for error in errors:
                    print(f"    - {error}")
            else:
                print(f"  ✓ 통과")

        return results

    def _validate_command(self, cmd_name: str, schema: Dict[str, Any],
                         sample_files: List[Path]) -> List[str]:
        """특정 명령어 스키마 검증"""
        errors = []

        for sample_file in sample_files[:3]:  # 최대 3개 샘플 테스트
            try:
                result = subprocess.run(
                    [str(self.rhwp_binary), cmd_name, str(sample_file), '--json'],
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode != 0:
                    continue

                try:
                    output = json.loads(result.stdout)
                except json.JSONDecodeError as e:
                    errors.append(f"JSON 파싱 실패 ({sample_file.name}): {e}")
                    continue

                # 스키마 검증
                schema_errors = self._validate_against_schema(output, schema, "")
                errors.extend([f"{sample_file.name}: {e}" for e in schema_errors])

            except subprocess.TimeoutExpired:
                errors.append(f"타임아웃 ({sample_file.name})")
            except subprocess.SubprocessError as e:
                errors.append(f"실행 실패 ({sample_file.name}): {e}")

        return errors

    def _validate_against_schema(self, data: Any, schema: Dict[str, Any],
                                path: str) -> List[str]:
        """데이터가 스키마를 만족하는지 검증"""
        errors = []
        schema_type = schema.get('type')

        # 타입 검증
        if schema_type:
            if not self._check_type(data, schema_type):
                errors.append(f"{path}: 타입 불일치 (기대: {schema_type}, 실제: {type(data).__name__})")
                return errors

        # 객체 타입 검증
        if schema_type == 'object':
            properties = schema.get('properties', {})
            required = schema.get('required', [])

            # 필수 필드 검증
            for field in required:
                if field not in data:
                    errors.append(f"{path}: 필수 필드 누락 - {field}")

            # 각 프로퍼티 검증
            for key, value in data.items():
                if key in properties:
                    field_path = f"{path}.{key}" if path else key
                    field_schema = properties[key]
                    errors.extend(self._validate_against_schema(value, field_schema, field_path))

        # 배열 타입 검증
        elif schema_type == 'array':
            items_schema = schema.get('items')
            if items_schema and isinstance(data, list):
                for i, item in enumerate(data):
                    item_path = f"{path}[{i}]"
                    errors.extend(self._validate_against_schema(item, items_schema, item_path))

        return errors

    def _check_type(self, data: Any, schema_type: str) -> bool:
        """데이터 타입 검사"""
        type_map = {
            'string': str,
            'integer': int,
            'number': (int, float),
            'boolean': bool,
            'array': list,
            'object': dict,
            'null': type(None)
        }

        expected_type = type_map.get(schema_type)
        if expected_type is None:
            return True

        return isinstance(data, expected_type)


def find_sample_files() -> List[Path]:
    """샘플 HWP 파일 찾기"""
    sample_dirs = [
        Path('samples'),
        Path('pdf'),
        Path('tests/fixtures'),
        Path('mydocs')
    ]

    samples = []
    for sample_dir in sample_dirs:
        if sample_dir.exists():
            samples.extend(sample_dir.glob('**/*.hwp'))
            samples.extend(sample_dir.glob('**/*.hwpx'))
            if len(samples) >= 10:
                break

    return samples[:10]


def main():
    parser = argparse.ArgumentParser(
        description='JSON Schema 검증 도구'
    )
    parser.add_argument(
        '--commands-json',
        type=Path,
        required=True,
        help='commands.json 파일 경로'
    )
    parser.add_argument(
        '--rhwp-binary',
        type=Path,
        required=True,
        help='rhwp 바이너리 경로'
    )
    parser.add_argument(
        '--sample-files',
        type=Path,
        nargs='+',
        help='테스트할 샘플 파일들'
    )

    args = parser.parse_args()

    if not args.commands_json.exists():
        print(f"오류: commands.json을 찾을 수 없습니다: {args.commands_json}", file=sys.stderr)
        sys.exit(1)

    if not args.rhwp_binary.exists():
        print(f"오류: rhwp 바이너리를 찾을 수 없습니다: {args.rhwp_binary}", file=sys.stderr)
        sys.exit(1)

    sample_files = args.sample_files if args.sample_files else find_sample_files()
    if not sample_files:
        print("경고: 샘플 파일을 찾을 수 없습니다. 검증을 건너뜁니다.", file=sys.stderr)
        sys.exit(0)

    print(f"샘플 파일 {len(sample_files)}개로 스키마 검증 시작...")

    validator = SchemaValidator(args.commands_json, args.rhwp_binary)
    results = validator.validate_all(sample_files)

    if results:
        print(f"\n검증 실패: {len(results)}개 명령어에서 오류 발견")
        sys.exit(1)
    else:
        print("\n✓ 모든 스키마 검증 통과")
        sys.exit(0)


if __name__ == '__main__':
    main()
