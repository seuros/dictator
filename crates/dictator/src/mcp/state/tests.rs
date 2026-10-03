use super::{Mood, ServerState};

#[test]
fn staged_secrets_trump_violation_counts() {
    let mut state = ServerState::default();
    assert_eq!(state.mood(), Mood::Magnanimous);

    state.record_classified_staged(true);
    assert_eq!(state.mood(), Mood::Paranoid);
    assert!(state.mood_dirty, "mood change must notify subscribers");

    state.mood_dirty = false;
    state.record_classified_staged(true);
    assert!(!state.mood_dirty, "unchanged state must not re-notify");

    state.record_classified_staged(false);
    assert_eq!(state.mood(), Mood::Magnanimous);
    assert!(state.mood_dirty);
}
