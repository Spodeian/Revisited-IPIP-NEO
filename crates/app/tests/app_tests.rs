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
