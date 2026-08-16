//! `rhwp scaffold` — 스펙(JSON) → 유효 HWPX 문서 생성.
//!
//! rhwp 의 읽기/편집 축과 직교하는 **생성(authoring)** 축이다. 에이전트가 문서를
//! *소비*하는 데서 나아가 구조화된 명세로부터 문서를 *만든다*. 입력은 문서가 아니라
//! 호출자(사용자/에이전트)가 작성한 계획서이므로 출력 봉투는 신뢰 불가 표지가 붙지
//! 않는다(`src/provenance.rs` 의 `scaffold` 항목 참조).
//!
//! 지원 요소는 **왕복 검증으로 통과한 것만** 노출한다 — 문서 제목, 개요 수준 제목(1~7),
//! 본문 문단(한글 포함), 단순 표(행×열 텍스트 셀). 각 요소는 `serialize_hwpx` 로 쓴 뒤
//! `parse_hwpx` 로 되읽어 내용이 그대로 복원됨을 이 모듈의 테스트가 증명한다.

pub mod builder;
pub mod schema;

pub use builder::build_scaffold;
pub use schema::{Block, PageSize, ScaffoldSpec, SCAFFOLD_SCHEMA_VERSION};

use crate::error::HwpError;

/// JSON 바이트로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_bytes(bytes: &[u8]) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_slice(bytes)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    Ok(spec)
}

/// 문자열로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_str(s: &str) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_str(s)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    Ok(spec)
}

