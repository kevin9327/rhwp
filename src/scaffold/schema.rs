//! `scaffold_schema_v1` serde 모델.
//!
//! `rhwp scaffold` 의 입력 명세다. 에이전트가 **무(無)에서** 유효한 HWPX 문서를
//! 만들기 위해 작성하는 최상위 JSON 이며, `version="1"` 로 고정한다.
//!
//! 설계 원칙(왕복 정직성): 이 스키마는 rhwp 가 파싱해 되읽었을 때 **바이트 그대로**
//! 복원되는 기능만 노출한다 — 문서 제목, 개요 수준 제목(1~7), 본문 문단, 단순 표.
//! 미지 필드는 조용히 버리지 않고 [`Block`] 의 수동 `Deserialize` 로 즉시 거부한다
//! (`src/parser/ingest/schema.rs` 의 `StemBlock` 규약과 정합 — 기계 생성 입력은
//! 관용 파싱의 이득이 없고 실패는 빠를수록 싸다).

use serde::{Deserialize, Serialize};

/// 지원하는 스키마 버전 — 정의는 버전 단일 출처(#4329)인
/// `schema_registry` 에 있고 여기서는 재수출만 한다.
pub use crate::schema_registry::SCAFFOLD_SCHEMA_VERSION;

/// 문서 전체 — 에이전트가 작성하는 JSON 최상위.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldSpec {
    /// 스키마 버전 (현재 "1" 만 허용).
    pub version: String,

    /// 문서 제목. 있으면 본문 최상단에 가운데 정렬 제목 문단으로 출력한다.
    #[serde(default)]
    pub title: Option<String>,

    /// 기본 글꼴 이름.
    #[serde(default = "default_font")]
    pub font: String,

    /// 페이지 크기 (mm). 미지정 시 A4(210×297).
    #[serde(default = "default_page_size")]
    pub page_size: PageSize,

    /// 본문 블록 시퀀스 (제목/문단/표).
    #[serde(default)]
    pub blocks: Vec<Block>,
}

fn default_font() -> String {
    "함초롬바탕".to_string()
}

fn default_page_size() -> PageSize {
    PageSize {
        width_mm: 210.0,
        height_mm: 297.0,
    }
}

/// 페이지 크기 (mm 단위).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSize {
    pub width_mm: f32,
    pub height_mm: f32,
}

/// 문단 정렬. 값은 소문자 문자열(`"left"`/`"center"`/`"right"`/`"justify"`)로
/// 받는다 — 오타는 `serde` 가 알 수 없는 variant 오류로 즉시 거부한다.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParagraphAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// 문단의 정렬·글자 서식을 한 객체로 묶는다 — 속성 하나마다 최상위 필드를
/// 따로 추가하지 않고, 왕복 검증을 마친 서식 축을 이 구조체 하나에 계속
/// 얹어 나간다(스키마 문법 변경 없이 확장). 전부 선택 필드이며 생략하면
/// 문서 기본값(양쪽맞춤·보통체)을 쓴다.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ParagraphStyle {
    #[serde(default)]
    pub align: Option<ParagraphAlign>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
    /// 취소선. 정정·무효 표시(예: "당초 계획" 위에 그어 폐기함을 나타냄)에 쓴다 —
    /// 공문서에서 자간 조정으로 흉내 내던 것을 실제 취소선 속성으로 대체한다.
    #[serde(default)]
    pub strikethrough: Option<bool>,
    /// 아래 첨자(예: 화학식 H₂O, 각주 표시). `superscript`와 동시에 `true`면 즉시
    /// 거부(같은 텍스트가 위·아래 첨자를 동시에 가질 수 없음).
    #[serde(default)]
    pub subscript: Option<bool>,
    /// 위 첨자(예: 제곱 x², 각주 번호, 서수 1st). `subscript`와 상호 배타.
    #[serde(default)]
    pub superscript: Option<bool>,
    /// 글자 색 `"#RRGGBB"`(6자리 16진수, `#` 포함 7글자). 형식이 다르면 즉시
    /// 거부. 강조 문구(경고·긴급)나 서명란 안내문 등에 쓴다.
    #[serde(default)]
    pub color: Option<String>,
    /// 글자 크기(pt). 제목·각주처럼 본문과 다른 크기가 필요한 문단에 쓴다 —
    /// heading 블록의 고정 크기 체계와 별개로 문단 단위 임의 크기를 준다.
    #[serde(default)]
    pub font_size: Option<f32>,
    /// 왼쪽 여백(mm). 인용문·별첨 목록처럼 본문보다 안쪽으로 들여야 하는
    /// 문단에 쓴다.
    #[serde(default)]
    pub margin_left: Option<f32>,
    /// 오른쪽 여백(mm).
    #[serde(default)]
    pub margin_right: Option<f32>,
    /// 첫 줄 들여쓰기(+, mm) 또는 내어쓰기(-, mm). 공문서의 "1. 2. 3." 항목
    /// 번호 뒤 내어쓰기(음수)나 문단 첫 줄만 들여쓰는 관행(양수)에 쓴다.
    #[serde(default)]
    pub indent: Option<f32>,
    /// 문단 간격 위(pt). 절 제목 앞에 여백을 두는 등, 문단 사이 시각적
    /// 구분에 쓴다.
    #[serde(default)]
    pub spacing_before: Option<f32>,
    /// 문단 간격 아래(pt).
    #[serde(default)]
    pub spacing_after: Option<f32>,
    /// 줄 간격(%, 문서 기본값 160). 서명란처럼 줄 사이를 넓게 벌려 실제
    /// 서명 공간을 만들거나, 표 안 안내문처럼 좁게 눌러 담아야 하는
    /// 문단에 쓴다.
    #[serde(default)]
    pub line_spacing_percent: Option<u16>,
}

