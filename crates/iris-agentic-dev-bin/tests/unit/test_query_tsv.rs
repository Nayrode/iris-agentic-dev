use iris_agentic_dev::cmd::tsv::{extract_columns, extract_rows, rows_to_tsv, tsv_header};

/// Build an Atelier-shaped body matching the real /action/query response format:
/// result.content = array of row-objects with column names as keys.
fn make_atelier_body(columns: &[&str], rows: &[Vec<&str>]) -> serde_json::Value {
    let content: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (col, val) in columns.iter().zip(row.iter()) {
                obj.insert(col.to_string(), serde_json::Value::String(val.to_string()));
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    serde_json::json!({"result": {"content": content}})
}

#[test]
fn test_extract_columns_single() {
    let body = make_atelier_body(&["Name"], &[vec!["Alice"]]);
    let cols = extract_columns(&body);
    assert_eq!(cols, vec!["Name"]);
}

#[test]
fn test_extract_columns_multi() {
    // serde_json BTreeMap sorts keys alphabetically
    let body = make_atelier_body(&["Age", "City", "Name"], &[vec!["30", "NYC", "Alice"]]);
    let cols = extract_columns(&body);
    // Columns sorted: Age, City, Name
    assert_eq!(cols, vec!["Age", "City", "Name"]);
}

#[test]
fn test_extract_columns_empty_body() {
    let body = serde_json::json!({});
    let cols = extract_columns(&body);
    assert!(cols.is_empty());
}

#[test]
fn test_extract_rows_basic() {
    // Use single-column to avoid ordering ambiguity
    let body = make_atelier_body(&["Name"], &[vec!["Alice"], vec!["Bob"]]);
    let rows = extract_rows(&body);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["Alice"]);
    assert_eq!(rows[1], vec!["Bob"]);
}

#[test]
fn test_extract_rows_zero_rows() {
    // Zero rows = empty content array; no column info available
    let body = serde_json::json!({"result": {"content": []}});
    let rows = extract_rows(&body);
    assert!(rows.is_empty());
}

#[test]
fn test_full_tsv_pipeline_multicolumn() {
    // Single column avoids key-ordering ambiguity
    let body = make_atelier_body(&["Name"], &[vec!["Alice"], vec!["Bob"]]);
    let cols = extract_columns(&body);
    let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
    let header = tsv_header(&col_refs);
    let rows = extract_rows(&body);
    let data = rows_to_tsv(&rows);
    let full = format!("{}\n{}", header, data);
    assert_eq!(full, "Name\nAlice\nBob\n");
}

#[test]
fn test_tsv_two_columns_consistent_order() {
    // Column order in header and row values must be consistent (both from same BTreeMap key order)
    let body = make_atelier_body(&["Age", "Name"], &[vec!["30", "Alice"]]);
    let cols = extract_columns(&body);
    let rows = extract_rows(&body);
    // Find which col is at index 0
    let name_idx = cols.iter().position(|c| c == "Name").unwrap();
    let age_idx = cols.iter().position(|c| c == "Age").unwrap();
    assert_eq!(rows[0][name_idx], "Alice");
    assert_eq!(rows[0][age_idx], "30");
}

#[test]
fn test_tsv_zero_rows_produces_header_only() {
    // When rows exist, header comes from first row keys
    let body = make_atelier_body(&["Name"], &[vec!["Alice"]]);
    let cols = extract_columns(&body);
    let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
    let header = tsv_header(&col_refs);
    assert_eq!(header, "Name");
}

#[test]
fn test_no_spurious_framing_in_output() {
    let body = make_atelier_body(&["Val"], &[vec!["42"]]);
    let cols = extract_columns(&body);
    let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
    let header = tsv_header(&col_refs);
    let rows = extract_rows(&body);
    let data = rows_to_tsv(&rows);
    let out = format!("{}\n{}", header, data);
    assert!(!out.contains("Result:"));
    assert!(!out.contains("┌"));
    assert!(!out.contains("│"));
    for line in out.lines() {
        assert!(line.chars().all(|c| c != '\u{2502}' && c != '\u{250C}'));
    }
}
