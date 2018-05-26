use hxo_types::{Cursor, Position, Span};

#[test]
fn test_position_default() {
    let pos = Position::default();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_cursor_new() {
    let source = "hello world";
    let cursor = Cursor::new(source);
    assert_eq!(cursor.source, source);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 1);
}

#[test]
fn test_cursor_peek() {
    let cursor = Cursor::new("abc");
    assert_eq!(cursor.peek(), 'a');
    assert_eq!(cursor.peek_n(1), 'b');
    assert_eq!(cursor.peek_n(2), 'c');
    assert_eq!(cursor.peek_n(3), '\0');
}

#[test]
fn test_cursor_is_eof() {
    let mut cursor = Cursor::new("a");
    assert!(!cursor.is_eof());
    cursor.consume();
    assert!(cursor.is_eof());
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start.line, 0);
    assert_eq!(span.end.line, 0);
}

#[test]
fn test_cursor_utf16_columns() {
    let mut cursor = Cursor::new("🚀a");

    // Initial state
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 1);

    // After consuming Rocket Emoji (UTF-16 length: 2)
    cursor.consume();
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 3); // 1 + 2

    // After consuming 'a' (UTF-16 length: 1)
    cursor.consume();
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.column, 4); // 3 + 1
}
