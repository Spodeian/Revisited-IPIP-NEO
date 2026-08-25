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

