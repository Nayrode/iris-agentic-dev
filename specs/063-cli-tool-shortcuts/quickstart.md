# Quickstart: CLI Tool Shortcuts (063)

## Execute ObjectScript

```bash
# Inline code
iris-agentic-dev exec 'write $ZVersion,!'

# From a file
iris-agentic-dev exec --file my-script.os

# From stdin
echo 'write $ZVersion,!' | iris-agentic-dev exec -

# Macros work
iris-agentic-dev exec 'write $$$OK,!'

# Different namespace
iris-agentic-dev exec --namespace %SYS 'write $ZVersion,!'

# Specific container
iris-agentic-dev exec --container iris-dev-iris 'write $ZVersion,!'
```

## Compile Classes

```bash
# Compile specific files
iris-agentic-dev compile MyApp/MyClass.cls MyApp/OtherClass.cls

# Compile everything in iris-dev.toml (existing behavior)
iris-agentic-dev compile

# CI usage — exit code signals success/failure
iris-agentic-dev compile MyApp/MyClass.cls && echo "Build OK"
```

## Run SQL Queries

```bash
# Basic query
iris-agentic-dev query 'SELECT Name FROM %Dictionary.ClassDefinition ORDER BY Name'

# Pipe to grep
iris-agentic-dev query 'SELECT Name FROM %Dictionary.ClassDefinition' | grep Config

# In %SYS namespace
iris-agentic-dev query --namespace %SYS \
  'SELECT Name FROM %Dictionary.ClassDefinition WHERE Name %STARTSWITH "Config.Map"'

# Pipe to awk for column extraction
iris-agentic-dev query 'SELECT Name, Super FROM %Dictionary.ClassDefinition' \
  | awk -F'\t' 'NR>1 {print $1}'
```

## Read/Write Class Documents

```bash
# Read a class source
iris-agentic-dev doc get Config.MapMirrors --namespace %SYS

# Save to file
iris-agentic-dev doc get Config.MapMirrors --namespace %SYS > Config.MapMirrors.cls

# Write a class from file
iris-agentic-dev doc put MyApp.MyClass --file MyApp/MyClass.cls

# Write from stdin
cat MyClass.cls | iris-agentic-dev doc put MyApp.MyClass -
```

## Invoke Any Tool by Name

```bash
# No args
iris-agentic-dev tool iris_info

# With args
iris-agentic-dev tool iris_execute --args '{"code":"write $ZVersion,!"}'
iris-agentic-dev tool iris_query --args '{"query":"SELECT Name FROM %Dictionary.ClassDefinition"}'

# Discover available tool names
iris-agentic-dev tool nonexistent_tool  # prints full list to stderr
```

## Connection Override Examples

All subcommands accept the same connection flags:

```bash
# Remote IRIS instance (no Docker)
iris-agentic-dev exec \
  --host myserver.example.com \
  --web-port 52773 \
  --username SuperUser \
  --password secret \
  --namespace MYAPP \
  'write $ZVersion,!'

# Specific Docker container
iris-agentic-dev exec --container opsreview-iris 'write $ZVersion,!'
```

---

## Implementation Deviations from Plan

### `--port` renamed to `--web-port`

The plan used `--port` as the connection flag. The implementation uses `--web-port` (short: `-p`)
to match the existing `mcp`/`compile` subcommand's `ConnectionArgs` convention, which maps to
the `IRIS_WEB_PORT` environment variable. All connection-flag examples above use `--web-port`.

### `iris_info` requires `--args '{"what":"version"}'`

The quickstart shows `iris-agentic-dev tool iris_info` with no args. In practice `iris_info`
requires a `what` field. Use `iris-agentic-dev tool iris_info --args '{"what":"version"}'`
or `iris-agentic-dev tool check_config --args '{}'` for a simpler config check.

### `doc put` stdin: positional `CLASSNAME` is `-`, not `--stdin` flag

The plan proposed a `--stdin` flag. The implementation uses clap's `allow_hyphen_values = true`
on the positional `CLASSNAME` arg: pass `-` as the class name to read content from stdin.
This matches conventional Unix tool behavior (`cat file | cmd -` patterns).

### Atelier `query` response: row-objects not column-descriptors

The spec assumed Atelier returns `result.columns` (descriptors) + `result.rows` (arrays).
The actual response is `result.content` as an array of row-objects
(`[{"Name": "val"}, ...]`). `extract_columns` and `extract_rows` in `tsv.rs` were rewritten
to handle the real format. Zero-row results return an empty `content` array with no column
metadata — the `query` subcommand exits 0 with no output in this case (no header possible).

### IRIS runtime errors exit 0 (not non-zero)

The plan stated runtime errors should cause non-zero exit. The HTTP generator API returns
HTTP 200 with error text in the response body. The `exec` subcommand exits 0; callers must
inspect stdout for `ERROR:` prefix. Only compile errors and HTTP failures produce non-zero exits.
