//! `remove_blocked_mentions` removes exactly the `next-action` mentions that
//! `derive_edges` reads as a block, so `unblock` persists across a re-import.

use markdown_project_manager::parse::remove_blocked_mentions as remove;

const B: &str = "FU-2026-05-23-042";
const C: &str = "FU-2026-05-29-001";

#[test]
fn test_remove_blocked_mentions_without_a_block_returns_none() {
    for text in ["", "fix the parser", "blocked: soon", "blocked-on: FU-x"] {
        assert_eq!(remove(text, None), None, "{text:?}");
    }
}

#[test]
fn test_remove_blocked_mentions_inside_a_word_is_no_block() {
    assert_eq!(remove(&format!("unblocked: {B} now"), None), None);
    assert_eq!(remove(&format!("re-blocked: {B}"), None), None);
}

#[test]
fn test_remove_blocked_mentions_of_another_id_returns_none() {
    assert_eq!(remove(&format!("blocked: {B}"), Some(C)), None);
    assert_eq!(remove(&format!("blocked: {B}0"), Some(B)), None);
}

#[test]
fn test_remove_blocked_mentions_takes_the_mention_and_one_side_of_spaces() {
    let cases = [
        (format!("a b blocked: {B} c"), "a b c"),
        (format!("see blocked: {B}, first"), "see first"),
        (format!("blocked-on {B} then\nnext line"), "then\nnext line"),
        (format!("do x\nblocked: {B}\nnext line"), "do x\nnext line"),
        (format!("do x — blocked: {B}"), "do x —"),
        (format!("go blocked: [{B}] now"), "go now"),
        (format!("go blocked: [[{B}]] now"), "go now"),
        (format!("blocked: {B}"), ""),
        (format!("é — blocked: {B} — ü"), "é — — ü"),
    ];
    for (text, want) in cases {
        assert_eq!(remove(&text, None).as_deref(), Some(want), "{text:?}");
    }
}

#[test]
fn test_remove_blocked_mentions_on_two_mentions_follows_what_a_reimport_reads() {
    let text = format!("blocked: {B} and blocked: {C}");
    // Only the first mention is a block, so `--on C` finds nothing to remove.
    assert_eq!(remove(&text, Some(C)), None);
    assert_eq!(
        remove(&text, Some(B)).as_deref(),
        Some(format!("and blocked: {C}").as_str())
    );
    assert_eq!(remove(&text, None).as_deref(), Some("and"));
}

#[test]
fn test_remove_blocked_mentions_of_a_repeated_id_removes_each() {
    let text = format!("blocked: {B}; later blocked: {B}");
    assert_eq!(remove(&text, Some(B)).as_deref(), Some("later"));
}

#[test]
fn test_remove_blocked_mentions_before_a_semicolon_drops_the_phrase() {
    let text = format!("blocked: {B}; {C} covers docs");
    assert_eq!(
        remove(&text, Some(B)).as_deref(),
        Some(format!("{C} covers docs").as_str())
    );
}

#[test]
fn test_remove_blocked_mentions_of_a_whole_line_with_a_separator_leaves_no_blank_line() {
    let text = format!("first\nblocked: {B};\nlast");
    assert_eq!(remove(&text, None).as_deref(), Some("first\nlast"));
}

#[test]
fn test_remove_blocked_mentions_after_cjk_text_removes_the_mention() {
    let text = format!("等待blocked: {B}");
    assert_eq!(remove(&text, None).as_deref(), Some("等待"));
}
