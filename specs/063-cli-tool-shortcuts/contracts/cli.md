# CLI Contracts: 063-cli-tool-shortcuts

## Subcommand: exec

```
iris-agentic-dev exec [OPTIONS] [CODE]
iris-agentic-dev exec [OPTIONS] -           # read from stdin
iris-agentic-dev exec [OPTIONS] --file PATH # read from file
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--namespace` | `-n` | String | IRIS namespace (overrides discovery) |
| `--container` | | String | Docker container name |
| `--host` | | String | IRIS web host |
| `--port` | | u16 | IRIS web port |
| `--username` | `-u` | String | IRIS username |
| `--password` | `-p` | String | IRIS password |
| `--file` | `-f` | PathBuf | Read code from file (mutually exclusive with inline code) |

**stdout**: raw IRIS output  
**stderr**: connection errors, argument errors  
**exit 0**: success  
**exit 1**: IRIS runtime error, connection failure, argument error  

---

## Subcommand: compile

```
iris-agentic-dev compile [OPTIONS] [FILES...]
iris-agentic-dev compile [OPTIONS]           # reads iris-dev.toml
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--namespace` | `-n` | String | IRIS namespace |
| *(connection flags)* | | | same as exec |

**stdout**: `OK: ClassName` or `ERROR: ClassName: <message>` per class  
**exit 0**: all classes compiled without errors  
**exit 1**: any compile error, connection failure  

---

## Subcommand: query

```
iris-agentic-dev query [OPTIONS] SQL
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--namespace` | `-n` | String | IRIS namespace |
| *(connection flags)* | | | same as exec |

**stdout**: TSV — header row + data rows, one per line, tab-separated  
**exit 0**: query executed (even if 0 rows)  
**exit 1**: SQL error, connection failure  

---

## Subcommand: doc

```
iris-agentic-dev doc get [OPTIONS] CLASSNAME
iris-agentic-dev doc put [OPTIONS] CLASSNAME --file PATH
iris-agentic-dev doc put [OPTIONS] CLASSNAME -     # read from stdin
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--namespace` | `-n` | String | IRIS namespace |
| `--file` | `-f` | PathBuf | Source file for `put` |
| *(connection flags)* | | | same as exec |

**stdout** (get): raw UDL class source  
**exit 0**: class found (get) / written (put)  
**exit 1**: class not found, write error, connection failure  

---

## Subcommand: tool

```
iris-agentic-dev tool [OPTIONS] TOOL_NAME
iris-agentic-dev tool [OPTIONS] TOOL_NAME --args JSON
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--args` | `-a` | String | JSON object of tool arguments, default `{}` |
| `--namespace` | `-n` | String | IRIS namespace |
| *(connection flags)* | | | same as exec |

**stdout**: raw tool result text  
**stderr**: unknown tool name + list of valid names  
**exit 0**: tool executed successfully  
**exit 1**: unknown tool name, tool error, connection failure  

---

## Shared Connection Flag Precedence

1. Explicit CLI flags (`--host`, `--port`, `--namespace`, etc.)
2. `.iris-dev.toml` workspace config
3. Environment variables (`IRIS_HOST`, `IRIS_WEB_PORT`, `IRIS_CONTAINER`, etc.)
4. Auto-discovery cascade (localhost scan → Docker scan → VS Code settings)

---

## Error Output Format (stderr)

```
error: <message>
```

For unknown tool names:
```
error: unknown tool 'foo'
available tools:
  check_config
  docs_introspect
  iris_compile
  iris_doc
  iris_execute
  iris_global
  iris_info
  iris_query
  ... (full list)
```
