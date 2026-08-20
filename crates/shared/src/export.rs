//! Data export and import generators for the Revisited IPIP-NEO Personality Assessment.
//! Supports exporting structured summaries and raw logs to CSV, JSON, vector SVG, and printable HTML,
//! as well as importing previously saved CSV/JSON progress to resume testing.

use crate::questionnaire::{
    parse_csv_line, Aspect, Facet, MetaTrait, QuestionnaireState, Response, Trait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct FullAssessmentReport {
    pub total_questions: usize,
    pub answered_questions: usize,
    pub completion_percentage: f32,
    pub is_completed: bool,
    pub meta_traits: Vec<ConstructReport>,
    pub traits: Vec<ConstructReport>,
    pub facets: Vec<ConstructReport>,
    pub item_responses: Vec<ItemResponseReport>,
}

#[derive(Serialize, Deserialize)]
pub struct ConstructReport {
    pub level: String,
    pub name: String,
    pub tier: Option<String>,
    pub normalized_score: Option<f32>,
    pub standard_error: Option<f32>,
    pub raw_score: f32,
    pub abs_weight: f32,
    pub answered_items: usize,
    pub total_items: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ItemResponseReport {
    pub question_number: usize,
    pub label: String,
    pub text: String,
    pub meta_trait: String,
    pub meta_trait_weight: f32,
    pub r#trait: String,
    pub trait_weight: f32,
    pub facet: String,
    pub facet_weight: f32,
    pub response_label: Option<String>,
    pub response_score: Option<f32>,
}

impl FullAssessmentReport {
    pub fn from_state(state: &QuestionnaireState) -> Self {
        let meta_traits = MetaTrait::ALL
            .iter()
            .map(|&m| {
                let acc = state.meta_trait_acc.get(&m).copied().unwrap_or_default();
                ConstructReport {
                    level: "Meta-Trait".to_string(),
                    name: m.display_name().to_string(),
                    tier: acc.tier().map(|t| t.label().to_string()),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    raw_score: acc.raw_score,
                    abs_weight: acc.total_abs_weight,
                    answered_items: acc.answered_count,
                    total_items: acc.total_items,
                }
            })
            .collect();

        let traits = Trait::ALL
            .iter()
            .map(|&t| {
                let acc = state.trait_acc.get(&t).copied().unwrap_or_default();
                ConstructReport {
                    level: "Trait".to_string(),
                    name: t.display_name().to_string(),
                    tier: acc.tier().map(|t| t.label().to_string()),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    raw_score: acc.raw_score,
                    abs_weight: acc.total_abs_weight,
                    answered_items: acc.answered_count,
                    total_items: acc.total_items,
                }
            })
            .collect();

        let facets = Facet::ALL
            .iter()
            .map(|&f| {
                let acc = state.facet_acc.get(&f).copied().unwrap_or_default();
                ConstructReport {
                    level: "Facet".to_string(),
                    name: f.display_name().to_string(),
                    tier: acc.tier().map(|t| t.label().to_string()),
                    normalized_score: acc.normalized_score(),
                    standard_error: acc.standard_error(),
                    raw_score: acc.raw_score,
                    abs_weight: acc.total_abs_weight,
                    answered_items: acc.answered_count,
                    total_items: acc.total_items,
                }
            })
            .collect();

        let item_responses = state
            .questions
            .iter()
            .map(|q| ItemResponseReport {
                question_number: q.id,
                label: q.label.clone(),
                text: q.text.clone(),
                meta_trait: q.meta_trait.category.display_name().to_string(),
                meta_trait_weight: q.meta_trait.weight,
                r#trait: q.r#trait.category.display_name().to_string(),
                trait_weight: q.r#trait.weight,
                facet: q.facet.category.display_name().to_string(),
                facet_weight: q.facet.weight,
                response_label: q.response.map(|r| r.label().to_string()),
                response_score: q.response.map(|r| r.to_score()),
            })
            .collect();

        Self {
            total_questions: state.total_questions(),
            answered_questions: state.answered_count(),
            completion_percentage: state.completion_rate() * 100.0,
            is_completed: state.is_completed(),
            meta_traits,
            traits,
            facets,
            item_responses,
        }
    }
}

pub fn export_to_json(state: &QuestionnaireState) -> String {
    let report = FullAssessmentReport::from_state(state);
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

pub fn export_to_csv(state: &QuestionnaireState) -> String {
    let mut out = String::new();

    // Section 1: Summary Construct Scores
    out.push_str("# CONSTRUCT SCORES\n");
    out.push_str("Level,Construct,Tier,Normalized Score [-1 to +1],Standard Error (SE),Raw Score,Abs Weight Sum,Answered Items,Total Items\n");

    let report = FullAssessmentReport::from_state(state);
    for c in report
        .meta_traits
        .iter()
        .chain(report.traits.iter())
        .chain(report.facets.iter())
    {
        let tier_str = c.tier.as_deref().unwrap_or("N/A");
        let score_str = c
            .normalized_score
            .map_or("N/A".to_string(), |s| format!("{:.4}", s));
        let se_str = c
            .standard_error
            .map_or("N/A".to_string(), |se| format!("{:.4}", se));

        out.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},{},{:.4},{:.4},{},{}\n",
            c.level, c.name, tier_str, score_str, se_str, c.raw_score, c.abs_weight, c.answered_items, c.total_items
        ));
    }

    out.push_str("\n# ITEM RESPONSES\n");
    out.push_str("Question #,Label,Question Text,Meta-Trait,Meta Weight,Trait,Trait Weight,Facet,Facet Weight,Response,Numeric Score\n");

    for item in &report.item_responses {
        let resp_label = item.response_label.as_deref().unwrap_or("Unanswered");
        let resp_score = item
            .response_score
            .map_or("N/A".to_string(), |s| format!("{:.1}", s));
        let clean_text = item.text.replace('"', "\"\"");

        out.push_str(&format!(
            "{},\"{}\",\"{}\",\"{}\",{:.3},\"{}\",{:.3},\"{}\",{:.3},\"{}\",{}\n",
            item.question_number,
            item.label,
            clean_text,
            item.meta_trait,
            item.meta_trait_weight,
            item.r#trait,
            item.trait_weight,
            item.facet,
            item.facet_weight,
            resp_label,
            resp_score
        ));
    }

    out
}

