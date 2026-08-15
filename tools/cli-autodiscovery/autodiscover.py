#!/usr/bin/env python3
"""
rhwp CLI Autodiscovery Tool

이 도구는 rhwp CLI의 모든 명령어, 옵션, 출력 형식을 자동으로 발견하고 문서화합니다.

주요 기능:
1. --help 출력 파싱하여 모든 명령어와 옵션 추출
2. --json 출력 샘플을 분석하여 JSON 스키마 추출
3. Markdown 문서 자동 생성
4. OpenAPI/JSON Schema 형식 출력

사용법:
    python autodiscover.py --rhwp-binary <path> --output-dir <dir>
"""

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Dict, List, Optional, Any
from collections import defaultdict


@dataclass
class CommandOption:
    """명령어 옵션 정보"""
    name: str
    short_flag: Optional[str] = None
    long_flag: Optional[str] = None
    argument: Optional[str] = None
    description: str = ""
    is_flag: bool = True  # 값이 없는 플래그인지 여부
    choices: List[str] = field(default_factory=list)  # enum 선택지


@dataclass
class Command:
    """CLI 명령어 정보"""
    name: str
    description: str = ""
    usage: str = ""
    options: List[CommandOption] = field(default_factory=list)
    supports_json: bool = False
    json_schema: Optional[Dict[str, Any]] = None
    exit_codes: Dict[int, str] = field(default_factory=dict)
    examples: List[str] = field(default_factory=list)


class HelpParser:
    """--help 출력 파서"""

    def __init__(self, help_text: str):
        self.help_text = help_text
        self.commands: Dict[str, Command] = {}

    def parse(self) -> Dict[str, Command]:
        """help 텍스트를 파싱하여 모든 명령어 정보 추출"""
        lines = self.help_text.split('\n')

        current_command = None
        in_command_section = False
        buffer = []

        for i, line in enumerate(lines):
            # 명령 섹션 시작 감지
            if line.strip() == "명령:":
                in_command_section = True
                continue

            if in_command_section:
                # 새 명령어 시작 (들여쓰기 2칸으로 시작)
                if line.startswith('  ') and not line.startswith('    '):
                    # 이전 명령어 처리
                    if current_command and buffer:
                        self._parse_command_options(current_command, buffer)
                        buffer = []

                    # 새 명령어 파싱
                    cmd_match = re.match(r'\s+(\S+)\s+(.*)', line)
                    if cmd_match:
                        cmd_name = cmd_match.group(1)
                        cmd_usage = cmd_match.group(2).strip()

                        # 다음 줄에서 설명 가져오기
                        description = ""
                        if i + 1 < len(lines):
                            desc_line = lines[i + 1].strip()
                            if desc_line and not desc_line.startswith('-'):
                                description = desc_line

                        current_command = Command(
                            name=cmd_name,
                            usage=cmd_usage,
                            description=description
                        )
                        self.commands[cmd_name] = current_command

                # 옵션 라인 수집 (들여쓰기 4칸 이상)
                elif line.startswith('    ') and current_command:
                    buffer.append(line)

                # 빈 줄로 명령어 구분
                elif not line.strip() and current_command and buffer:
                    self._parse_command_options(current_command, buffer)
                    buffer = []

        # 마지막 명령어 처리
        if current_command and buffer:
            self._parse_command_options(current_command, buffer)

        return self.commands

    def _parse_command_options(self, command: Command, option_lines: List[str]):
        """옵션 라인들을 파싱하여 CommandOption 객체로 변환"""
        i = 0
        while i < len(option_lines):
            line = option_lines[i]

            # 옵션 라인 패턴 매칭
            # 예: "      -o, --output <폴더>     출력 폴더 (기본: output/)"
            # 예: "      --json                  산출물 매니페스트를 JSON으로 stdout에 출력"
            option_match = re.match(
                r'\s+(-\w)?(?:,\s+)?(--[\w-]+)(?:\s+<([^>]+)>|\s*=\s*([^\s]+))?\s+(.*)',
                line
            )

            if option_match:
                short_flag = option_match.group(1)
                long_flag = option_match.group(2)
                argument = option_match.group(3) or option_match.group(4)
                description = option_match.group(5).strip()

                # 다음 라인들에서 추가 설명 수집
                j = i + 1
                while j < len(option_lines):
                    next_line = option_lines[j]
                    if re.match(r'\s+-', next_line):
                        break
                    if next_line.strip():
                        description += " " + next_line.strip()
                    j += 1

                # enum 선택지 추출
                choices = []
                choices_match = re.search(r'([^|]+(?:\|[^|]+)+)', argument or "")
                if choices_match:
                    choices = [c.strip() for c in choices_match.group(1).split('|')]

                option = CommandOption(
                    name=long_flag.lstrip('-') if long_flag else short_flag.lstrip('-'),
                    short_flag=short_flag,
                    long_flag=long_flag,
                    argument=argument,
                    description=description,
                    is_flag=(argument is None),
                    choices=choices
                )

                command.options.append(option)

                # --json 플래그 감지
                if long_flag == '--json':
                    command.supports_json = True

                i = j
            else:
                i += 1


