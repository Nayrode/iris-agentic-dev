use clap::Parser;
use iris_agentic_dev::cmd::compile::CompileCommand;

#[derive(Parser)]
struct TestCli {
    #[command(flatten)]
    cmd: CompileCommand,
}

fn parse(args: &[&str]) -> CompileCommand {
    let mut argv = vec!["test-cli"];
    argv.extend_from_slice(args);
    TestCli::parse_from(argv).cmd
}

#[test]
fn test_no_args_is_toml_mode() {
    let cmd = parse(&[]);
    assert!(cmd.files.is_empty(), "no files = toml mode");
}

#[test]
fn test_single_file_arg() {
    let cmd = parse(&["MyApp.MyClass.cls"]);
    assert_eq!(cmd.files.len(), 1);
    assert_eq!(cmd.files[0].to_str().unwrap(), "MyApp.MyClass.cls");
}

#[test]
fn test_multiple_file_args() {
    let cmd = parse(&["A.cls", "B.cls", "C.cls"]);
    assert_eq!(cmd.files.len(), 3);
}

#[test]
fn test_namespace_override_with_files() {
    let cmd = parse(&["--namespace", "APP", "MyClass.cls"]);
    assert_eq!(cmd.conn.namespace, "APP");
    assert_eq!(cmd.files.len(), 1);
}

#[test]
fn test_default_flags_are_cuk() {
    let cmd = parse(&[]);
    assert_eq!(cmd.flags, "cuk");
}

#[test]
fn test_custom_flags() {
    let cmd = parse(&["--flags", "ck"]);
    assert_eq!(cmd.flags, "ck");
}
