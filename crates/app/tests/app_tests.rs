use app::PersonalityApp;
use eframe::{App, Storage};
use shared::{AppState, Response};

#[derive(Default)]
struct MockStorage {
    data: std::collections::HashMap<String, String>,
}

impl Storage for MockStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.data.insert(key.to_owned(), value);
    }
    fn remove_string(&mut self, key: &str) {
        self.data.remove(key);
    }
    fn flush(&mut self) {}
}

#[test]
fn test_personality_app_initialization() {
    let app = PersonalityApp::default();
    assert_eq!(app.state.questionnaire.total_questions(), 221);
    assert_eq!(app.state.questionnaire.answered_count(), 0);
}

#[test]
fn test_personality_app_save_and_load() {
    let mut storage = MockStorage::default();
    let mut app = PersonalityApp::default();

    // Record response
    app.state.questionnaire.answer_question(0, Response::StronglyAgree);

    // Save state
    app.save(&mut storage);

    // Verify storage serialization
    let serialized = storage
        .get_string(eframe::APP_KEY)
        .expect("App key must exist in storage");
    assert!(serialized.contains("StronglyAgree"));

    // Simulate loading state
    let mut loaded_state: AppState = eframe::get_value(&storage, eframe::APP_KEY).unwrap();
    loaded_state.questionnaire.rebuild_cache();
    assert_eq!(loaded_state.questionnaire.answered_count(), 1);
    assert_eq!(
        loaded_state.questionnaire.questions[0].response,
        Some(Response::StronglyAgree)
    );
}

#[test]
fn test_shared_link_does_not_overwrite_persistent_state() {
    let mut storage = MockStorage::default();

    // 1. User has their own saved state with question 0 answered StronglyAgree
    let mut initial_app = PersonalityApp::default();
    initial_app.state.questionnaire.answer_question(0, Response::StronglyAgree);
    initial_app.save(&mut storage);

    // Verify initial user state in storage
    let initial_loaded: AppState = eframe::get_value(&storage, eframe::APP_KEY).unwrap();
    assert_eq!(initial_loaded.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // 2. User loads a shared friend's result link (e.g. friend answered question 0 with Disagree and question 1 with Agree)
    let mut shared_app = PersonalityApp::default();
    shared_app.state.questionnaire.answer_question(0, Response::Disagree);
    shared_app.state.questionnaire.answer_question(1, Response::Agree);
    shared_app.is_viewing_shared_link = true;

    // Trigger save (e.g., auto-save or navigation) while viewing shared link
    shared_app.save(&mut storage);

    // Verify storage was NOT overwritten with friend's answers
    let persistent_after_shared: AppState = eframe::get_value(&storage, eframe::APP_KEY).unwrap();
    assert_eq!(
        persistent_after_shared.questionnaire.questions[0].response,
        Some(Response::StronglyAgree)
    );
    assert_eq!(
        persistent_after_shared.questionnaire.questions[1].response,
        None
    );

    // 3. User modifies/answers a question -> is_viewing_shared_link is cleared
    shared_app.is_viewing_shared_link = false;
    shared_app.state.questionnaire.answer_question(0, Response::Neutral);

    // Trigger save again
    shared_app.save(&mut storage);

    // Verify storage IS now updated with the user's new explicit answer
    let updated_persistent: AppState = eframe::get_value(&storage, eframe::APP_KEY).unwrap();
    assert_eq!(
        updated_persistent.questionnaire.questions[0].response,
        Some(Response::Neutral)
    );
}

#[test]
fn test_restore_saved_instance() {
    let mut app = PersonalityApp::default();

    // User answered question 0 with StronglyAgree
    app.state.questionnaire.answer_question(0, Response::StronglyAgree);
    app.saved_local_state = Some(app.state.clone());

    // Friend's link loaded:
    app.state.questionnaire.answer_question(0, Response::Disagree);
    app.state.questionnaire.answer_question(1, Response::Agree);
    app.is_viewing_shared_link = true;

    assert_eq!(app.state.questionnaire.questions[0].response, Some(Response::Disagree));
    assert!(app.is_viewing_shared_link);

    // User clicks "Return to Saved Instance"
    app.restore_saved_instance();

    assert!(!app.is_viewing_shared_link);
    assert_eq!(app.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app.state.questionnaire.questions[1].response, None);
}

#[test]
fn test_app_undo_functionality() {
    let mut app = PersonalityApp::default();
    assert_eq!(app.state.questionnaire.answered_count(), 0);

    // Answer Q0
    app.state.questionnaire.answer_question(0, Response::StronglyAgree);
    assert_eq!(app.state.questionnaire.answered_count(), 1);
    assert_eq!(app.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // Change Q0
    app.state.questionnaire.answer_question(0, Response::Disagree);
    assert_eq!(app.state.questionnaire.questions[0].response, Some(Response::Disagree));

    // Undo change -> should be StronglyAgree
    assert!(app.state.questionnaire.undo());
    assert_eq!(app.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app.state.questionnaire.answered_count(), 1);

    // Undo first answer -> should be None
    assert!(app.state.questionnaire.undo());
    assert_eq!(app.state.questionnaire.questions[0].response, None);
    assert_eq!(app.state.questionnaire.answered_count(), 0);

    // No more undo steps
    assert!(!app.state.questionnaire.undo());
}

#[test]
fn test_app_grid_matrix_navigation() {
    let mut app = PersonalityApp::default();
    assert!(!app.show_grid_dialog);
    assert_eq!(app.state.questionnaire.current_focus_idx, 0);

    // Toggle grid dialog
    app.show_grid_dialog = true;
    assert!(app.show_grid_dialog);

    // Simulate clicking question 42
    app.state.questionnaire.current_focus_idx = 42;
    assert_eq!(app.state.questionnaire.current_focus_idx, 42);
}

#[test]
fn test_import_from_bytes_all_formats() {
    let mut source_app = PersonalityApp::default();
    source_app.state.questionnaire.answer_question(0, Response::StronglyAgree);
    source_app.state.questionnaire.answer_question(1, Response::Disagree);

    // 1. Test CSV format bytes
    let csv_str = shared::export_to_csv(&source_app.state.questionnaire);
    let mut app_csv = PersonalityApp::default();
    let res = app_csv.import_from_bytes(csv_str.as_bytes(), "backup.csv");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 2);
    assert_eq!(app_csv.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app_csv.state.questionnaire.questions[1].response, Some(Response::Disagree));

    // 2. Test JSON format bytes
    let json_str = shared::export_to_json(&source_app.state.questionnaire);
    let mut app_json = PersonalityApp::default();
    let res = app_json.import_from_bytes(json_str.as_bytes(), "backup.json");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 2);
    assert_eq!(app_json.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app_json.state.questionnaire.questions[1].response, Some(Response::Disagree));

    // 3. Test raw BSON binary bytes
    let bson_bytes = shared::export_to_compressed_bson(&source_app.state.questionnaire).unwrap();
    let mut app_bson = PersonalityApp::default();
    let res = app_bson.import_from_bytes(&bson_bytes, "backup.bson");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 2);
    assert_eq!(app_bson.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app_bson.state.questionnaire.questions[1].response, Some(Response::Disagree));

    // 4. Test Base64-encoded BSON string bytes
    use base64::{engine::general_purpose, Engine as _};
    let b64_str = general_purpose::STANDARD.encode(&bson_bytes);
    let mut app_b64 = PersonalityApp::default();
    let res = app_b64.import_from_bytes(b64_str.as_bytes(), "pasted_b64");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 2);
    assert_eq!(app_b64.state.questionnaire.questions[0].response, Some(Response::StronglyAgree));
    assert_eq!(app_b64.state.questionnaire.questions[1].response, Some(Response::Disagree));
}

