use shared::{QuestionnaireState, Response, ScoreTier, load_optimized_questions};

#[test]
fn test_dataset_item_count_and_uniqueness() {
    let questions = load_optimized_questions();
    assert_eq!(
        questions.len(),
        221,
        "Dataset must have exactly 221 questions"
    );

    let mut seen_ids = std::collections::HashSet::new();
    let mut seen_labels = std::collections::HashSet::new();

    for q in &questions {
        assert!(seen_ids.insert(q.id), "Question ID {} must be unique", q.id);
        assert!(
            seen_labels.insert(q.label.clone()),
            "Label {} must be unique",
            q.label
        );
        assert!(!q.text.trim().is_empty(), "Question text cannot be empty");
        assert!(q.facet.weight.abs() > 0.0, "Facet weight cannot be zero");
        assert!(q.r#trait.weight.abs() > 0.0, "Trait weight cannot be zero");
        assert!(
            q.meta_trait.weight.abs() > 0.0,
            "Meta-Trait weight cannot be zero"
        );
    }
}

#[test]
fn test_hierarchy_coverage() {
    let questions = load_optimized_questions();
    let mut facets_found = std::collections::HashSet::new();
    let mut traits_found = std::collections::HashSet::new();
    let mut meta_traits_found = std::collections::HashSet::new();

    for q in &questions {
        facets_found.insert(q.facet.category);
        traits_found.insert(q.r#trait.category);
        meta_traits_found.insert(q.meta_trait.category);
    }

    assert_eq!(facets_found.len(), 28, "All 28 facets must be present");
    assert_eq!(traits_found.len(), 6, "All 6 traits must be present");
    assert_eq!(
        meta_traits_found.len(),
        3,
        "All 3 meta-traits must be present"
    );
}

#[test]
fn test_scoring_math_and_se() {
    let mut state = QuestionnaireState::from_embedded_data();

    // Answer first question with Strongly Agree (+1.0)
    let q0 = &state.questions[0];
    let facet = q0.facet.category;
    let facet_w = q0.facet.weight;

    state.answer_question(0, Response::StronglyAgree);

    let acc = state.facet_acc.get(&facet).unwrap();
    assert_eq!(acc.answered_count, 1);
    assert_eq!(acc.raw_score, 1.0 * facet_w);
    assert_eq!(acc.total_abs_weight, facet_w.abs());
    assert_eq!(acc.total_sq_weight, facet_w * facet_w);

    let expected_norm = (1.0 * facet_w) / facet_w.abs();
    let expected_se = ((facet_w * facet_w).sqrt() / facet_w.abs()) * 0.5; // for 1 item, (sqrt(w^2)/|w|) * 0.5 = 0.5

    assert!((acc.normalized_score().unwrap() - expected_norm).abs() < 1e-5);
    assert!((acc.standard_error().unwrap() - expected_se).abs() < 1e-5);
}

#[test]
fn test_score_tier_mapping() {
    assert_eq!(ScoreTier::from_score(-0.8), ScoreTier::VeryLow);
    assert_eq!(ScoreTier::from_score(-0.5), ScoreTier::Low);
    assert_eq!(ScoreTier::from_score(0.0), ScoreTier::Average);
    assert_eq!(ScoreTier::from_score(0.4), ScoreTier::High);
    assert_eq!(ScoreTier::from_score(0.9), ScoreTier::VeryHigh);
}

#[test]
fn test_skip_queue_logic() {
    let mut state = QuestionnaireState::from_embedded_data();
    assert_eq!(state.pending_queue.len(), 221);
    assert_eq!(state.pending_queue.front().copied(), Some(0));
    assert_eq!(state.current_focus_idx, 0);

    // Skip current question (0) -> should move to end of queue and focus on next (1)
    state.skip_current();
    assert_eq!(state.pending_queue.len(), 221);
    assert_eq!(state.pending_queue.front().copied(), Some(1));
    assert_eq!(state.pending_queue.back().copied(), Some(0));
    assert_eq!(state.current_focus_idx, 1);

    // Answer question 1 -> removes from queue and advances
    state.answer_question(1, Response::Agree);
    assert_eq!(state.pending_queue.len(), 220);
    assert_eq!(state.answered_count(), 1);
    assert_eq!(state.pending_queue.front().copied(), Some(2));
    assert_eq!(state.current_focus_idx, 2);
}

#[test]
fn test_navigate_unanswered() {
    let mut state = QuestionnaireState::from_embedded_data();

    // Answer questions 0, 1, and 3, leaving 2 unanswered
    state.answer_question(0, Response::Agree);
    state.answer_question(1, Response::Disagree);
    state.answer_question(3, Response::Neutral);

    // Jump to index 0
    state.current_focus_idx = 0;

    // Navigating forward to the next unanswered should skip 1 and land directly on 2
    state.navigate_next_unanswered();
    assert_eq!(state.current_focus_idx, 2);

    // Navigating forward again should skip 3 and land on 4
    state.navigate_next_unanswered();
    assert_eq!(state.current_focus_idx, 4);

    // Navigating backwards to unanswered should skip 3 and land on 2
    state.navigate_previous_unanswered();
    assert_eq!(state.current_focus_idx, 2);
}

#[test]
fn test_bson_compressed_roundtrip() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyAgree);
    state.answer_question(5, Response::Disagree);
    state.answer_question(10, Response::Neutral);

    let compressed_bson = shared::export_to_compressed_bson(&state).expect("BSON export failed");
    assert!(
        !compressed_bson.is_empty(),
        "Compressed BSON should not be empty"
    );

    let mut restored_state = QuestionnaireState::from_embedded_data();
    let applied = shared::import_responses_from_bson(&mut restored_state, &compressed_bson)
        .expect("BSON import failed");

    assert_eq!(applied, 3);
    assert_eq!(
        restored_state.questions[0].response,
        Some(Response::StronglyAgree)
    );
    assert_eq!(
        restored_state.questions[5].response,
        Some(Response::Disagree)
    );
    assert_eq!(
        restored_state.questions[10].response,
        Some(Response::Neutral)
    );
}