fn tier_color_hex(tier_label: &str) -> &'static str {
    match tier_label {
        "Very Low" => "#ef4444",
        "Low" => "#f97316",
        "Average" => "#94a3b8",
        "High" => "#22c55e",
        "Very High" => "#3b82f6",
        _ => "#94a3b8",
    }
}

pub fn export_to_svg(state: &QuestionnaireState) -> String {
    let report = FullAssessmentReport::from_state(state);
    let mut svg_elements = String::new();
    let mut y: i32 = 110;

    for &meta in &MetaTrait::ALL {
        let meta_acc = state.meta_trait_acc.get(&meta).copied().unwrap_or_default();
        let meta_tier = meta_acc.tier().map(|t| t.label()).unwrap_or("N/A");
        let meta_score_str = meta_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.2}", s));
        let meta_se_str = meta_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.2}", s));
        let tier_color = tier_color_hex(meta_tier);

        // Meta-Trait Header Bar
        svg_elements.push_str(&format!(
            r#"<rect x="30" y="{}" width="840" height="42" rx="6" fill="{}" stroke="{}" stroke-width="2"/>
<text x="45" y="{}" font-family="system-ui, -apple-system, sans-serif" font-size="15" font-weight="700" fill="{}">META-TRAIT: {}</text>
<rect x="420" y="{}" width="80" height="22" rx="4" fill="{}"/>
<text x="460" y="{}" font-family="system-ui, sans-serif" font-size="11" font-weight="700" fill="{}" text-anchor="middle">{}</text>
<text x="520" y="{}" font-family="system-ui, sans-serif" font-size="12" fill="{}">Score: <tspan font-weight="700" fill="{}">{}</tspan>  (SE: {})  Raw: {:.1}</text>
"#,
            y,
            "#1e293b",
            "#3b82f6",
            y + 26,
            "#93c5fd",
            escape_html(meta.display_name()),
            y + 10,
            tier_color,
            y + 25,
            "#ffffff",
            escape_html(meta_tier),
            y + 26,
            "#cbd5e1",
            "#ffffff",
            meta_score_str,
            meta_se_str,
            meta_acc.raw_score
        ));
        y += 52;

        for trait_item in meta.child_traits() {
            let trait_acc = state.trait_acc.get(&trait_item).copied().unwrap_or_default();
            let trait_tier = trait_acc.tier().map(|t| t.label()).unwrap_or("N/A");
            let trait_score_str = trait_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.2}", s));
            let trait_se_str = trait_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.2}", s));
            let trait_tier_color = tier_color_hex(trait_tier);

            // Trait Section Box
            svg_elements.push_str(&format!(
                r#"<rect x="50" y="{}" width="820" height="34" rx="4" fill="{}" stroke="{}" stroke-width="1"/>
<text x="65" y="{}" font-family="system-ui, sans-serif" font-size="13" font-weight="700" fill="{}">Trait: {}</text>
<rect x="420" y="{}" width="75" height="20" rx="3" fill="{}"/>
<text x="457" y="{}" font-family="system-ui, sans-serif" font-size="10" font-weight="700" fill="{}" text-anchor="middle">{}</text>
<text x="520" y="{}" font-family="system-ui, sans-serif" font-size="11" fill="{}">Score: <tspan font-weight="700" fill="{}">{}</tspan> (SE: {}) Raw: {:.1}</text>
"#,
                y,
                "#0f172a",
                "#334155",
                y + 22,
                "#f8fafc",
                escape_html(trait_item.display_name()),
                y + 7,
                trait_tier_color,
                y + 21,
                "#ffffff",
                escape_html(trait_tier),
                y + 22,
                "#94a3b8",
                "#e2e8f0",
                trait_score_str,
                trait_se_str,
                trait_acc.raw_score
            ));
            y += 40;

            for facet in trait_item.child_facets() {
                let facet_acc = state.facet_acc.get(&facet).copied().unwrap_or_default();
                let facet_tier = facet_acc.tier().map(|t| t.label()).unwrap_or("N/A");
                let facet_score_str = facet_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.2}", s));
                let facet_se_str = facet_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.2}", s));
                let f_color = tier_color_hex(facet_tier);

                // Facet Row
                svg_elements.push_str(&format!(
                    r#"<rect x="70" y="{}" width="800" height="24" rx="3" fill="{}" opacity="0.6"/>
<text x="85" y="{}" font-family="system-ui, sans-serif" font-size="11" fill="{}">└─ {}</text>
<rect x="420" y="{}" width="70" height="16" rx="3" fill="{}"/>
<text x="455" y="{}" font-family="system-ui, sans-serif" font-size="9" font-weight="700" fill="{}" text-anchor="middle">{}</text>
<text x="520" y="{}" font-family="system-ui, sans-serif" font-size="11" fill="{}">Score: <tspan font-weight="600" fill="{}">{}</tspan> (SE: {}) Raw: {:.1} [{}/{} items]</text>
"#,
                    y,
                    "#1e293b",
                    y + 16,
                    "#cbd5e1",
                    escape_html(facet.display_name()),
                    y + 4,
                    f_color,
                    y + 16,
                    "#ffffff",
                    escape_html(facet_tier),
                    y + 16,
                    "#94a3b8",
                    "#f1f5f9",
                    facet_score_str,
                    facet_se_str,
                    facet_acc.raw_score,
                    facet_acc.answered_count,
                    facet_acc.total_items
                ));
                y += 28;
            }
            y += 6;
        }
        y += 14;
    }

    let total_height = y + 40;

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {}" width="900" height="{}">
  <rect width="900" height="{}" fill="{}"/>
  <!-- Header Banner -->
  <text x="30" y="42" font-family="system-ui, -apple-system, sans-serif" font-size="20" font-weight="800" fill="{}">Revisited IPIP-NEO (TGA) Personality Assessment Results</text>
  <text x="30" y="68" font-family="system-ui, sans-serif" font-size="12" fill="{}">Answered: {} / {} questions ({:.1}%) • Model: 3 Meta-Traits ➔ 6 Traits ➔ 28 Facets</text>
  <text x="30" y="86" font-family="system-ui, sans-serif" font-size="11" fill="{}">Methodology DOI: 10.1177/08902070251352590 • Client-side 100% Local Execution</text>
  <line x1="30" y1="96" x2="870" y2="96" stroke="{}" stroke-width="1.5"/>

  <!-- Hierarchical Psychometric Tree -->
  {}

  <!-- Footer -->
  <text x="450" y="{}" font-family="system-ui, sans-serif" font-size="11" fill="{}" text-anchor="middle">Generated by Revisited IPIP-NEO (TGA) • https://tga-ipip-neo.spodeian.trade/</text>
