---
kind: working
status: active
canonical: mydocs/working/scoreboard.md
last_verified: 2026-08-16
---

# 내부 스코어보드 — 절대 외부(PR/커밋/공개 문서)에 노출하지 않는다

이 문서는 순수 내부 관리용이다. 여기 적힌 수치·비교는 PR 본문·커밋 메시지·
`mydocs/tech/` 설계 문서·코드 주석 어디에도 옮겨 적지 않는다. 모든 공개 산출물은
"rhwp 자체 결정"으로만 서술한다.

## 지표 1 — gym 벤치마크 통과율

- **미측정.** 저장소에 있는 베이스라인(`gym/baselines/claude-fable-5/report.md`,
  194/194)은 구버전 gym(pack 10개) 기준이라 현재 `gym/README.md`(pack 12개·과제
  100건·만점 221)와 맞지 않는다. 재측정하려면 `cargo build --release --bin rhwp`
  후 `python gym/score.py --agent <이름>` 직접 실행 필요 — 아직 안 함.
- **다음 행동**: 재측정 자체를 Phase 3 착수 전 1회 작업으로 큐에 넣는다(빌드
  1회 필요, 짧은 작업).

## 지표 2 — closed-loop 완성도 (locate → plan → act → diff → verify → repair → commit)

실측(2026-08-16, `agent_native_runtime_asset_map.md` 근거):

| 단계 | 상태 | 근거 |
|---|---|---|
| locate | **부분** — 두 체계 불일치 | `docdiff::NodePath`(계층, 문단/셀까지)와 `hwp_doc_tree`의 `p0/t0`(평면, 쪽/표만)가 안 이어짐 |
| plan | **있음** | `run` 계획서 정적 선검증(전량 보고) + `--dry-run` preview |
| act | **있음** | `document_core/commands/` 쓰기 계층 다수, `run` 저널 실행 |
| diff | **부분** — 엔진은 완성, 호출자 0 | `src/docdiff/`(LCS 정렬 의미 diff)가 어디서도 호출 안 됨. `ir-diff`는 구식 재파싱 경로 |
| verify | **있음** | `--expect-*`(rhwp-agent서 승격), `edit --verify`, `render-diff` CI |
| repair | **없음** | 실패→진단→재시도 루프 자체가 없음 — 이번 미션 P1e가 유일한 순수 신축 |
| commit | **있음, 부분 롤백 미검증** | R22 CAS(`preconditions.inputSha256`, 전량 거부) + R23 저널 지문(PR #4925). 부분 실행 후 롤백 여부는 미확인 |

**현재 7단계 중 확실히 "있음" 4개(plan/act/verify/commit), "부분" 2개(locate/diff),
"없음" 1개(repair).** 목표(7/7)까지 남은 것은: diff 배선(재구현 아님, ir-diff를
docdiff 위로 옮기는 것), locate 통합(설계 판단 필요), repair 루프(순수 신축).

## 지표 3 — MCP 도구 커버리지 (내부 기준선, 절대 공개 안 함)

`competitive_baseline_agent_native.md` 근거. 조사 대상 8개 프로젝트 + 참고 1개.

- **이미 상회**: 실행 환경(무설치·크로스플랫폼), 도구 수(83 CLI/82 MCP vs 대부분
  10~35), 보안 콘텐츠 스캔(은닉/주입/유니코드 — 조사 대상 중 유일), 작업
  증빙/재현성(캡슐·계보·서명 — 조사 대상 중 유일).
- **구조적으로 유일할 가능성**: 편집→자동 렌더-diff→CI 게이트를 닫은 곳이
  조사 대상에 없음(`render-diff.yml` + `render-diff` 명령). **단, 이건 신축이 아니라
  이미 있는데 아무도 모르는 상태** — 지표 갱신용 신축 작업 없음.
- **가장 진지한 경쟁 지점(내부 판단용, 절대 공개 언급 금지)**: 트랜잭션
  (원자+롤백+멱등키)과 편집 후 렌더 자기검증을 이미 실전 배치한 경쟁자 1건 확인.
  rhwp의 `run`이 부분 실행 롤백을 갖췄는지가 이 항목의 승부처 — **미검증, 다음
  조사 대상**.
- **rhwp 파생 다운스트림 1건이 rhwp 자신의 쓰기 경로(`exportHwpx()`)를 못 믿고
  우회한 전례**를 확인 — 왕복 경로 신뢰도가 실전에서 시험대에 오른 적이
  있다는 내부 신호로만 기록(공개 언급 금지, 특정 시점 기준이라 현재 유효성 미확인).

## 지표 4 — PR 머지율

- **2026-08-16 기준 kevin9327 오픈 PR 15건** — 관례("10건 내외")를 초과.
  **머지 없이 계속 새 PR을 열면 지표 4가 나빠진다.** 신규 빌더 투입보다
  기존 15건의 리뷰/머지 처리를 병행해야 함.
- 이번 미션에서 새로 연 PR: #4925(R23), #4926(R92), #4927(R91), #4929(SWS 게이트),
  #4930(DAR 층3) — 5건 추가로 20건 근처. **PR 승인 게이트(사용자 승인 후 push/PR)
  를 지금부터 엄격 적용해 무분별 증가를 막는다.**

## 지표 5 — 회귀 0

- 각 PR 단위로 `cargo test`/`clippy`/`rustfmt` 확인됨(개별 PR 본문 참고). 워크스페이스
  전체 통합 실행은 미실시 — 15건 오픈 PR이 전부 머지된 가정하에 통합 테스트가 필요.

## 다음 행동 (score 상승폭 기준 우선순위, 파일 격리)

1. **[진행 요청 예정] W1 도구 등록** — `mcp_tool_definitions()`에 `hwp_ws_*`/`hwp_doc_tree`
   4종 등재 + `agent_knowledge_map.md` 수치 갱신. 위험 최저, 지표2·3 즉시 상승.
2. **[진행 요청 예정] docdiff → ir-diff 배선** — 문자열 재파싱 집계를
   `docdiff::diff_documents` 직접 호출로 교체(출력 계약 유지). 지표2 diff단계 완성.
3. **[대기] LOCATE 통합 설계** — `NodePath` vs `p0/t0` 두 체계를 하나로 합칠지
   분리 문서화만 할지 설계 판단 필요 — 빌더 투입 전에 오케스트레이터가 결정.
4. **[대기] REPAIR LOOP** — 순수 신축, 다른 항목과 파일 겹침 없어 안전하게 배정 가능.
5. **[보류] 신규 PR 억제, 15건 머지 처리 우선** — 지표4 관리를 위해 다음 빌더
   투입 전 사용자와 머지 전략 확인.
