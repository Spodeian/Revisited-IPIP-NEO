use criterion::{Criterion, criterion_group, criterion_main};
use shared::{
    FullAssessmentReport, MetaTrait, NormSet, QuestionnaireState, Response, Trait,
    decode_responses_from_url_code, encode_responses_to_url_code, export_shareable_summary_json,
    load_optimized_questions,
};
use std::hint::black_box;

fn populated_state() -> QuestionnaireState {
    let mut state = QuestionnaireState::from_embedded_data();
    let choices = [
        Response::StronglyDisagree,
        Response::Disagree,
        Response::Neutral,
        Response::Agree,
        Response::StronglyAgree,
    ];
    for (i, q) in state.questions.iter_mut().enumerate() {
        q.response = Some(choices[i % choices.len()]);
    }
    state.rebuild_cache();
    state
}

fn bench_question_loading(c: &mut Criterion) {
    c.bench_function("load_optimized_questions_cached", |b| {
        b.iter(|| black_box(load_optimized_questions()));
    });
}

fn bench_rebuild_cache(c: &mut Criterion) {
    let template = populated_state();
    c.bench_function("questionnaire_rebuild_cache_221", |b| {
        b.iter(|| {
            let mut s = template.clone();
            s.rebuild_cache();
            black_box(s.meta_trait_acc.len())
        });
    });
}

fn bench_norming_and_narratives(c: &mut Criterion) {
    let state = populated_state();
    let norms = NormSet::theoretical_tga();
    c.bench_function("tga_norming_and_narratives", |b| {
        b.iter(|| {
            let mut count = 0usize;
            for &m in &MetaTrait::ALL {
                if let Some(acc) = state.meta_trait_acc.get(&m) {
                    black_box(acc.z_score(norms.for_meta_trait(m)));
                    black_box(acc.percentile_estimate(norms.for_meta_trait(m)));
                    black_box(acc.narrative(&m));
                    count += 1;
                }
            }
            for &t in &Trait::ALL {
                if let Some(acc) = state.trait_acc.get(&t) {
                    black_box(acc.z_score(norms.for_trait(t)));
                    black_box(acc.percentile_estimate(norms.for_trait(t)));
                    black_box(acc.narrative(&t));
                    count += 1;
                }
            }
            black_box(count)
        });
    });
}

fn bench_exports_and_url_codec(c: &mut Criterion) {
    let state = populated_state();
    let encoded = encode_responses_to_url_code(&state);

    c.bench_function("full_report_generation", |b| {
        b.iter(|| black_box(FullAssessmentReport::from_state(&state).traits.len()));
    });

    c.bench_function("shareable_summary_json_export", |b| {
        b.iter(|| black_box(export_shareable_summary_json(&state)));
    });

    c.bench_function("url_code_roundtrip", |b| {
        b.iter(|| {
            let code = encode_responses_to_url_code(&state);
            let mut target = QuestionnaireState::from_embedded_data();
            let decoded = decode_responses_from_url_code(&mut target, black_box(&encoded)).ok();
            black_box((code.len(), decoded))
        });
    });
}

criterion_group!(
    benches,
    bench_question_loading,
    bench_rebuild_cache,
    bench_norming_and_narratives,
    bench_exports_and_url_codec
);
criterion_main!(benches);
