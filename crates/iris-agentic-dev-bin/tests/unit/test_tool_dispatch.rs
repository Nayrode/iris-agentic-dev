use iris_agentic_dev::cmd::tool::{dispatch_map_keys, TOOL_NAMES};
use iris_agentic_dev_core::tools::{IrisTools, Toolset};

#[test]
fn test_dispatch_map_keys_match_registered_tool_names() {
    // Build the Merged toolset (same as MCP server uses) and compare names
    let tools = IrisTools::new_with_toolset(None, Toolset::Merged).unwrap();
    let registered = tools.registered_tool_names();
    let dispatch = dispatch_map_keys();

    let mut missing: Vec<String> = registered
        .iter()
        .filter(|n| !dispatch.contains(n.as_str()))
        .cloned()
        .collect();
    missing.sort();
    let mut extra: Vec<&str> = dispatch
        .iter()
        .copied()
        .filter(|n| !registered.contains(*n))
        .collect();
    extra.sort();

    assert!(
        missing.is_empty(),
        "tools in registered_tool_names() but not in dispatch map: {:?}",
        missing
    );
    assert!(
        extra.is_empty(),
        "tools in dispatch map but not in registered_tool_names(): {:?}",
        extra
    );
}

#[test]
fn test_tool_names_sorted() {
    let sorted: Vec<&str> = {
        let mut v = TOOL_NAMES.to_vec();
        v.sort_unstable();
        v
    };
    assert_eq!(TOOL_NAMES.to_vec(), sorted, "TOOL_NAMES must be sorted");
}

#[test]
fn test_unknown_tool_is_rejected() {
    assert!(
        !dispatch_map_keys().contains("nonexistent_tool_xyz"),
        "unknown tool should not be in dispatch map"
    );
}