impl ParagraphStyle {
    /// 모든 필드가 `None`인가 — `Some(ParagraphStyle::default())`와 필드 생략을
    /// 같은 취급으로 만들어, 문단 IR 생성 쪽에서 "스타일 없음" 분기를 하나로 합친다.
    fn is_empty(&self) -> bool {
        self.align.is_none()
            && self.bold.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
            && self.subscript.is_none()
            && self.superscript.is_none()
            && self.color.is_none()
            && self.font_size.is_none()
            && self.margin_left.is_none()
            && self.margin_right.is_none()
            && self.indent.is_none()
            && self.spacing_before.is_none()
            && self.spacing_after.is_none()
            && self.line_spacing_percent.is_none()
    }

    /// `subscript`/`superscript` 동시 지정, `color` 형식 오류를 즉시 거부한다
    /// (관용 파싱 금지 — 이 모듈의 확립된 원칙).
    fn validate(&self) -> Result<(), String> {
        if self.subscript == Some(true) && self.superscript == Some(true) {
            return Err(
                "style.subscript 와 style.superscript 를 동시에 true 로 줄 수 없습니다".to_string(),
            );
        }
        if let Some(c) = &self.color {
            let valid = c.len() == 7
                && c.starts_with('#')
                && c[1..].chars().all(|ch| ch.is_ascii_hexdigit());
            if !valid {
                return Err(format!(
                    "style.color 는 \"#RRGGBB\" 형식이어야 합니다 (받음: {c:?})"
                ));
            }
        }
        Ok(())
    }
}

/// 본문 블록.
///
/// `Deserialize` 는 수동 구현이다 — serde 의 internally-tagged enum 은
/// `deny_unknown_fields` 를 지원하지 않아 필드 오타·구조 착오(예: paragraph 에 `rows`)가
/// 조용히 무시된다. 전 필드 합집합([`RawBlock`], `deny_unknown_fields`)으로 받은 뒤 type
/// 별 허용 필드를 검증해, 틀린 입력은 무엇이 왜 틀렸는지 힌트가 붙은 오류로 즉시 실패한다.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    /// 개요 수준 제목(1~7). `export-structure` 가 개요 노드로 인식한다.
    Heading {
        /// 개요 수준 (1=최상위). 1 미만은 1로, 7 초과는 7로 클램프한다.
        level: u8,
        /// 제목 텍스트.
        text: String,
    },
    /// 본문 문단 (평문, 한글 포함).
    Paragraph {
        /// 문단 텍스트.
        text: String,
        /// 정렬·글자 서식. 생략하면 문서 기본값(양쪽맞춤·보통체).
        #[serde(default)]
        style: Option<ParagraphStyle>,
    },
    /// 표 (행 × 열). 각 셀은 평문 문자열(단축 표기) 또는
    /// `{"text":..,"row_span":..,"col_span":..}` 객체로 쓴다. 행마다 길이가 다르면
    /// 최대 열 수에 맞춰 빈 셀(row_span=col_span=1)로 채운다(직사각 정규화).
    Table {
        /// 행 목록. 각 행은 셀의 목록이다.
        rows: Vec<Vec<TableCell>>,
        /// 처음 N개 행을 제목 행(`isHeader:true`)으로 표시한다. 0이면 없음.
        /// 표 행 수를 초과하면 빌드 시 오류로 거부된다(`build_scaffold` 참조).
        #[serde(default)]
        header_rows: usize,
    },
}