#[test]
fn test_serde_backward_compatibility() {
    // Deserialize an older JSON state missing newer fields
    let legacy_json = r#"{"config":{"theme":"Dark"},"questionnaire":{"questions":[],"pending_queue":[],"current_focus_idx":0,"show_results":false,"show_detailed_stats":false}}"#;
    let app_state: Result<shared::AppState, _> = serde_json::from_str(legacy_json);
    assert!(
        app_state.is_ok(),
        "Should parse legacy state without failure"
    );
}

#[test]
fn test_undo_redo_and_branch_invalidation() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyAgree);
    state.answer_question(1, Response::Agree);

    assert_eq!(state.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(state.questions[1].response, Some(Response::Agree));
    assert!(state.can_undo());
    assert!(!state.can_redo());

    // Undo answering Q1
    assert!(state.undo());
    assert_eq!(state.questions[1].response, None);
    assert!(state.can_redo());

    // Redo answering Q1
    assert!(state.redo());
    assert_eq!(state.questions[1].response, Some(Response::Agree));
    assert!(!state.can_redo());

    // Undo answering Q1 again
    assert!(state.undo());
    assert_eq!(state.questions[1].response, None);
    assert!(state.can_redo());

    // Mutating a new action on this branch invalidates the redo future
    state.answer_question(2, Response::Disagree);
    assert_eq!(state.questions[2].response, Some(Response::Disagree));
    assert!(
        !state.can_redo(),
        "Redo stack must be cleared when a new action is performed"
    );
    assert!(!state.redo(), "Redo should fail after branch divergence");
}