class JSONSchemaExtractor:
    """JSON 출력 스키마 추출기"""

    def __init__(self, rhwp_binary: Path):
        self.rhwp_binary = rhwp_binary

    def extract_schema(self, command: Command, sample_files: List[Path] = None) -> Optional[Dict[str, Any]]:
        """명령어의 JSON 출력 스키마 추출"""
        if not command.supports_json:
            return None

        # 샘플 파일로 JSON 출력 얻기
        if not sample_files:
            # 기본 샘플 파일 찾기
            sample_files = self._find_sample_files()

        if not sample_files:
            return None

        # 여러 샘플에서 JSON 수집
        samples = []
        for sample_file in sample_files[:3]:  # 최대 3개 샘플
            try:
                result = subprocess.run(
                    [str(self.rhwp_binary), command.name, str(sample_file), '--json'],
                    capture_output=True,
                    text=True,
                    encoding='utf-8',
                    timeout=30
                )
                if result.returncode == 0 and result.stdout:
                    try:
                        sample = json.loads(result.stdout)
                        samples.append(sample)
                    except json.JSONDecodeError:
                        pass
            except (subprocess.TimeoutExpired, subprocess.SubprocessError):
                pass

        if not samples:
            return None

        # 샘플들로부터 스키마 추론
        return self._infer_schema(samples)

    def _infer_schema(self, samples: List[Dict[str, Any]]) -> Dict[str, Any]:
        """JSON 샘플들로부터 스키마 추론"""
        if not samples:
            return {}

        # 첫 샘플을 기준으로 스키마 생성
        schema = {
            "type": "object",
            "properties": {}
        }

        # 모든 샘플에 공통으로 나타나는 필드만 required로 설정
        all_keys = set(samples[0].keys())
        for sample in samples[1:]:
            all_keys &= set(sample.keys())

        schema["required"] = list(all_keys)

        # 각 필드의 타입 추론
        for key in samples[0].keys():
            schema["properties"][key] = self._infer_field_schema(
                [s.get(key) for s in samples if key in s]
            )

        return schema

    def _infer_field_schema(self, values: List[Any]) -> Dict[str, Any]:
        """필드 값들로부터 스키마 추론"""
        if not values:
            return {"type": "null"}

        # null이 아닌 첫 값으로 타입 결정
        value = next((v for v in values if v is not None), None)

        if value is None:
            return {"type": "null"}

        if isinstance(value, bool):
            return {"type": "boolean"}
        elif isinstance(value, int):
            return {"type": "integer"}
        elif isinstance(value, float):
            return {"type": "number"}
        elif isinstance(value, str):
            return {"type": "string"}
        elif isinstance(value, list):
            if value:
                item_schema = self._infer_field_schema([item for v in values if isinstance(v, list) for item in v])
                return {
                    "type": "array",
                    "items": item_schema
                }
            return {"type": "array"}
        elif isinstance(value, dict):
            return self._infer_schema([v for v in values if isinstance(v, dict)])

        return {}

    def _find_sample_files(self) -> List[Path]:
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
                if len(samples) >= 5:
                    break

        return samples[:5]