</svg>"#,
        total_height,
        total_height,
        total_height,
        "#0b0f19",
        "#f8fafc",
        "#94a3b8",
        report.answered_questions,
        report.total_questions,
        report.completion_percentage,
        "#64748b",
        "#334155",
        svg_elements,
        total_height - 15,
        "#64748b"
    )
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn export_to_printable_html(state: &QuestionnaireState) -> String {
    let report = FullAssessmentReport::from_state(state);

    let mut hierarchy_html = String::new();

    for &meta in &MetaTrait::ALL {
        let meta_acc = state.meta_trait_acc.get(&meta).copied().unwrap_or_default();
        let meta_tier = meta_acc.tier().map(|t| t.label()).unwrap_or("N/A");
        let meta_score = meta_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.3}", s));
        let meta_se = meta_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.3}", s));

        hierarchy_html.push_str(&format!(
            r#"<div class="meta-card">
    <div class="meta-header">
        <div class="meta-title">
            <span class="level-tag">Meta-Trait</span>
            <h3>{}</h3>
        </div>
        <div class="meta-metrics">
            <span class="badge tier-badge">{}</span>
            <span class="metric"><strong>Score:</strong> {}</span>
            <span class="metric"><strong>SE:</strong> {}</span>
            <span class="metric"><strong>Raw:</strong> {:.2}</span>
            <span class="metric"><strong>Items:</strong> {}/{}</span>
        </div>
    </div>
    <div class="traits-container">"#,
            escape_html(meta.display_name()),
            escape_html(meta_tier),
            meta_score,
            meta_se,
            meta_acc.raw_score,
            meta_acc.answered_count,
            meta_acc.total_items
        ));

        for trait_item in meta.child_traits() {
            let trait_acc = state.trait_acc.get(&trait_item).copied().unwrap_or_default();
            let trait_tier = trait_acc.tier().map(|t| t.label()).unwrap_or("N/A");
            let trait_score = trait_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.3}", s));
            let trait_se = trait_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.3}", s));

            hierarchy_html.push_str(&format!(
                r#"<div class="trait-card">
        <div class="trait-header">
            <div class="trait-title">
                <span class="level-tag trait-tag">Trait</span>
                <h4>{}</h4>
            </div>
            <div class="trait-metrics">
                <span class="badge tier-badge">{}</span>
                <span class="metric"><strong>Score:</strong> {}</span>
                <span class="metric"><strong>SE:</strong> {}</span>
                <span class="metric"><strong>Raw:</strong> {:.2}</span>
                <span class="metric"><strong>Items:</strong> {}/{}</span>
            </div>
        </div>
        <table class="facet-table">
            <thead>
                <tr>
                    <th>Facet (Subdimension)</th>
                    <th>Tier</th>
                    <th>Score [-1, 1]</th>
                    <th>Standard Error (SE)</th>
                    <th>Raw Score</th>
                    <th>Answered</th>
                </tr>
            </thead>
            <tbody>"#,
                escape_html(trait_item.display_name()),
                escape_html(trait_tier),
                trait_score,
                trait_se,
                trait_acc.raw_score,
                trait_acc.answered_count,
                trait_acc.total_items
            ));

            for facet in trait_item.child_facets() {
                let facet_acc = state.facet_acc.get(&facet).copied().unwrap_or_default();
                let facet_tier = facet_acc.tier().map(|t| t.label()).unwrap_or("N/A");
                let facet_score = facet_acc.normalized_score().map_or("N/A".to_string(), |s| format!("{:.3}", s));
                let facet_se = facet_acc.standard_error().map_or("N/A".to_string(), |s| format!("{:.3}", s));

                hierarchy_html.push_str(&format!(
                    r#"<tr>
                        <td><strong>{}</strong></td>
                        <td><span class="badge">{}</span></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{:.2}</td>
                        <td>{}/{}</td>
                    </tr>"#,
                    escape_html(facet.display_name()),
                    escape_html(facet_tier),
                    facet_score,
                    facet_se,
                    facet_acc.raw_score,
                    facet_acc.answered_count,
                    facet_acc.total_items
                ));
            }

            hierarchy_html.push_str(
                r#"            </tbody>
        </table>
    </div>"#
            );
        }

        hierarchy_html.push_str(
            r#"    </div>
</div>"#
        );
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Revisited IPIP-NEO (TGA) Personality Assessment Report</title>
<style>
    body {{
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
        color: #1f2937;
        background: #ffffff;
        margin: 0;
        padding: 28px;
        line-height: 1.45;
        font-size: 13px;
    }}
    .header {{
        border-bottom: 2px solid #374151;
        padding-bottom: 14px;
        margin-bottom: 24px;
    }}
    h1 {{ margin: 0 0 6px 0; font-size: 22px; color: #111827; }}
    h2 {{ margin: 28px 0 14px 0; font-size: 17px; color: #1f2937; border-bottom: 1px solid #e5e7eb; padding-bottom: 6px; }}
    h3 {{ margin: 0; font-size: 16px; color: #1e3a8a; }}
    h4 {{ margin: 0; font-size: 14px; color: #374151; }}
    .meta-bar {{ color: #4b5563; font-size: 13px; font-weight: 500; }}

    .meta-card {{
        border: 2px solid #93c5fd;
        border-radius: 8px;
        background: #f8fafc;
        padding: 16px;
        margin-bottom: 24px;
        page-break-inside: avoid;
    }}
    .meta-header {{
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #bfdbfe;
        padding-bottom: 10px;
        margin-bottom: 14px;
        flex-wrap: wrap;
        gap: 8px;
    }}
    .meta-title {{ display: flex; align-items: center; gap: 8px; }}
    .meta-metrics, .trait-metrics {{ display: flex; gap: 12px; align-items: center; font-size: 12px; }}

    .traits-container {{
        display: flex;
        flex-direction: column;
        gap: 16px;
    }}
    .trait-card {{
        border: 1px solid #cbd5e1;
        border-left: 4px solid #3b82f6;
        border-radius: 6px;
        background: #ffffff;
        padding: 12px 14px;
    }}
    .trait-header {{
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
        flex-wrap: wrap;
        gap: 6px;
    }}
    .trait-title {{ display: flex; align-items: center; gap: 6px; }}

    .level-tag {{
        font-size: 10px;
        text-transform: uppercase;
        font-weight: 700;
        background: #dbeafe;
        color: #1e40af;
        padding: 2px 6px;
        border-radius: 4px;
    }}
    .trait-tag {{
        background: #e2e8f0;
        color: #334155;
    }}

    table {{ width: 100%; border-collapse: collapse; margin-top: 6px; font-size: 12px; }}
    th, td {{ border: 1px solid #e2e8f0; padding: 5px 8px; text-align: left; }}
    th {{ background-color: #f1f5f9; font-weight: 600; color: #334155; }}

    .badge {{
        display: inline-block;
        padding: 2px 6px;
        border-radius: 4px;
        background: #eef2ff;
        color: #3730a3;
        font-weight: 600;
        font-size: 11px;
    }}
    .tier-badge {{
        background: #dbeafe;
        color: #1e40af;
        font-size: 11px;
    }}

    @media print {{
        body {{ padding: 0; }}
        .meta-card {{ border: 1px solid #94a3b8; page-break-inside: avoid; }}
        .trait-card {{ border: 1px solid #cbd5e1; page-break-inside: avoid; }}
    }}
</style>
</head>
<body>
<div class="header">
    <h1>Revisited IPIP-NEO (TGA) Personality Assessment Report</h1>
    <div class="meta-bar">
        Progress: {} / {} questions answered ({:.1}%) • Model: 3 Meta-Traits ➔ 6 Traits ➔ 28 Facets
    </div>
</div>

<h2>Hierarchical Psychometric Breakdown</h2>
{}

</body>
</html>"#,
        report.answered_questions,
        report.total_questions,
        report.completion_percentage,
        hierarchy_html
    )
}

const BASE64_URL_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encodes all current question responses into a compact, URL-safe Base64 string (~148 chars).
pub fn encode_responses_to_url_code(state: &QuestionnaireState) -> String {
    let mut packed_bytes = Vec::with_capacity(state.questions.len().div_ceil(2));
    let mut iter = state.questions.iter();

    while let Some(q1) = iter.next() {
        let v1 = match q1.response {
            None => 0u8,
            Some(Response::StronglyDisagree) => 1,
            Some(Response::Disagree) => 2,
            Some(Response::Neutral) => 3,
            Some(Response::Agree) => 4,
            Some(Response::StronglyAgree) => 5,
        };
        let v2 = match iter.next() {
            None => 0u8,
            Some(q2) => match q2.response {
                None => 0u8,
                Some(Response::StronglyDisagree) => 1,
                Some(Response::Disagree) => 2,
                Some(Response::Neutral) => 3,
                Some(Response::Agree) => 4,
                Some(Response::StronglyAgree) => 5,
            },
        };
        packed_bytes.push((v1 << 3) | v2);
    }

    base64_url_encode(&packed_bytes)
}

/// Decodes responses from a compact URL-safe Base64 string into state.
/// Returns the number of successfully applied answers.
pub fn decode_responses_from_url_code(state: &mut QuestionnaireState, code: &str) -> Result<usize, &'static str> {
    let bytes = base64_url_decode(code.trim())?;
    let mut applied_count = 0;
    let mut q_idx = 0;

    for byte in bytes {
        let v1 = (byte >> 3) & 0x07;
        let v2 = byte & 0x07;

        for val in [v1, v2] {
            if q_idx >= state.questions.len() {
                break;
            }
            let resp = match val {
                0 => None,
                1 => Some(Response::StronglyDisagree),
                2 => Some(Response::Disagree),
                3 => Some(Response::Neutral),
                4 => Some(Response::Agree),
                5 => Some(Response::StronglyAgree),
                _ => None,
            };
            if resp.is_some() {
                applied_count += 1;
            }
            state.questions[q_idx].response = resp;
            q_idx += 1;
        }
    }

    if applied_count > 0 {
        state.rebuild_cache();
        Ok(applied_count)
    } else {
        Err("No answered responses found in shared link code.")
    }
}

fn base64_url_encode(data: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(BASE64_URL_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(BASE64_URL_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if i + 1 < data.len() {
            out.push(BASE64_URL_ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        }
        if i + 2 < data.len() {
            out.push(BASE64_URL_ALPHABET[(triple & 0x3F) as usize] as char);
        }
        i += 3;
    }
    out
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::new();
    let chars: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut i = 0;

    let decode_char = |c: u8| -> Result<u32, &'static str> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'-' | b'+' => Ok(62),
            b'_' | b'/' => Ok(63),
            _ => Err("Invalid character in shared code"),
        }
    };

    while i < chars.len() {
        let c0 = decode_char(chars[i])?;
        let c1 = if i + 1 < chars.len() { decode_char(chars[i + 1])? } else { 0 };
        let c2 = if i + 2 < chars.len() { decode_char(chars[i + 2])? } else { 0 };
        let c3 = if i + 3 < chars.len() { decode_char(chars[i + 3])? } else { 0 };

        let chunk = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;

        out.push(((chunk >> 16) & 0xFF) as u8);
        if i + 2 < chars.len() {
            out.push(((chunk >> 8) & 0xFF) as u8);
        }
        if i + 3 < chars.len() {
            out.push((chunk & 0xFF) as u8);
        }
        i += 4;
    }
    Ok(out)
}

/// Imports and applies answers from a previously exported JSON results string.
/// Returns the number of successfully applied answers.
pub fn import_responses_from_json(state: &mut QuestionnaireState, json_str: &str) -> Result<usize, &'static str> {
    let report: FullAssessmentReport = serde_json::from_str(json_str)
        .map_err(|_| "Invalid JSON format. Please ensure this is a valid Revisited IPIP-NEO results file.")?;

    let mut applied_count = 0;

    // Build a map: Label -> Response Score
    let response_map: HashMap<String, f32> = report
        .item_responses
        .iter()
        .filter_map(|item| {
            item.response_score.map(|score| (item.label.clone(), score))
        })
        .collect();

    for q in state.questions.iter_mut() {
        if let Some(&score) = response_map.get(&q.label) {
            // Map floating point score back to Response enum
            let resp = if (score - 1.0).abs() < 0.1 {
                Some(Response::StronglyAgree)
            } else if (score - 0.5).abs() < 0.1 {
                Some(Response::Agree)
            } else if (score - 0.0).abs() < 0.1 {
                Some(Response::Neutral)
            } else if (score - (-0.5)).abs() < 0.1 {
                Some(Response::Disagree)
            } else if (score - (-1.0)).abs() < 0.1 {
                Some(Response::StronglyDisagree)
            } else {
                None
            };

            if let Some(r) = resp {
                q.response = Some(r);
                applied_count += 1;
            }
        }
    }

    if applied_count > 0 {
        state.rebuild_cache();
        Ok(applied_count)
    } else {
        Err("No matching answered questions found in the imported JSON file.")
    }
}

/// Imports and applies answers from a previously exported CSV results string.
/// Returns the number of successfully applied answers.
pub fn import_responses_from_csv(state: &mut QuestionnaireState, csv_str: &str) -> Result<usize, &'static str> {
    let mut applied_count = 0;
    let mut in_responses_section = false;

    for line in csv_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.contains("# ITEM RESPONSES") {
            in_responses_section = true;
            continue;
        }

        if in_responses_section {
            if line.starts_with('#') || line.starts_with("Question #") {
                continue; // Skip comments and header rows
            }

            let fields = parse_csv_line(line);
            if fields.len() < 11 {
                continue;
            }

            let label = &fields[1];
            let resp_score_str = &fields[10];

            if let Ok(score) = resp_score_str.parse::<f32>() {
                for q in &mut state.questions {
                    if q.label == *label {
                        let resp = if (score - 1.0).abs() < 0.1 {
                            Some(Response::StronglyAgree)
                        } else if (score - 0.5).abs() < 0.1 {
                            Some(Response::Agree)
                        } else if (score - 0.0).abs() < 0.1 {
                            Some(Response::Neutral)
                        } else if (score - (-0.5)).abs() < 0.1 {
                            Some(Response::Disagree)
                        } else if (score - (-1.0)).abs() < 0.1 {
                            Some(Response::StronglyDisagree)
                        } else {
                            None
                        };

                        if let Some(r) = resp {
                            q.response = Some(r);
                            applied_count += 1;
                        }
                    }
                }
            }
        }
    }

    if applied_count > 0 {
        state.rebuild_cache();
        Ok(applied_count)
    } else {
        Err("No matching answered questions found in the imported CSV file.")
    }
}