#[test]
fn test_undo_assessment_reset() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyAgree);
    state.answer_question(1, Response::Agree);
    state.answer_question(2, Response::Neutral);

    assert_eq!(state.answered_count(), 3);

    // Reset with undo support
    state.reset_with_undo();
    assert_eq!(state.answered_count(), 0);
    assert_eq!(state.questions[0].response, None);
    assert_eq!(state.questions[1].response, None);
    assert_eq!(state.questions[2].response, None);

    // Undo the clear
    assert!(state.can_undo());
    assert!(state.undo());

    assert_eq!(state.answered_count(), 3);
    assert_eq!(state.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(state.questions[1].response, Some(Response::Agree));
    assert_eq!(state.questions[2].response, Some(Response::Neutral));

    // Redo the clear
    assert!(state.can_redo());
    assert!(state.redo());
    assert_eq!(state.answered_count(), 0);
}

#[test]
fn test_undo_shared_link_overwrite() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyAgree);
    state.answer_question(1, Response::Agree);

    // Friend shares their responses with Q0=Disagree, Q1=Disagree, Q2=Neutral
    let mut friend_responses = vec![None; state.questions.len()];
    friend_responses[0] = Some(Response::Disagree);
    friend_responses[1] = Some(Response::Disagree);
    friend_responses[2] = Some(Response::Neutral);

    state.load_snapshot_with_undo(friend_responses, true, "Friend's Shared Link Loaded");

    // State now has friend's answers
    assert_eq!(state.questions[0].response, Some(Response::Disagree));
    assert_eq!(state.questions[1].response, Some(Response::Disagree));
    assert_eq!(state.questions[2].response, Some(Response::Neutral));

    // User realizes they overwrote their own view and hits Undo
    assert!(state.can_undo());
    assert!(state.undo());

    // User's original responses are restored
    assert_eq!(state.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(state.questions[1].response, Some(Response::Agree));
    assert_eq!(state.questions[2].response, None);

    // Redo restores friend's view
    assert!(state.redo());
    assert_eq!(state.questions[0].response, Some(Response::Disagree));
}

#[test]
fn test_compact_history_exponential_recency() {
    let mut state = QuestionnaireState::from_embedded_data();

    // Perform 40 individual answer changes
    for i in 0..40 {
        state.answer_question(i % 10, Response::Agree);
    }
    assert_eq!(state.undo_stack.len(), 40);

    // Trigger compaction down to target 12 entries
    state.compact_history(12);

    assert!(
        state.undo_stack.len() <= 15,
        "Compacted stack should be significantly smaller"
    );
    assert!(
        state.can_undo(),
        "Compacted history should still support undo"
    );

    // Perform an undo step
    assert!(state.undo());
}

#[test]
fn test_norm_set_z_score_and_percentile_estimates() {
    use shared::{Aspect, MetaTrait, NormParameters, NormSet, ScoreAccumulator};

    let norms = NormSet::theoretical_tga();
    let mut neutral_acc = ScoreAccumulator::default();
    neutral_acc.record_response(0.0, 1.0);

    let meta_norm = norms.for_meta_trait(MetaTrait::Stability);
    let z_zero = neutral_acc.z_score(meta_norm).unwrap();
    let pct_zero = neutral_acc.percentile_estimate(meta_norm).unwrap();
    assert!(z_zero.abs() < 1e-5);
    assert!((pct_zero - 50.0).abs() < 0.25);

    let mut high_acc = ScoreAccumulator::default();
    high_acc.record_response(0.8, 1.0);
    let z_high = high_acc.z_score(meta_norm).unwrap();
    let pct_high = high_acc.percentile_estimate(meta_norm).unwrap();
    assert!(z_high > 1.5);
    assert!(pct_high > 90.0 && pct_high <= 100.0);

    let invalid_norm = NormParameters::new(0.0, 0.0);
    assert!(high_acc.z_score(invalid_norm).is_none());
    assert!(high_acc.percentile_estimate(invalid_norm).is_none());

    let narrative = high_acc
        .narrative(&MetaTrait::Stability)
        .expect("Narrative should exist once answered");
    assert!(narrative.contains(MetaTrait::Stability.display_name()));
    assert!(narrative.contains("Very High"));
}

