# Data Model: CLI Tool Shortcuts (063)

No persistent data model. This feature is stateless — each CLI invocation opens a connection, performs one operation, prints output, and exits. All state lives in the existing IRIS instance.

## CLI Input Entities

### ConnectionArgs (shared across all subcommands)

Clap `Args` group reused by all subcommands.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `--namespace` / `-n` | String | from toml or discovery | IRIS namespace |
| `--container` | String | `IRIS_CONTAINER` env | Named Docker container |
| `--host` | String | from discovery | IRIS web host |
| `--port` | u16 | from discovery | IRIS web port |
| `--username` | String | `_SYSTEM` | IRIS username |
| `--password` | String | `SYS` | IRIS password |

### ExecArgs

| Field | Type | Notes |
|-------|------|-------|
| `code` (positional) | Option\<String\> | Inline ObjectScript; mutually exclusive with `--file` and `-` |
| `--file` | Option\<PathBuf\> | Read code from file |
| `--namespace` | via ConnectionArgs | |

`-` as the positional `code` value triggers stdin read.

### CompileArgs

| Field | Type | Notes |
|-------|------|-------|
| `files` (positional, 0..) | Vec\<PathBuf\> | .cls files to compile; empty = read iris-dev.toml |
| `--namespace` | via ConnectionArgs | |

### QueryArgs

| Field | Type | Notes |
|-------|------|-------|
| `sql` (positional) | String | SQL statement |
| `--namespace` | via ConnectionArgs | |

### DocArgs

| Field | Type | Notes |
|-------|------|-------|
| `action` (subcommand) | `get` \| `put` | |
| `class` (positional) | String | Class name (e.g. `Config.MapMirrors`) |
| `--file` | Option\<PathBuf\> | For `put`: source file (stdin `-` also accepted) |
| `--namespace` | via ConnectionArgs | |

### ToolArgs

| Field | Type | Notes |
|-------|------|-------|
| `name` (positional) | String | Exact MCP tool name (e.g. `iris_info`) |
| `--args` | String | JSON object string, default `{}` |
| `--namespace` | via ConnectionArgs | |

## Output Shapes

### exec
```
<raw IRIS stdout>
```
Exit 0 on success, non-zero on IRIS error or connection failure.

### compile
```
OK: IrisDevTmp.MyClass
ERROR: IrisDevTmp.BadClass: Syntax error at line 5
```
Exit 0 only if all classes compiled without errors.

### query
```
Name\tAge\tCity
Alice\t30\tBoston
Bob\t25\tNYC
```
Tab-separated, header row first. No padding, no box-drawing. Exit 0 even if 0 rows.

### doc get
```
Class Config.MapMirrors [ ... ]
{
...
}
```
Raw UDL source as returned by Atelier. Exit 0 if class found, non-zero if not found.

### tool
```
<raw tool result text>
```
Same text as `CallToolResult.content[0].text` from MCP. Exit 0 on success, non-zero on tool error.