class MarkdownGenerator:
    """Markdown 문서 생성기"""

    def __init__(self, commands: Dict[str, Command]):
        self.commands = commands

    def generate(self) -> str:
        """전체 Markdown 문서 생성"""
        sections = [
            self._generate_header(),
            self._generate_toc(),
            self._generate_overview(),
            self._generate_commands_section()
        ]

        return '\n\n'.join(sections)

    def _generate_header(self) -> str:
        """헤더 생성"""
        return """# rhwp CLI Reference

이 문서는 `tools/cli-autodiscovery/autodiscover.py` 도구로 자동 생성되었습니다.

rhwp는 HWP/HWPX/HML 문서를 읽고 편집하며 다양한 형식으로 변환하는 CLI 도구입니다."""

    def _generate_toc(self) -> str:
        """목차 생성"""
        toc = ["## 목차\n"]

        # 명령어 카테고리별로 분류
        categories = {
            "내보내기": [],
            "정보 조회": [],
            "진단/디버그": [],
            "변환": [],
            "기타": []
        }

        for cmd_name in sorted(self.commands.keys()):
            if cmd_name.startswith('export-'):
                categories["내보내기"].append(cmd_name)
            elif cmd_name in ['info', 'capabilities', 'fields', 'search']:
                categories["정보 조회"].append(cmd_name)
            elif cmd_name.startswith('hwp5-') or cmd_name in ['dump', 'dump-note-shape', 'dump-pages', 'dump-records', 'diag']:
                categories["진단/디버그"].append(cmd_name)
            elif cmd_name in ['convert', 'batch', 'edit']:
                categories["변환"].append(cmd_name)
            else:
                categories["기타"].append(cmd_name)

        for category, cmds in categories.items():
            if cmds:
                toc.append(f"### {category}\n")
                for cmd in sorted(cmds):
                    toc.append(f"- [{cmd}](#{cmd.replace('-', '')})")
                toc.append("")

        return '\n'.join(toc)

    def _generate_overview(self) -> str:
        """개요 섹션 생성"""
        total = len(self.commands)
        json_support = sum(1 for cmd in self.commands.values() if cmd.supports_json)

        return f"""## 개요

- 총 명령어 수: **{total}개**
- JSON 출력 지원: **{json_support}개**
- 자동 발견 도구: `tools/cli-autodiscovery/autodiscover.py`"""

    def _generate_commands_section(self) -> str:
        """명령어 섹션 생성"""
        sections = ["## 명령어\n"]

        for cmd_name in sorted(self.commands.keys()):
            cmd = self.commands[cmd_name]
            sections.append(self._generate_command_doc(cmd))

        return '\n\n'.join(sections)

    def _generate_command_doc(self, cmd: Command) -> str:
        """개별 명령어 문서 생성"""
        lines = [
            f"### {cmd.name}\n",
            f"{cmd.description}\n" if cmd.description else "",
            f"**사용법:**",
            f"```bash",
            f"rhwp {cmd.name} {cmd.usage}",
            f"```\n"
        ]

        if cmd.options:
            lines.append("**옵션:**\n")
            for opt in cmd.options:
                flag_str = ""
                if opt.short_flag and opt.long_flag:
                    flag_str = f"{opt.short_flag}, {opt.long_flag}"
                elif opt.long_flag:
                    flag_str = opt.long_flag
                elif opt.short_flag:
                    flag_str = opt.short_flag

                if opt.argument:
                    flag_str += f" `<{opt.argument}>`"

                lines.append(f"- **{flag_str}**: {opt.description}")

                if opt.choices:
                    lines.append(f"  - 선택지: {', '.join(f'`{c}`' for c in opt.choices)}")

        if cmd.supports_json:
            lines.append("\n**JSON 출력:** 지원됨 (`--json` 플래그)")

            if cmd.json_schema:
                lines.append("\n**JSON 스키마:**")
                lines.append("```json")
                lines.append(json.dumps(cmd.json_schema, indent=2, ensure_ascii=False))
                lines.append("```")

        if cmd.exit_codes:
            lines.append("\n**종료 코드:**")
            for code, desc in sorted(cmd.exit_codes.items()):
                lines.append(f"- `{code}`: {desc}")

        return '\n'.join(lines)