#[test]
fn test_rapid_112_mode_preserves_answers_and_covers_all_28_facets() {
    use shared::{AdministrationMode, Facet, QuestionnaireState, Response, rapid_112_indices};
    use std::collections::HashSet;

    let mut state = QuestionnaireState::from_embedded_data();
    assert_eq!(state.admin_mode, AdministrationMode::Full221);
    assert_eq!(state.active_total_count(), 221);

    let rapid_idx = rapid_112_indices(&state.questions);
    assert_eq!(rapid_idx.len(), 112);

    let covered_facets: HashSet<Facet> = rapid_idx
        .iter()
        .map(|&idx| state.questions[idx].facet.category)
        .collect();
    assert_eq!(
        covered_facets.len(),
        Facet::ALL.len(),
        "Rapid 112 protocol must cover all 28 facets"
    );

    // Record answers under Full221 first
    for i in 0..8 {
        state.answer_question(i, Response::Agree);
    }

    state.set_administration_mode(AdministrationMode::Rapid112);
    assert_eq!(state.active_total_count(), 112);
    assert_eq!(state.total_questions(), 221);

    // Every previously recorded answer must remain intact
    for i in 0..8 {
        assert_eq!(state.questions[i].response, Some(Response::Agree));
    }

    // Complete remaining active Rapid112 items
    for idx in rapid_idx {
        if state.questions[idx].response.is_none() {
            state.answer_question(idx, Response::Neutral);
        }
    }
    assert!(state.is_completed());
    assert_eq!(state.unanswered_count(), 0);
    assert!((state.completion_rate() - 1.0).abs() < 1e-5);
    assert!(state.show_results);

    // Switching back to Full221 exposes remaining unanswered items while preserving all 112+ answers
    state.set_administration_mode(AdministrationMode::Full221);
    assert_eq!(state.active_total_count(), 221);
    assert!(!state.is_completed());
    assert!(state.unanswered_count() > 0);
}

#[test]
fn test_report_schema_v2_and_shareable_summary_omits_raw_items() {
    use shared::{
        FullAssessmentReport, QuestionnaireState, REPORT_SCHEMA_VERSION, Response,
        ShareableSummaryReport, export_shareable_summary_json,
    };

    assert_eq!(REPORT_SCHEMA_VERSION, 2);

    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyAgree);
    state.answer_question(1, Response::Disagree);

    let full = FullAssessmentReport::from_state(&state);
    assert_eq!(full.schema_version, 2);

    let summary_json = export_shareable_summary_json(&state);
    assert!(
        !summary_json.contains("\"item_responses\""),
        "Shareable summary must strip raw item_responses"
    );
    let parsed: ShareableSummaryReport =
        serde_json::from_str(&summary_json).expect("Summary JSON should parse cleanly");
    assert_eq!(parsed.schema_version, 2);
    assert_eq!(parsed.meta_traits.len(), 3);
    assert_eq!(parsed.traits.len(), 6);
    assert_eq!(parsed.facets.len(), 28);
    assert!(parsed.meta_traits.iter().any(|m| m.narrative.is_some()));
}

#[test]
fn test_app_state_longitudinal_profile_snapshots_bounded() {
    use shared::{AppState, MAX_PROFILE_HISTORY, Response};

    let mut app_state = AppState::default();
    app_state.questionnaire.answer_question(0, Response::Agree);
    app_state
        .questionnaire
        .answer_question(1, Response::StronglyAgree);

    for i in 1..=(MAX_PROFILE_HISTORY + 5) {
        let snap = app_state.record_profile_snapshot(format!("Snap {}", i), format!("t={}", i));
        assert_eq!(snap.meta_scores.len(), 3);
        assert_eq!(snap.trait_scores.len(), 6);
        assert_eq!(snap.facet_scores.len(), 28);
    }

    assert_eq!(app_state.profile_history.len(), MAX_PROFILE_HISTORY);
    assert_eq!(app_state.profile_history.first().unwrap().label, "Snap 6");
    assert_eq!(app_state.profile_history.last().unwrap().label, "Snap 55");
}