fn validate_version(spec: &ScaffoldSpec) -> Result<(), HwpError> {
    if spec.version != SCAFFOLD_SCHEMA_VERSION {
        return Err(HwpError::InvalidFile(format!(
            "지원하지 않는 scaffold 스키마 버전 '{}' (지원: \"{}\")",
            spec.version, SCAFFOLD_SCHEMA_VERSION
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_core::queries::structure::{build_structure, StructureMode};
    use crate::model::control::Control;
    use crate::parser::hwpx::parse_hwpx;
    use crate::serializer::hwpx::roundtrip::roundtrip_ir_diff;
    use crate::serializer::serialize_hwpx;

    fn full_spec() -> ScaffoldSpec {
        parse_scaffold_str(
            r#"{
                "version": "1",
                "title": "2026년 1분기 실적 보고서",
                "font": "함초롬바탕",
                "blocks": [
                    {"type": "heading", "level": 1, "text": "1. 개요"},
                    {"type": "paragraph", "text": "본 보고서는 자동 생성되었습니다."},
                    {"type": "heading", "level": 2, "text": "1.1 매출"},
                    {"type": "paragraph", "text": "매출은 전년 대비 증가했습니다."},
                    {"type": "table", "rows": [
                        ["항목", "1분기", "2분기"],
                        ["매출", "100", "120"],
                        ["영업이익", "20", "25"]
                    ]}
                ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn build_serializes_to_hwpx() {
        let doc = build_scaffold(&full_spec()).unwrap();
        let bytes = serialize_hwpx(&doc).expect("scaffold 문서는 HWPX 로 직렬화되어야 한다");
        assert!(
            bytes.len() > 100,
            "생성 산출물이 비어있다: {}바이트",
            bytes.len()
        );
    }

    /// 왕복 안정성: 생성 바이트를 파싱→재직렬화→재파싱했을 때 IR 차이가 없어야 한다.
    #[test]
    fn roundtrip_ir_is_stable() {
        let doc = build_scaffold(&full_spec()).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let diff = roundtrip_ir_diff(&bytes).expect("왕복 IR diff 계산");
        assert!(
            diff.is_empty(),
            "생성 HWPX 왕복이 안정적이지 않다: {:?}",
            diff.differences
        );
    }

    /// 본문 문단 텍스트가 바이트 그대로 복원된다.
    #[test]
    fn paragraph_text_round_trips_verbatim() {
        let doc = build_scaffold(&full_spec()).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("생성 HWPX 재파싱");
        let texts: Vec<String> = reparsed.sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect();
        assert!(
            texts.iter().any(|t| t == "2026년 1분기 실적 보고서"),
            "제목이 복원되지 않았다: {texts:?}"
        );
        assert!(
            texts
                .iter()
                .any(|t| t == "본 보고서는 자동 생성되었습니다."),
            "본문 문단이 복원되지 않았다: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "매출은 전년 대비 증가했습니다."),
            "본문 문단이 복원되지 않았다: {texts:?}"
        );
    }

    /// `style.align` 이 실제로 para_shape 의 alignment 로 왕복되고, 미지정 문단은
    /// 기본(PS_NORMAL, justify)을 그대로 쓴다.
    #[test]
    fn paragraph_align_round_trips_to_para_shape() {
        use crate::model::style::Alignment;
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"왼쪽","style":{"align":"left"}},
                {"type":"paragraph","text":"가운데","style":{"align":"center"}},
                {"type":"paragraph","text":"기본"}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let paras = &reparsed.sections[0].paragraphs;
        let align_of = |text: &str| -> Alignment {
            let p = paras.iter().find(|p| p.text == text).unwrap_or_else(|| {
                panic!(
                    "문단을 못 찾음: {text} (실제: {:?})",
                    paras.iter().map(|p| &p.text).collect::<Vec<_>>()
                )
            });
            reparsed.doc_info.para_shapes[p.para_shape_id as usize].alignment
        };
        assert_eq!(align_of("왼쪽"), Alignment::Left);
        assert_eq!(align_of("가운데"), Alignment::Center);
        assert_eq!(align_of("기본"), Alignment::Justify);
    }

    /// `style.bold`/`italic`/`underline` 이 실제로 char_shape 로 왕복된다.
    #[test]
    fn paragraph_char_style_round_trips_to_char_shape() {
        use crate::model::style::UnderlineType;
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"굵게","style":{"bold":true}},
                {"type":"paragraph","text":"기울임+밑줄","style":{"italic":true,"underline":true}},
                {"type":"paragraph","text":"기본"}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let paras = &reparsed.sections[0].paragraphs;
        let cs_of = |text: &str| -> crate::model::style::CharShape {
            let p = paras.iter().find(|p| p.text == text).unwrap();
            let cs_id = p.char_shapes[0].char_shape_id as usize;
            reparsed.doc_info.char_shapes[cs_id].clone()
        };
        let bold = cs_of("굵게");
        assert!(bold.bold);
        assert!(!bold.italic);
        assert_eq!(bold.underline_type, UnderlineType::None);

        let styled = cs_of("기울임+밑줄");
        assert!(!styled.bold);
        assert!(styled.italic);
        assert_eq!(styled.underline_type, UnderlineType::Bottom);

        let normal = cs_of("기본");
        assert!(!normal.bold);
        assert!(!normal.italic);
        assert_eq!(normal.underline_type, UnderlineType::None);
    }

    /// strikethrough/subscript/superscript 가 실제로 char_shape 로 왕복된다.
    #[test]
    fn emphasis_marks_round_trip_to_char_shape() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"a","style":{"strikethrough":true}},
                {"type":"paragraph","text":"b","style":{"superscript":true}},
                {"type":"paragraph","text":"c","style":{"subscript":true}}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let cs_of = |text: &str| -> crate::model::style::CharShape {
            let p = reparsed.sections[0]
                .paragraphs
                .iter()
                .find(|p| p.text == text)
                .unwrap();
            let cs_id = p.char_shapes[0].char_shape_id as usize;
            reparsed.doc_info.char_shapes[cs_id].clone()
        };
        assert!(cs_of("a").strikethrough);
        assert!(cs_of("b").superscript);
        assert!(cs_of("c").subscript);
    }

    /// `subscript`와 `superscript`를 동시에 `true`로 주면 파싱 시점에 즉시 거부된다.
    #[test]
    fn subscript_and_superscript_together_is_rejected() {
        let e = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"paragraph","text":"x","style":{"subscript":true,"superscript":true}}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("subscript") && e.contains("superscript"), "{e}");
    }

    /// color/font_size 가 실제로 char_shape 로 왕복된다.
    #[test]
    fn color_and_font_size_round_trip_to_char_shape() {
        let spec = parse_scaffold_str(
            r##"{"version":"1","blocks":[
                {"type":"paragraph","text":"a","style":{"color":"#FF0000"}},
                {"type":"paragraph","text":"b","style":{"font_size":20.0}}
            ]}"##,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let cs_of = |text: &str| -> crate::model::style::CharShape {
            let p = reparsed.sections[0]
                .paragraphs
                .iter()
                .find(|p| p.text == text)
                .unwrap();
            let cs_id = p.char_shapes[0].char_shape_id as usize;
            reparsed.doc_info.char_shapes[cs_id].clone()
        };
        assert_eq!(cs_of("a").text_color, 0x00FF0000);
        assert_eq!(cs_of("b").base_size, 2000); // 20pt * 100
    }

    /// `color`가 `"#RRGGBB"` 형식이 아니면 즉시 거부된다.
    #[test]
    fn invalid_color_format_is_rejected() {
        let e = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"paragraph","text":"x","style":{"color":"red"}}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("color"), "{e}");
    }

    /// margin_left/margin_right/indent 가 실제로 para_shape 로 왕복된다.
    #[test]
    fn margins_round_trip_to_para_shape() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"a","style":{"margin_left":10.0,"margin_right":5.0}},
                {"type":"paragraph","text":"b","style":{"indent":-5.0}}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let ps_of = |text: &str| -> crate::model::style::ParaShape {
            let p = reparsed.sections[0]
                .paragraphs
                .iter()
                .find(|p| p.text == text)
                .unwrap();
            reparsed.doc_info.para_shapes[p.para_shape_id as usize].clone()
        };
        let a = ps_of("a");
        assert_eq!(a.margin_left, 2835); // 10mm * 7200/25.4, round
        assert_eq!(a.margin_right, 1417);
        assert_eq!(ps_of("b").indent, -1417); // -5mm, 음수 보존(내어쓰기)
    }

    /// spacing_before/spacing_after/line_spacing_percent 가 실제로 para_shape
    /// 로 왕복된다.
    #[test]
    fn spacing_round_trips_to_para_shape() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"a","style":{"spacing_before":10.0,"spacing_after":5.0}},
                {"type":"paragraph","text":"b","style":{"line_spacing_percent":200}}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let ps_of = |text: &str| -> crate::model::style::ParaShape {
            let p = reparsed.sections[0]
                .paragraphs
                .iter()
                .find(|p| p.text == text)
                .unwrap();
            reparsed.doc_info.para_shapes[p.para_shape_id as usize].clone()
        };
        let a = ps_of("a");
        assert_eq!(a.spacing_before, 1000); // 10pt*100
        assert_eq!(a.spacing_after, 500);
        assert_eq!(ps_of("b").line_spacing, 200);
    }

    /// 표 셀 `background_color` 가 실제로 셀 border_fill 의 배경색으로
    /// 왕복되고, 지정 없는 셀은 기존 무배경 실선(BF_SOLID)을 그대로 쓴다.
    #[test]
    fn cell_background_color_round_trips() {
        let spec = parse_scaffold_str(
            r##"{"version":"1","blocks":[{"type":"table","rows":[
                [{"text":"헤더","background_color":"#FFFF00"},"평문"]
            ]}]}"##,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let table = reparsed.sections[0]
            .paragraphs
            .iter()
            .find_map(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::Table(t) => Some(t),
                    _ => None,
                })
            })
            .expect("표를 찾지 못함");
        let yellow_cell = table
            .cells
            .iter()
            .find(|c| c.col == 0 && c.row == 0)
            .unwrap();
        let plain_cell = table
            .cells
            .iter()
            .find(|c| c.col == 1 && c.row == 0)
            .unwrap();
        assert_ne!(
            yellow_cell.border_fill_id, plain_cell.border_fill_id,
            "배경색 지정 셀과 미지정 셀이 같은 border_fill_id 를 쓰면 안 된다"
        );
        let bf = &reparsed.doc_info.border_fills[(yellow_cell.border_fill_id - 1) as usize];
        assert_eq!(
            bf.fill.solid.as_ref().map(|s| s.background_color),
            Some(0x00FFFF00)
        );
    }

    /// `background_color` 가 `"#RRGGBB"` 형식이 아니면 즉시 거부된다.
    #[test]
    fn invalid_cell_background_color_is_rejected() {
        let e = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[[{"text":"x","background_color":"yellow"}]]}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("background_color"), "{e}");
    }

    /// 표 셀 `vertical_align` 이 실제로 셀 IR 로 왕복되고, 미지정 셀은 기본
    /// (가운데)을 쓴다.
    #[test]
    fn cell_vertical_align_round_trips() {
        use crate::model::table::VerticalAlign;
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[
                [{"text":"위","vertical_align":"top"},{"text":"아래","vertical_align":"bottom"},"기본"]
            ]}]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).expect("재파싱");
        let table = reparsed.sections[0]
            .paragraphs
            .iter()
            .find_map(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::Table(t) => Some(t),
                    _ => None,
                })
            })
            .expect("표를 찾지 못함");
        let va_of = |col: u16| {
            table
                .cells
                .iter()
                .find(|c| c.col == col && c.row == 0)
                .unwrap()
                .vertical_align
        };
        assert_eq!(va_of(0), VerticalAlign::Top);
        assert_eq!(va_of(1), VerticalAlign::Bottom);
        assert_eq!(va_of(2), VerticalAlign::Center);
    }

    /// `vertical_align` 에 알 수 없는 값을 주면 즉시 거부된다.
    #[test]
    fn invalid_cell_vertical_align_is_rejected() {
        let e = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[[{"text":"x","vertical_align":"middle"}]]}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            e.contains("unknown variant") || e.contains("vertical_align") || e.contains("middle"),
            "{e}"
        );
    }

    /// 같은 style 값을 쓰는 문단 여러 개가 para_shape/char_shape 항목을 중복
    /// 생성하지 않는다.
    #[test]
    fn repeated_style_reuses_same_shapes() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[
                {"type":"paragraph","text":"a","style":{"align":"right","bold":true}},
                {"type":"paragraph","text":"b","style":{"align":"right","bold":true}},
                {"type":"paragraph","text":"c","style":{"align":"right","bold":true}}
            ]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let selected: Vec<_> = doc.sections[0]
            .paragraphs
            .iter()
            .filter(|p| p.text == "a" || p.text == "b" || p.text == "c")
            .collect();
        assert_eq!(selected.len(), 3);
        let ps_ids: Vec<u16> = selected.iter().map(|p| p.para_shape_id).collect();
        let cs_ids: Vec<u32> = selected
            .iter()
            .map(|p| p.char_shapes[0].char_shape_id)
            .collect();
        assert!(
            ps_ids.iter().all(|id| *id == ps_ids[0]),
            "같은 align 이 서로 다른 para_shape_id 를 씀: {ps_ids:?}"
        );
        assert!(
            cs_ids.iter().all(|id| *id == cs_ids[0]),
            "같은 char style 이 서로 다른 char_shape_id 를 씀: {cs_ids:?}"
        );
    }

    /// heading/table 블록에 `style` 을 주면 즉시 거부된다(paragraph 전용 필드).
    #[test]
    fn style_on_non_paragraph_block_is_rejected() {
        let e = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"heading","level":1,"text":"x","style":{"bold":true}}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("style"), "{e}");
    }

    /// 개요 수준 제목이 export-structure(outline)에서 올바른 수준으로 인식된다.
    #[test]
    fn headings_round_trip_with_correct_levels() {
        let doc = build_scaffold(&full_spec()).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).unwrap();
        let structure = build_structure(&reparsed, StructureMode::Outline);
        // 최상위 개요 노드는 "1. 개요"(level 1), 그 자식은 "1.1 매출"(level 2).
        assert_eq!(structure.roots.len(), 1, "구조: {structure:?}");
        assert_eq!(structure.roots[0].level, 1);
        assert_eq!(structure.roots[0].heading, "1. 개요");
        assert_eq!(structure.roots[0].children.len(), 1);
        assert_eq!(structure.roots[0].children[0].level, 2);
        assert_eq!(structure.roots[0].children[0].heading, "1.1 매출");
    }

    /// 표 치수와 셀 텍스트가 그대로 복원된다.
    #[test]
    fn table_round_trips_dims_and_cell_text() {
        let doc = build_scaffold(&full_spec()).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).unwrap();
        let table = reparsed.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Table(t) => Some(t.as_ref()),
                _ => None,
            })
            .expect("표 컨트롤이 복원되어야 한다");
        assert_eq!(table.row_count, 3, "행 수");
        assert_eq!(table.col_count, 3, "열 수");
        let cell_text = |row: u16, col: u16| -> String {
            table
                .cells
                .iter()
                .find(|c| c.row == row && c.col == col)
                .map(|c| {
                    c.paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default()
        };
        assert_eq!(cell_text(0, 0), "항목");
        assert_eq!(cell_text(0, 2), "2분기");
        assert_eq!(cell_text(1, 0), "매출");
        assert_eq!(cell_text(2, 2), "25");
    }

    /// 비어있는 명세도 유효한(문단 1개) 문서를 만든다.
    #[test]
    fn empty_spec_builds_valid_document() {
        let spec = parse_scaffold_str(r#"{"version":"1","blocks":[]}"#).unwrap();
        let doc = build_scaffold(&spec).unwrap();
        assert_eq!(doc.sections.len(), 1);
        assert!(!doc.sections[0].paragraphs.is_empty());
        let bytes = serialize_hwpx(&doc).expect("빈 명세도 직렬화되어야 한다");
        assert!(roundtrip_ir_diff(&bytes).unwrap().is_empty());
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let e = parse_scaffold_str(r#"{"version":"2","blocks":[]}"#).unwrap_err();
        assert!(format!("{e}").contains("스키마 버전"), "{e}");
    }

    /// 개요 수준 7 초과는 7로 클램프된다.
    #[test]
    fn heading_level_is_clamped() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"heading","level":99,"text":"깊은 제목"}]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).unwrap();
        let structure = build_structure(&reparsed, StructureMode::Outline);
        assert_eq!(structure.roots[0].level, 7, "구조: {structure:?}");
    }

    fn table_control(doc: &crate::model::document::Document) -> crate::model::table::Table {
        doc.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Table(t) => Some((**t).clone()),
                _ => None,
            })
            .expect("표 컨트롤이 있어야 한다")
    }

    /// `header_rows`가 지정한 행의 셀만 `is_header=true` 로 되읽힌다.
    #[test]
    fn header_rows_marks_is_header_and_round_trips() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","header_rows":1,"rows":[
                ["항목","값"],
                ["매출","100"]
            ]}]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        let reparsed = parse_hwpx(&bytes).unwrap();
        let table = table_control(&reparsed);
        let header_cell = table
            .cells
            .iter()
            .find(|c| c.row == 0 && c.col == 0)
            .unwrap();
        assert!(
            header_cell.is_header,
            "헤더 행 셀은 is_header=true 여야 한다"
        );
        let body_cell = table
            .cells
            .iter()
            .find(|c| c.row == 1 && c.col == 0)
            .unwrap();
        assert!(
            !body_cell.is_header,
            "헤더 행이 아닌 셀은 is_header=false 여야 한다"
        );
    }

    /// 셀 `col_span`/`row_span`이 실제 표 컨트롤의 병합으로 되읽힌다.
    #[test]
    fn cell_spans_merge_and_round_trip() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[
                [{"text":"제목","col_span":2}],
                [{"text":"좌"},{"text":"우"}]
            ]}]}"#,
        )
        .unwrap();
        let doc = build_scaffold(&spec).unwrap();
        let bytes = serialize_hwpx(&doc).unwrap();
        assert!(roundtrip_ir_diff(&bytes).unwrap().is_empty());
        let reparsed = parse_hwpx(&bytes).unwrap();
        let table = table_control(&reparsed);
        assert_eq!(table.row_count, 2);
        assert_eq!(table.col_count, 2);
        let anchor = table
            .cells
            .iter()
            .find(|c| c.row == 0 && c.col == 0)
            .unwrap();
        assert_eq!(anchor.col_span, 2, "병합된 앵커 셀의 col_span");
        assert_eq!(anchor.row_span, 1);
        // 병합으로 덮인 (0,1)은 앵커로 존재하지 않는다.
        assert!(!table.cells.iter().any(|c| c.row == 0 && c.col == 1));
    }

    /// `header_rows`가 표 행 수를 초과하면 오류로 거부된다.
    #[test]
    fn header_rows_exceeding_row_count_is_rejected() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","header_rows":5,"rows":[["a"]]}]}"#,
        )
        .unwrap();
        let e = build_scaffold(&spec).unwrap_err();
        assert!(
            e.contains("header_rows(5)가 표 행 수(1)를 초과합니다"),
            "{e}"
        );
    }

    /// 병합 사각형이 표 경계를 벗어나면 오류로 거부된다.
    #[test]
    fn merge_out_of_bounds_is_rejected() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[
                [{"text":"a","col_span":3}]
            ]}]}"#,
        )
        .unwrap();
        let e = build_scaffold(&spec).unwrap_err();
        assert!(e.contains("표 크기"), "{e}");
    }

    /// 겹치는 두 병합 요청은 오류로 거부된다.
    #[test]
    fn overlapping_merges_are_rejected() {
        let spec = parse_scaffold_str(
            r#"{"version":"1","blocks":[{"type":"table","rows":[
                [{"text":"a","row_span":2,"col_span":2},{"text":"b"}],
                [{"text":"c","row_span":2}, {"text":"d"}],
                [{"text":"e"},{"text":"f"}]
            ]}]}"#,
        )
        .unwrap();
        let e = build_scaffold(&spec).unwrap_err();
        assert!(e.contains("병합"), "{e}");
    }
}
