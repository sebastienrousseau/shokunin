use ssg_template::Context;

#[test]
fn test_context_new() {
    let context = Context::new();
    assert!(context.elements.is_empty());
}

#[test]
fn test_context_set_and_get() {
    let mut context = Context::new();
    context.set("name", "Alice");
    assert_eq!(context.get("name"), Some(&"Alice"));
}

#[test]
fn test_context_update_existing_key() {
    let mut context = Context::new();
    context.set("name", "Alice");
    context.set("name", "Bob");
    assert_eq!(context.get("name"), Some(&"Bob"));
}

#[test]
fn test_context_get_nonexistent_key() {
    let context = Context::new();
    assert_eq!(context.get("nonexistent"), None);
}

#[test]
fn test_context_multiple_entries() {
    let mut context = Context::new();
    context.set("name", "Alice");
    context.set("age", "30");
    assert_eq!(context.get("name"), Some(&"Alice"));
    assert_eq!(context.get("age"), Some(&"30"));
}
