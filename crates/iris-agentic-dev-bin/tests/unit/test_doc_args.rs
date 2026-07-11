use clap::{Parser, Subcommand};
use iris_agentic_dev::cmd::doc::{DocAction, DocCommand};

#[derive(Parser)]
struct TestCli {
    #[command(subcommand)]
    cmd: TestDocCmd,
}

#[derive(Subcommand)]
enum TestDocCmd {
    Doc(DocCommand),
}

fn parse(args: &[&str]) -> DocCommand {
    let mut argv = vec!["test-cli", "doc"];
    argv.extend_from_slice(args);
    match TestCli::parse_from(argv).cmd {
        TestDocCmd::Doc(d) => d,
    }
}

#[test]
fn test_get_mode() {
    let cmd = parse(&["get", "MyApp.MyClass"]);
    assert!(matches!(cmd.action, DocAction::Get { .. }));
}

#[test]
fn test_get_captures_class_name() {
    let cmd = parse(&["get", "Config.MapMirrors"]);
    if let DocAction::Get { name } = cmd.action {
        assert_eq!(name, "Config.MapMirrors");
    } else {
        panic!("expected Get");
    }
}

#[test]
fn test_put_mode_with_file() {
    let cmd = parse(&["put", "MyApp.MyClass", "--file", "/tmp/MyClass.cls"]);
    assert!(matches!(cmd.action, DocAction::Put { .. }));
    if let DocAction::Put { name, file } = cmd.action {
        assert_eq!(name, "MyApp.MyClass");
        assert!(file.is_some());
    } else {
        panic!("expected Put");
    }
}

#[test]
fn test_put_mode_stdin_via_name() {
    // Stdin is triggered by passing "-" as the CLASSNAME positional arg
    let cmd = parse(&["put", "-"]);
    if let DocAction::Put { name, .. } = cmd.action {
        assert_eq!(name, "-");
    } else {
        panic!("expected Put");
    }
}

#[test]
fn test_get_with_namespace() {
    // --namespace is a top-level DocCommand flag; must come before the subcommand
    let cmd = parse(&["--namespace", "%SYS", "get", "Config.MapMirrors"]);
    assert_eq!(cmd.conn.namespace, "%SYS");
}

#[test]
fn test_put_file_path_preserved() {
    let cmd = parse(&["put", "X.cls", "--file", "/path/to/X.cls"]);
    if let DocAction::Put { file, .. } = cmd.action {
        let p = file.unwrap();
        assert_eq!(p.to_str().unwrap(), "/path/to/X.cls");
    } else {
        panic!("expected Put");
    }
}
