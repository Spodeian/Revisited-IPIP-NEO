use crate::questionnaire::{Aspect, Facet, MetaTrait, QuestionnaireState, Trait};
use serde::Serialize;

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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

pub fn export_to_printable_html(state: &QuestionnaireState) -> String {
    let report = FullAssessmentReport::from_state(state);

    let mut rows_meta = String::new();
    for c in &report.meta_traits {
        let tier = c.tier.as_deref().unwrap_or("N/A");
        let score = c.normalized_score.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        let se = c.standard_error.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        rows_meta.push_str(&format!(
            "<tr><td><strong>{}</strong></td><td><span class='badge'>{}</span></td><td>{}</td><td>{}</td><td>{:.2}</td><td>{}/{}</td></tr>",
            c.name, tier, score, se, c.raw_score, c.answered_items, c.total_items
        ));
    }

    let mut rows_traits = String::new();
    for c in &report.traits {
        let tier = c.tier.as_deref().unwrap_or("N/A");
        let score = c.normalized_score.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        let se = c.standard_error.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        rows_traits.push_str(&format!(
            "<tr><td><strong>{}</strong></td><td><span class='badge'>{}</span></td><td>{}</td><td>{}</td><td>{:.2}</td><td>{}/{}</td></tr>",
            c.name, tier, score, se, c.raw_score, c.answered_items, c.total_items
        ));
    }

    let mut rows_facets = String::new();
    for c in &report.facets {
        let tier = c.tier.as_deref().unwrap_or("N/A");
        let score = c.normalized_score.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        let se = c.standard_error.map_or("N/A".to_string(), |s| format!("{:.3}", s));
        rows_facets.push_str(&format!(
            "<tr><td>{}</td><td><span class='badge'>{}</span></td><td>{}</td><td>{}</td><td>{:.2}</td><td>{}/{}</td></tr>",
            c.name, tier, score, se, c.raw_score, c.answered_items, c.total_items
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Revisited IPIP-NEO Personality Assessment Report</title>
<style>
    body {{
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
        color: #1a1a1a;
        background: #ffffff;
        margin: 0;
        padding: 32px;
        line-height: 1.5;
    }}
    .header {{
        border-bottom: 2px solid #333;
        padding-bottom: 16px;
        margin-bottom: 24px;
    }}
    h1 {{ margin: 0 0 8px 0; font-size: 24px; }}
    h2 {{ margin: 24px 0 12px 0; font-size: 18px; border-bottom: 1px solid #ddd; padding-bottom: 4px; }}
    .meta-bar {{ color: #555; font-size: 14px; }}
    table {{ width: 100%; border-collapse: collapse; margin-bottom: 24px; font-size: 13px; }}
    th, td {{ border: 1px solid #e0e0e0; padding: 6px 10px; text-align: left; }}
    th {{ background-color: #f5f5f7; font-weight: 600; }}
    .badge {{
        display: inline-block;
        padding: 2px 6px;
        border-radius: 4px;
        background: #eef2ff;
        color: #3730a3;
        font-weight: 600;
        font-size: 12px;
    }}
    @media print {{
        body {{ padding: 0; }}
        button {{ display: none; }}
    }}
</style>
</head>
<body>
<div class="header">
    <h1>Revisited IPIP-NEO Personality Assessment Report</h1>
    <div class="meta-bar">
        Questions Answered: {} / {} ({:.1}%)
    </div>
</div>

<h2>Meta-Traits (Higher-Order Stability & Plasticity)</h2>
<table>
    <thead>
        <tr><th>Meta-Trait</th><th>Tier</th><th>Score [-1, 1]</th><th>SE</th><th>Raw</th><th>Items</th></tr>
    </thead>
    <tbody>
        {}
    </tbody>
</table>

<h2>Traits (Six Core Personality Dimensions)</h2>
<table>
    <thead>
        <tr><th>Trait</th><th>Tier</th><th>Score [-1, 1]</th><th>SE</th><th>Raw</th><th>Items</th></tr>
    </thead>
    <tbody>
        {}
    </tbody>
</table>

<h2>Facets (28 Detailed Construct Subdimensions)</h2>
<table>
    <thead>
        <tr><th>Facet</th><th>Tier</th><th>Score [-1, 1]</th><th>SE</th><th>Raw</th><th>Items</th></tr>
    </thead>
    <tbody>
        {}
    </tbody>
</table>

<script>
window.onload = function() {{
    // Automatically trigger print dialog if requested
}};
</script>
</body>
</html>"#,
        report.answered_questions,
        report.total_questions,
        report.completion_percentage,
        rows_meta,
        rows_traits,
        rows_facets
    )
}