class OpenAPIGenerator:
    """OpenAPI 스펙 생성기"""

    def __init__(self, commands: Dict[str, Command]):
        self.commands = commands

    def generate(self) -> Dict[str, Any]:
        """OpenAPI 3.0 스펙 생성"""
        spec = {
            "openapi": "3.0.0",
            "info": {
                "title": "rhwp CLI",
                "description": "HWP/HWPX/HML 문서 처리 CLI 도구",
                "version": "0.8.2"
            },
            "paths": {},
            "components": {
                "schemas": {}
            }
        }

        # JSON 출력을 지원하는 명령어들을 API 엔드포인트로 변환
        for cmd_name, cmd in self.commands.items():
            if cmd.supports_json and cmd.json_schema:
                path = f"/commands/{cmd_name}"
                spec["paths"][path] = self._generate_path_item(cmd)

                # 스키마 등록
                schema_name = f"{cmd_name.replace('-', '_')}_response"
                spec["components"]["schemas"][schema_name] = cmd.json_schema

        return spec

    def _generate_path_item(self, cmd: Command) -> Dict[str, Any]:
        """명령어를 OpenAPI path item으로 변환"""
        parameters = []

        for opt in cmd.options:
            if not opt.is_flag and opt.long_flag != '--json':
                param = {
                    "name": opt.name,
                    "in": "query",
                    "description": opt.description,
                    "required": False,
                    "schema": {
                        "type": "string"
                    }
                }

                if opt.choices:
                    param["schema"]["enum"] = opt.choices

                parameters.append(param)

        schema_name = f"{cmd.name.replace('-', '_')}_response"

        return {
            "get": {
                "summary": cmd.description,
                "description": f"rhwp {cmd.name} 명령어 실행",
                "parameters": parameters,
                "responses": {
                    "200": {
                        "description": "성공",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": f"#/components/schemas/{schema_name}"
                                }
                            }
                        }
                    }
                }
            }
        }


