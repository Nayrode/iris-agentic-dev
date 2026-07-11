use iris_agentic_dev::cmd::tsv::{rows_to_tsv, tsv_header};

#[test]
fn test_header_single_column() {
    let h = tsv_header(&["Name"]);
    assert_eq!(h, "Name");
}

#[test]
fn test_header_multi_column() {
    let h = tsv_header(&["Name", "Age", "City"]);
    assert_eq!(h, "Name\tAge\tCity");
}

#[test]
fn test_header_empty() {
    let h = tsv_header(&[]);
    assert_eq!(h, "");
}

#[test]
fn test_rows_to_tsv_single_column() {
    let rows = vec![vec!["Alice".to_string()], vec!["Bob".to_string()]];
    let out = rows_to_tsv(&rows);
    assert_eq!(out, "Alice\nBob\n");
}

#[test]
fn test_rows_to_tsv_multi_column() {
    let rows = vec![
        vec!["Alice".to_string(), "30".to_string(), "Boston".to_string()],
        vec!["Bob".to_string(), "25".to_string(), "NYC".to_string()],
    ];
    let out = rows_to_tsv(&rows);
    assert_eq!(out, "Alice\t30\tBoston\nBob\t25\tNYC\n");
}

#[test]
fn test_rows_to_tsv_zero_rows() {
    let rows: Vec<Vec<String>> = vec![];
    let out = rows_to_tsv(&rows);
    assert_eq!(out, "");
}

#[test]
fn test_rows_to_tsv_embedded_tab_escaped() {
    // An embedded tab in a cell value must be escaped as literal \t
    let rows = vec![vec!["hello\tworld".to_string()]];
    let out = rows_to_tsv(&rows);
    assert_eq!(out, "hello\\tworld\n");
}

#[test]
fn test_rows_to_tsv_embedded_newline_escaped() {
    let rows = vec![vec!["line1\nline2".to_string()]];
    let out = rows_to_tsv(&rows);
    assert_eq!(out, "line1\\nline2\n");
}

#[test]
fn test_rows_to_tsv_no_padding_or_alignment() {
    // Values of different lengths should not be padded
    let rows = vec![
        vec!["short".to_string(), "a much longer value".to_string()],
        vec!["x".to_string(), "y".to_string()],
    ];
    let out = rows_to_tsv(&rows);
    // No extra spaces around values
    assert_eq!(out, "short\ta much longer value\nx\ty\n");
}

#[test]
fn test_tsv_header_no_tabs_in_column_names() {
    // Column name with tab — escape it
    let h = tsv_header(&["Col\tName"]);
    assert_eq!(h, "Col\\tName");
}