/// 표 셀 하나. JSON 에서는 평문 문자열(단축 표기, `row_span=col_span=1` 로 취급)
/// 또는 `{"text":..,"row_span":..,"col_span":..}` 객체로 받는다.
///
/// `Deserialize` 를 수동 구현하는 이유는 [`Block`] 과 같다 — 객체 형태에서 미지
/// 필드(오타 등)를 조용히 버리지 않고 즉시 거부해야 하는데, `#[serde(untagged)]` 는
/// 어느 variant 도 맞지 않을 때 뭉뚱그린 오류만 내어 어떤 필드가 문제인지 알려주지
/// 않는다.
/// 표 셀의 세로 정렬. `"top"`/`"center"`/`"bottom"`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CellVerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TableCell {
    /// 셀 텍스트.
    pub text: String,
    /// 세로 병합 개수 (병합 없으면 1). 0은 허용하지 않는다.
    pub row_span: u16,
    /// 가로 병합 개수 (병합 없으면 1). 0은 허용하지 않는다.
    pub col_span: u16,
    /// 셀 배경색 `"#RRGGBB"`. 헤더 행 강조나 합계 행 구분처럼, `header_rows`
    /// 만으로는 못 표현하는 임의 셀 단위 강조에 쓴다. 형식이 다르면 즉시 거부.
    pub background_color: Option<String>,
    /// 셀 안 내용의 세로 정렬. 생략하면 문서 기본값(가운데 정렬)을 쓴다 —
    /// 서명란처럼 셀이 넓고 내용이 아래쪽에 붙어야 하는 경우(bottom)나,
    /// 각주형 안내문처럼 위쪽에 붙어야 하는 경우(top)에 쓴다.
    pub vertical_align: Option<CellVerticalAlign>,
}

impl<'de> Deserialize<'de> for TableCell {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct CellVisitor;

        impl<'de> Visitor<'de> for CellVisitor {
            type Value = TableCell;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "표 셀 문자열 또는 {{text, row_span?, col_span?, background_color?, vertical_align?}} 객체"
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<TableCell, E>
            where
                E: Error,
            {
                Ok(TableCell {
                    text: v.to_string(),
                    row_span: 1,
                    col_span: 1,
                    background_color: None,
                    vertical_align: None,
                })
            }

            fn visit_string<E>(self, v: String) -> Result<TableCell, E>
            where
                E: Error,
            {
                Ok(TableCell {
                    text: v,
                    row_span: 1,
                    col_span: 1,
                    background_color: None,
                    vertical_align: None,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<TableCell, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut text: Option<String> = None;
                let mut row_span: Option<u16> = None;
                let mut col_span: Option<u16> = None;
                let mut background_color: Option<String> = None;
                let mut vertical_align: Option<CellVerticalAlign> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "text" => text = Some(map.next_value()?),
                        "row_span" => row_span = Some(map.next_value()?),
                        "col_span" => col_span = Some(map.next_value()?),
                        "background_color" => background_color = Some(map.next_value()?),
                        "vertical_align" => vertical_align = Some(map.next_value()?),
                        other => {
                            return Err(A::Error::custom(format!(
                                "표 셀에 허용되지 않는 필드 '{other}' (허용: text|row_span|col_span|background_color|vertical_align)"
                            )))
                        }
                    }
                }
                let text =
                    text.ok_or_else(|| A::Error::custom("표 셀 객체에 'text' 필드가 필요합니다"))?;
                let row_span = row_span.unwrap_or(1);
                let col_span = col_span.unwrap_or(1);
                if row_span == 0 || col_span == 0 {
                    return Err(A::Error::custom(
                        "표 셀의 row_span/col_span 은 1 이상이어야 합니다",
                    ));
                }
                if let Some(c) = &background_color {
                    let valid = c.len() == 7
                        && c.starts_with('#')
                        && c[1..].chars().all(|ch| ch.is_ascii_hexdigit());
                    if !valid {
                        return Err(A::Error::custom(format!(
                            "표 셀의 background_color 는 \"#RRGGBB\" 형식이어야 합니다 (받음: {c:?})"
                        )));
                    }
                }
                Ok(TableCell {
                    text,
                    row_span,
                    col_span,
                    background_color,
                    vertical_align,
                })
            }
        }