class CLIAutodiscovery:
    """CLI 자동 발견 메인 클래스"""

    def __init__(self, rhwp_binary: Path, output_dir: Path):
        self.rhwp_binary = rhwp_binary
        self.output_dir = output_dir
        self.commands: Dict[str, Command] = {}

    def discover(self):
        """전체 발견 프로세스 실행"""
        print("rhwp CLI 자동 발견 시작...")

        # 1. --help 파싱
        print("\n[1/4] --help 출력 파싱 중...")
        help_text = self._get_help_text()
        parser = HelpParser(help_text)
        self.commands = parser.parse()
        print(f"  발견된 명령어: {len(self.commands)}개")

        # 2. JSON 스키마 추출
        print("\n[2/4] JSON 출력 스키마 추출 중...")
        schema_extractor = JSONSchemaExtractor(self.rhwp_binary)
        json_commands = [cmd for cmd in self.commands.values() if cmd.supports_json]
        print(f"  JSON 지원 명령어: {len(json_commands)}개")

        for cmd in json_commands:
            print(f"  - {cmd.name} 스키마 추출 중...", end=' ')
            cmd.json_schema = schema_extractor.extract_schema(cmd)
            print("✓" if cmd.json_schema else "✗")

        # 3. 문서 생성
        print("\n[3/4] 문서 생성 중...")
        self._generate_docs()

        # 4. OpenAPI 스펙 생성
        print("\n[4/4] OpenAPI 스펙 생성 중...")
        self._generate_openapi()

        print(f"\n완료! 출력 디렉토리: {self.output_dir}")

    def _get_help_text(self) -> str:
        """rhwp --help 출력 가져오기"""
        try:
            result = subprocess.run(
                [str(self.rhwp_binary), '--help'],
                capture_output=True,
                text=True,
                encoding='utf-8',
                timeout=10
            )
            return result.stdout
        except subprocess.SubprocessError as e:
            print(f"오류: rhwp --help 실행 실패: {e}", file=sys.stderr)
            sys.exit(1)

    def _generate_docs(self):
        """문서 생성 및 저장"""
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Markdown 문서
        md_gen = MarkdownGenerator(self.commands)
        md_content = md_gen.generate()
        md_path = self.output_dir / 'CLI_REFERENCE.md'
        md_path.write_text(md_content, encoding='utf-8')
        print(f"  Markdown: {md_path}")

        # JSON 덤프
        commands_json = {
            name: {
                'name': cmd.name,
                'description': cmd.description,
                'usage': cmd.usage,
                'options': [asdict(opt) for opt in cmd.options],
                'supports_json': cmd.supports_json,
                'json_schema': cmd.json_schema
            }
            for name, cmd in self.commands.items()
        }
        json_path = self.output_dir / 'commands.json'
        json_path.write_text(json.dumps(commands_json, indent=2, ensure_ascii=False), encoding='utf-8')
        print(f"  JSON: {json_path}")

    def _generate_openapi(self):
        """OpenAPI 스펙 생성 및 저장"""
        openapi_gen = OpenAPIGenerator(self.commands)
        spec = openapi_gen.generate()

        openapi_path = self.output_dir / 'openapi.json'
        openapi_path.write_text(json.dumps(spec, indent=2, ensure_ascii=False), encoding='utf-8')
        print(f"  OpenAPI: {openapi_path}")


def main():
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')

    parser = argparse.ArgumentParser(
        description='rhwp CLI 자동 발견 및 문서화 도구'
    )
    parser.add_argument(
        '--rhwp-binary',
        type=Path,
        help='rhwp 바이너리 경로 (기본: cargo run --bin rhwp)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('output/cli-docs'),
        help='출력 디렉토리 (기본: output/cli-docs)'
    )
    parser.add_argument(
        '--build',
        action='store_true',
        help='먼저 cargo build로 바이너리 빌드'
    )

    args = parser.parse_args()

    # rhwp 바이너리 확인/빌드
    if args.rhwp_binary:
        rhwp_binary = args.rhwp_binary
        if not rhwp_binary.exists():
            print(f"오류: rhwp 바이너리를 찾을 수 없습니다: {rhwp_binary}", file=sys.stderr)
            sys.exit(1)
    else:
        # cargo를 사용하여 빌드 또는 실행
        if args.build:
            print("rhwp 빌드 중...")
            result = subprocess.run(['cargo', 'build', '--bin', 'rhwp'], capture_output=True)
            if result.returncode != 0:
                print("오류: cargo build 실패", file=sys.stderr)
                print(result.stderr.decode(), file=sys.stderr)
                sys.exit(1)

        # 빌드된 바이너리 찾기
        target_dir = Path('target/debug')
        rhwp_binary = target_dir / 'rhwp.exe' if sys.platform == 'win32' else target_dir / 'rhwp'

        if not rhwp_binary.exists():
            print(f"오류: 빌드된 바이너리를 찾을 수 없습니다: {rhwp_binary}", file=sys.stderr)
            print("--build 플래그를 사용하거나 --rhwp-binary로 경로를 지정하세요.", file=sys.stderr)
            sys.exit(1)

    # 자동 발견 실행
    autodiscovery = CLIAutodiscovery(rhwp_binary, args.output_dir)
    autodiscovery.discover()


if __name__ == '__main__':
    main()
