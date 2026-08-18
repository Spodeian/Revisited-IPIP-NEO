use shared::{export_to_csv, export_to_json, export_to_printable_html, AppState, QuestionnaireState, Response};

#[test]
fn test_state_serialization_roundtrip() {
    let mut state = AppState::default();
    state.questionnaire.answer_question(0, Response::Agree);
    state.questionnaire.answer_question(5, Response::StronglyDisagree);
    state.questionnaire.skip_current();

    let json = serde_json::to_string(&state).expect("Serialize AppState");
    let mut loaded: AppState = serde_json::from_str(&json).expect("Deserialize AppState");
    loaded.questionnaire.rebuild_cache();

    assert_eq!(loaded.questionnaire.answered_count(), 2);
    assert_eq!(loaded.questionnaire.questions[0].response, Some(Response::Agree));
    assert_eq!(loaded.questionnaire.questions[5].response, Some(Response::StronglyDisagree));
    assert_eq!(loaded.questionnaire.pending_queue.len(), 219);
}

#[test]
fn test_clear_and_modify_answer() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::StronglyDisagree);
    assert_eq!(state.answered_count(), 1);

    // Modify answer to Strongly Agree
    state.answer_question(0, Response::StronglyAgree);
    assert_eq!(state.answered_count(), 1);
    assert_eq!(state.questions[0].response, Some(Response::StronglyAgree));

    // Clear answer
    state.clear_response(0);
    assert_eq!(state.answered_count(), 0);
    assert_eq!(state.questions[0].response, None);
    assert_eq!(state.pending_queue.len(), 221);
}

#[test]
fn test_export_generation() {
    let mut state = QuestionnaireState::from_embedded_data();
    state.answer_question(0, Response::Agree);

    let csv = export_to_csv(&state);
    assert!(csv.contains("# CONSTRUCT SCORES"));
    assert!(csv.contains("# ITEM RESPONSES"));

    let json = export_to_json(&state);
    assert!(json.contains("\"total_questions\": 221"));
    assert!(json.contains("\"answered_questions\": 1"));

    let html = export_to_printable_html(&state);
    assert!(html.contains("Revisited IPIP-NEO (TGA) Personality Assessment Report"));
    assert!(html.contains("Hierarchical Psychometric Breakdown"));
}
