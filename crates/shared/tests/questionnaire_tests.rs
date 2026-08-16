use shared::{load_optimized_questions, QuestionnaireState, Response, ScoreTier};

#[test]
fn test_dataset_item_count_and_uniqueness() {
    let questions = load_optimized_questions();
    assert_eq!(questions.len(), 221, "Dataset must have exactly 221 questions");

    let mut seen_ids = std::collections::HashSet::new();
    let mut seen_labels = std::collections::HashSet::new();

    for q in &questions {
        assert!(seen_ids.insert(q.id), "Question ID {} must be unique", q.id);
        assert!(seen_labels.insert(q.label.clone()), "Label {} must be unique", q.label);
        assert!(!q.text.trim().is_empty(), "Question text cannot be empty");
        assert!(q.facet.weight.abs() > 0.0, "Facet weight cannot be zero");
        assert!(q.r#trait.weight.abs() > 0.0, "Trait weight cannot be zero");
        assert!(q.meta_trait.weight.abs() > 0.0, "Meta-Trait weight cannot be zero");
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
    assert_eq!(meta_traits_found.len(), 3, "All 3 meta-traits must be present");
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
    let expected_se = (facet_w * facet_w).sqrt() / facet_w.abs(); // for 1 item, sqrt(w^2)/|w| = 1.0

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