        deserializer.deserialize_any(CellVisitor)
    }
}

/// [`Block`] 전 변형의 필드 합집합 — 미지 필드 거부와 type 별 검증의 중간층.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    level: Option<u8>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    rows: Option<Vec<Vec<TableCell>>>,
    #[serde(default)]
    header_rows: Option<usize>,
    #[serde(default)]
    style: Option<ParagraphStyle>,
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = RawBlock::deserialize(deserializer)?;
        let forbid =
            |present: bool, block: &str, field: &str, hint: &str| -> Result<(), D::Error> {
                if present {
                    Err(D::Error::custom(format!(
                        "{block} 블록에 허용되지 않는 필드 '{field}' — {hint}"
                    )))
                } else {
                    Ok(())
                }
            };
        match raw.block_type.as_str() {
            "heading" => {
                forbid(
                    raw.rows.is_some(),
                    "heading",
                    "rows",
                    "표는 type:\"table\" 블록을 쓰세요",
                )?;
                forbid(
                    raw.header_rows.is_some(),
                    "heading",
                    "header_rows",
                    "header_rows 는 table 블록 전용입니다",
                )?;
                forbid(
                    raw.style.is_some(),
                    "heading",
                    "style",
                    "style 은 paragraph 블록 전용입니다",
                )?;
                let text = raw
                    .text
                    .ok_or_else(|| D::Error::custom("heading 블록에 'text' 필드가 필요합니다"))?;
                let level = raw.level.ok_or_else(|| {
                    D::Error::custom("heading 블록에 'level' 필드가 필요합니다 (1~7)")
                })?;
                Ok(Block::Heading { level, text })
            }
            "paragraph" => {
                forbid(
                    raw.level.is_some(),
                    "paragraph",
                    "level",
                    "level 은 heading 블록 전용입니다",
                )?;
                forbid(
                    raw.rows.is_some(),
                    "paragraph",
                    "rows",
                    "표는 type:\"table\" 블록을 쓰세요",
                )?;
                forbid(
                    raw.header_rows.is_some(),
                    "paragraph",
                    "header_rows",
                    "header_rows 는 table 블록 전용입니다",
                )?;
                let text = raw
                    .text
                    .ok_or_else(|| D::Error::custom("paragraph 블록에 'text' 필드가 필요합니다"))?;
                if let Some(s) = &raw.style {
                    s.validate().map_err(D::Error::custom)?;
                }
                Ok(Block::Paragraph {
                    text,
                    style: raw.style.filter(|s| !s.is_empty()),
                })
            }
            "table" => {
                forbid(
                    raw.level.is_some(),
                    "table",
                    "level",
                    "level 은 heading 블록 전용입니다",
                )?;
                forbid(
                    raw.text.is_some(),
                    "table",
                    "text",
                    "셀 내용은 'rows' 안에 넣으세요",
                )?;
                forbid(
                    raw.style.is_some(),
                    "table",
                    "style",
                    "style 은 paragraph 블록 전용입니다",
                )?;
                let rows = raw
                    .rows
                    .ok_or_else(|| D::Error::custom("table 블록에 'rows' 필드가 필요합니다"))?;
                let header_rows = raw.header_rows.unwrap_or(0);
                Ok(Block::Table { rows, header_rows })
            }
            other => Err(D::Error::custom(format!(
                "알 수 없는 블록 type '{other}' (지원: heading|paragraph|table)"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "version": "1",
            "title": "2026년 1분기 실적 보고서",
            "font": "함초롬바탕",
            "blocks": [
                {"type": "heading", "level": 1, "text": "개요"},
                {"type": "paragraph", "text": "본 문서는 자동 생성되었습니다."},
                {"type": "heading", "level": 2, "text": "매출 현황"},
                {"type": "table", "rows": [["항목", "값"], ["매출", "100"]]}
            ]
        }"#
    }

    #[test]
    fn parse_sample() {
        let spec: ScaffoldSpec = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(spec.version, "1");
        assert_eq!(spec.title.as_deref(), Some("2026년 1분기 실적 보고서"));
        assert_eq!(spec.blocks.len(), 4);
        assert!(matches!(spec.blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(spec.blocks[1], Block::Paragraph { .. }));
        assert!(matches!(spec.blocks[3], Block::Table { .. }));
    }

    #[test]
    fn defaults_apply() {
        let spec: ScaffoldSpec = serde_json::from_str(r#"{"version":"1","blocks":[]}"#).unwrap();
        assert_eq!(spec.font, "함초롬바탕");
        assert_eq!(spec.page_size.width_mm, 210.0);
        assert!(spec.title.is_none());
    }

    #[test]
    fn paragraph_with_rows_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"paragraph","text":"a","rows":[["x"]]}"#)
            .unwrap_err()
            .to_string();
        assert!(
            e.contains("paragraph 블록에 허용되지 않는 필드 'rows'"),
            "{e}"
        );
    }

    #[test]
    fn heading_without_level_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"heading","text":"제목"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("'level'"), "{e}");
    }

    #[test]
    fn unknown_block_type_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"image","text":"x"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("알 수 없는 블록 type 'image'"), "{e}");
    }

    #[test]
    fn unknown_field_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"paragraph","text":"a","bold":true}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("bold"), "{e}");
    }

    #[test]
    fn top_level_typo_is_rejected() {
        let e = serde_json::from_str::<ScaffoldSpec>(r#"{"version":"1","fnt":"바탕","blocks":[]}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("fnt"), "{e}");
    }

    #[test]
    fn plain_string_cell_is_shorthand_for_span_1() {
        let block: Block =
            serde_json::from_str(r#"{"type":"table","rows":[["항목","값"]]}"#).unwrap();
        let Block::Table { rows, header_rows } = block else {
            panic!("table 이어야 합니다: {block:?}");
        };
        assert_eq!(header_rows, 0);
        assert_eq!(rows[0][0].text, "항목");
        assert_eq!(rows[0][0].row_span, 1);
        assert_eq!(rows[0][0].col_span, 1);
    }

    #[test]
    fn object_cell_carries_spans_and_header_rows() {
        let block: Block = serde_json::from_str(
            r#"{"type":"table","header_rows":1,"rows":[
                [{"text":"제목","row_span":1,"col_span":2}],
                [{"text":"좌"},{"text":"우"}]
            ]}"#,
        )
        .unwrap();
        let Block::Table { rows, header_rows } = block else {
            panic!("table 이어야 합니다: {block:?}");
        };
        assert_eq!(header_rows, 1);
        assert_eq!(rows[0][0].col_span, 2);
        assert_eq!(rows[1][0].text, "좌");
        assert_eq!(rows[1][0].row_span, 1);
    }

    #[test]
    fn cell_object_unknown_field_is_rejected() {
        let e = serde_json::from_str::<Block>(
            r#"{"type":"table","rows":[[{"text":"a","bold":true}]]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("표 셀에 허용되지 않는 필드 'bold'"), "{e}");
    }

    #[test]
    fn cell_zero_span_is_rejected() {
        let e = serde_json::from_str::<Block>(
            r#"{"type":"table","rows":[[{"text":"a","row_span":0}]]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("row_span/col_span 은 1 이상"), "{e}");
    }

    #[test]
    fn cell_object_without_text_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"table","rows":[[{"col_span":2}]]}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("'text' 필드가 필요합니다"), "{e}");
    }

    #[test]
    fn heading_with_header_rows_is_rejected() {
        let e = serde_json::from_str::<Block>(
            r#"{"type":"heading","level":1,"text":"a","header_rows":1}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            e.contains("heading 블록에 허용되지 않는 필드 'header_rows'"),
            "{e}"
        );
    }
}
