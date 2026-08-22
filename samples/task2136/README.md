# Task #2136 재현 샘플

## neartop_reset_sb2500.hwpx (합성)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격). 실문서 재현원은 hwpdocs
  `148753276_제3회연구노트확산세미나(김호영_최종).hwp` pi46 (p4 used 942px > 본문
  933.6px 과적, 한글 p5 — 10k r12 PI TAIL_PUSH 계열).
- 형상: pi0 채움(저장 흐름 하단 64000HU > 60000) → pi1 텍스트 문단, **저장 vpos=2500 =
  sb(5000유닛=2500HU) 정확 일치**.
- 정답지: `pdf/task2136/neartop_reset_sb2500-2020.pdf` — 한글 2020 은 두 문단을
  **1쪽**에 낸다 (본문 상단 70.85pt 기준 y=70.7pt / y=111.7pt).
  `tests/fixtures/render_page_samples.tsv` 의 `hangul_pages=1` 과 같다.
- **주의(#5921).** 이 합성 샘플은 재현원과 달리 **과적이 아니다.** 본문 933.6px 중
  pi0 이 853.3px 를 써서 잔여 80.3px, pi1 필요 높이는 63.2px(sb 33.3 + 줄 29.9) 로
  1쪽에 그대로 들어간다. #2136 이 `native_near_top_reset` 상한을 2000→2500HU 로
  넓히면서 이 형상까지 무조건 쪽 경계로 승격시켜 정본 1쪽 대비 2쪽(delta +1)이
  됐고, #5921 에서 확장 구간 `2000 < cv <= 2500` 에 "직전 쪽에 들어가면 리셋이
  아니다" 항을 더해 1쪽으로 되돌렸다. 재현원(과적)은 그 항이 거짓이라 영향 없다.
- 검증: `rhwp dump-pages samples/task2136/neartop_reset_sb2500.hwpx` (1쪽) /
  `tests/issue_2136_neartop_reset_sb2500.rs`