#[test]
fn test_load_state_multi_tier_json_and_ron() {
    use app::storage_manager::{deserialize_app_state, load_state_multi_tier, DEDICATED_STORAGE_KEY};

    let mut original_state = AppState::default();
    original_state.questionnaire.answer_question(0, Response::StronglyAgree);
    original_state.questionnaire.answer_question(1, Response::Neutral);

    // 1. Verify JSON serialization roundtrip
    let json_str = serde_json::to_string(&original_state).unwrap();
    let from_json = deserialize_app_state(&json_str).expect("Should deserialize JSON");
    assert_eq!(from_json.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // 2. Verify RON serialization roundtrip
    let ron_str = ron::to_string(&original_state).unwrap();
    let from_ron = deserialize_app_state(&ron_str).expect("Should deserialize RON");
    assert_eq!(from_ron.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // 3. Verify load_state_multi_tier from dedicated storage key containing JSON
    let mut storage_dedicated = MockStorage::default();
    storage_dedicated.set_string(DEDICATED_STORAGE_KEY, json_str.clone());
    let loaded_dedicated = load_state_multi_tier(Some(&storage_dedicated)).expect("Should load from dedicated key");
    assert_eq!(loaded_dedicated.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // 4. Verify load_state_multi_tier from app key containing JSON (backward compatibility)
    let mut storage_json_app = MockStorage::default();
    storage_json_app.set_string(eframe::APP_KEY, json_str);
    let loaded_json_app = load_state_multi_tier(Some(&storage_json_app)).expect("Should load from app key JSON");
    assert_eq!(loaded_json_app.questionnaire.questions[0].response, Some(Response::StronglyAgree));

    // 5. Verify load_state_multi_tier from app key containing RON
    let mut storage_ron_app = MockStorage::default();
    storage_ron_app.set_string(eframe::APP_KEY, ron_str);
    let loaded_ron_app = load_state_multi_tier(Some(&storage_ron_app)).expect("Should load from app key RON");
    assert_eq!(loaded_ron_app.questionnaire.questions[0].response, Some(Response::StronglyAgree));
}

#[test]
fn test_results_persistence_and_completion_restoration() {
    use app::storage_manager::{load_state_multi_tier, DEDICATED_STORAGE_KEY};

    let mut storage = MockStorage::default();
    let mut app = PersonalityApp::default();

    // Answer questions and explicitly toggle show_results
    app.state.questionnaire.answer_question(0, Response::Agree);
    app.state.questionnaire.show_results = true;

    // Save app
    app.save(&mut storage);

    // Verify storage has DEDICATED_STORAGE_KEY
    assert!(storage.get_string(DEDICATED_STORAGE_KEY).is_some());

    // Load state
    let loaded = load_state_multi_tier(Some(&storage)).expect("Should load state");
    assert!(loaded.questionnaire.show_results, "show_results state should be preserved on load");
    assert_eq!(loaded.questionnaire.questions[0].response, Some(Response::Agree));

    // Also test 100% completion auto-reveals results
    let mut complete_state = AppState::default();
    for i in 0..complete_state.questionnaire.questions.len() {
        complete_state.questionnaire.questions[i].response = Some(Response::Neutral);
    }
    complete_state.questionnaire.show_results = false; // explicitly false
    let complete_json = serde_json::to_string(&complete_state).unwrap();

    let mut storage_complete = MockStorage::default();
    storage_complete.set_string(DEDICATED_STORAGE_KEY, complete_json);

    let loaded_complete = load_state_multi_tier(Some(&storage_complete)).expect("Should load complete state");
    assert!(
        loaded_complete.questionnaire.show_results,
        "show_results should be auto-set to true when 100% completed"
    );
}

