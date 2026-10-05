use super::*;

fn marked(source: &str) -> Vec<usize> {
    heredoc_lines(source)
        .iter()
        .enumerate()
        .filter_map(|(idx, in_heredoc)| in_heredoc.then_some(idx))
        .collect()
}

#[test]
fn marks_squiggly_body_and_terminator() {
    let src = "sql = <<~SQL\n  #{table} (\n  id int\n  SQL\nfoo\n";
    assert_eq!(marked(src), vec![1, 2, 3]);
}

#[test]
fn bare_heredoc_needs_terminator_at_column_zero() {
    let src = "x = <<EOS\n  EOS\nEOS\n#code\n";
    assert_eq!(marked(src), vec![1, 2]);
}

#[test]
fn terminator_with_trailing_whitespace_does_not_close() {
    let src = "x = <<~E\n  a\n  E  \n  E\n";
    assert_eq!(marked(src), vec![1, 2, 3]);
}

#[test]
fn quoted_identifiers_open_heredocs() {
    let src = "a = <<~'RAW'\n#{x}\nRAW\nb = <<-\"Q\"\n#{y}\n  Q\n";
    assert_eq!(marked(src), vec![1, 2, 4, 5]);
}

#[test]
fn stacked_heredocs_close_in_order() {
    let src = "foo(<<~A, <<~B)\n  one\nA\n  two\nB\nbar\n";
    assert_eq!(marked(src), vec![1, 2, 3, 4]);
}

#[test]
fn shift_and_singleton_class_are_not_heredocs() {
    let src = "a<<b\nlist << ITEM\nclass << self\nITEM\nb\n";
    assert!(marked(src).is_empty());
}

#[test]
fn ignores_openers_in_strings_and_comments() {
    let src = "x = \"<<~SQL\"\n# see <<~SQL\nSQL\n";
    assert!(marked(src).is_empty());
}

#[test]
fn unterminated_opener_marks_nothing() {
    let src = "x = <<~NOPE\n#{a}\nend\n";
    assert!(marked(src).is_empty());
}

#[test]
fn handles_crlf() {
    let src = "x = <<~SQL\r\n#{a}\r\nSQL\r\ny\r\n";
    assert_eq!(marked(src), vec![1, 2]);
}
