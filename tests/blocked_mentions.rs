use markdown_project_manager::parse::remove_blocked_mentions as remove;

const B: &str = "FU-2026-05-23-042";
const C: &str = "FU-2026-05-29-001";

#[test]
fn test_remove_blocked_mentions_without_a_mention_returns_none() {
    assert_eq!(remove("", None), None);
    assert_eq!(remove("fix the parser", None), None);
    assert_eq!(remove("blocked: soon", None), None);
    assert_eq!(remove("blocked-on: FU-x", None), None);
}

#[test]
fn test_remove_blocked_mentions_of_another_id_returns_none() {
    assert_eq!(remove(&format!("blocked: {B}"), Some(C)), None);
    assert_eq!(remove(&format!("blocked: {B}0"), Some(B)), None);
}

#[test]
fn test_remove_blocked_mentions_keeps_every_other_byte() {
    assert_eq!(remove(&format!("a  b blocked: {B}; c  d"), None).as_deref(), Some("a  b c  d"));
    assert_eq!(remove(&format!("blocked-on {B} then\nnext line"), None).as_deref(), Some("then\nnext line"));
    assert_eq!(remove(&format!("do x\nblocked: {B}\nnext line"), None).as_deref(), Some("do x\nnext line"));
    assert_eq!(remove(&format!("Do X — blocked: {B}"), None).as_deref(), Some("Do X"));
    assert_eq!(remove(&format!("Do X (waits; blocked: {B})"), None).as_deref(), Some("Do X (waits)"));
}

#[test]
fn test_remove_blocked_mentions_with_brackets_removes_the_closing_bracket() {
    assert_eq!(remove(&format!("go blocked: [{B}] now"), None).as_deref(), Some("go now"));
}

#[test]
fn test_remove_blocked_mentions_on_a_list_removes_only_the_named_id() {
    let list = format!("blocked: {B}; {C}");
    assert_eq!(remove(&list, Some(B)).as_deref(), Some(format!("blocked: {C}").as_str()));
    assert_eq!(remove(&list, Some(C)).as_deref(), Some(format!("blocked: {B}").as_str()));
    assert_eq!(remove(&list, None).as_deref(), Some(""));
}

#[test]
fn test_remove_blocked_mentions_on_two_mentions_removes_only_the_named_one() {
    let text = format!("blocked: {B} and blocked: {C}");
    assert_eq!(remove(&text, Some(B)).as_deref(), Some(format!("and blocked: {C}").as_str()));
}
