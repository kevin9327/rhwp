//! Issue #5921 (#2136 후속): 저장 near-top 리셋은 직전 쪽 잔여 공간을 보고 판정한다.
//!
//! Regression shape (samples/task2136/neartop_reset_sb2500.hwpx, 합성):
//! - pi0 채움(저장 흐름 하단 64,000HU = 853.3px) 뒤 pi1 텍스트 문단이 **저장
//!   vpos=2500 = 문단 앞 간격 sb(5000유닛=2500HU)와 정확 일치**.
//! - #2136 은 `native_near_top_reset` 상한을 2000→2500HU 로 넓혀 이 형상을 무조건
//!   쪽 경계로 승격시켰다. 근거는 실문서 148753276 pi46 의 **과적**
//!   (used 942px > 본문 933.6px).
//! - 그런데 이 샘플은 과적이 아니다. 본문 933.6px, pi0 이 853.3px 를 써서 잔여
//!   80.3px, pi1 이 필요로 하는 높이는 sb 33.3 + 줄 29.9 = 63.2px —
//!   **63.2 ≤ 80.3 으로 1쪽에 그대로 들어간다.** 같은 샘플의 한글 2020 정본
//!   `pdf/task2136/neartop_reset_sb2500-2020.pdf` 도 두 문단을 **1쪽**에 낸다
//!   (본문 상단 70.85pt 기준 y=70.7pt / y=111.7pt). `render_page_samples.tsv` 의
//!   `hangul_pages=1` 과도 같다.
//! - 수정: 확장 구간 `2000 < cv <= 2500` 에서 문단이 직전 쪽 저장 흐름에 그대로
//!   들어가면 리셋으로 보지 않는다. 종전 구간 `cv <= 2000` 은 그대로 두어
//!   `samples/basic/sungeo.hwp`(pi63 cv=400, 잔여 28.5px 로 빠듯한 진짜 경계)
//!   같은 문서는 영향받지 않는다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2136/neartop_reset_sb2500.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

/// 잔여 공간에 들어가는 sb 일치 저장 리셋은 쪽을 가르지 않는다 — 한글 2020 정본 1쪽.
#[test]
fn issue_5921_fitting_sb_reset_stays_on_same_page() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        1,
        "잔여 80.3px ≥ 필요 63.2px 이므로 pi1 은 1쪽에 남는다 — 한글 2020 정본과 동일 (#5921)"
    );

    let page1 = doc.dump_page_items(Some(0));
    for pi in ["pi=0", "pi=1"] {
        assert!(
            page1.contains(pi),
            "{pi} 은 1쪽에 있어야 한다 — 쪽 분할은 #5921 회귀\n--- page 1 ---\n{page1}"
        );
    }
}
