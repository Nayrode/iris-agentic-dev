# Troubleshooting

---

## Symptom table

| Symptom                                      | Likely cause                                                     | Fix                                                                                   |
| -------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 404 on `/api/atelier` (Windows)              | IIS missing `/api` web application                               | See [Windows IIS setup](connecting.md#windows-iis-api-web-application-required)       |
| `check_config` works but compile/search fail | Atelier web app `Recurse=0`                                      | Management Portal → Security → Web Apps → `/api/atelier` → enable **Recurse**         |
| All tools fail, namespace listing works      | API version mismatch                                             | Verify IRIS supports Atelier v8 (`iris-agentic-dev --verbose` shows detected version) |
| 403 on write operations                      | Insufficient permissions                                         | Use a user with `%DB_USER` or `%All` role                                             |
| Connection delays on Windows                 | `localhost` DNS issue                                            | Use `host = "127.0.0.1"` in `.iris-agentic-dev.toml`                                  |
| `SERVER_MANAGER_CREDENTIAL_ERROR`            | Credential not in OS keychain                                    | VS Code → Server Manager → right-click server → **Reconnect**                         |
| `SERVER_MANAGER_AMBIGUOUS`                   | Multiple SM servers, no `IRIS_SERVER_NAME`                       | Set `IRIS_SERVER_NAME=<server-key>` (see `check_config` for available names)          |
| `STALE_CONTENT` from `iris_doc`              | `expected` text doesn't match current file                       | Re-fetch the document (`mode=get`) and retry with current content                     |
| `SCOPE_REQUIRED` from `iris_search`          | Search called with no document scope                             | Pass at least one category or document type in `scope`                                |
| `CODE_EDIT_BLOCKED`                          | Attempted write to `%Dictionary`, `$SYSTEM.OBJ`, or code globals | Use `iris_doc` (put) + `iris_compile` instead                                         |
| `CHECKIN_BLOCKED` from `iris_source_control` | CheckIn disabled by default                                      | Set `IRIS_SCM_ALLOW_CHECKIN=1` to enable                                              |
| `HTTP_EXECUTION_FAILED` from `iris_execute`  | Atelier execution failed and no Docker fallback                  | Verify Atelier endpoint reachable; set `IRIS_CONTAINER` for Docker fallback           |
| `IRIS_UNREACHABLE`                           | No IRIS connection discoverable                                  | Run `check_config` to see discovery state; check host/port/credentials                |

---

## Verbose HTTP logging

```bash
iris-agentic-dev mcp --verbose 2>debug.log
```

A 404 on `/api/atelier/v8/...` usually indicates the `Recurse` setting or a missing `/api`
web application. A 401/403 is an authentication issue. Connection refused means the host or
port is wrong.

---

## Connection and config verification

Run `check_config` from your AI assistant:

```text
Call check_config and show me the result.
```

Or from the terminal:

```bash
iris-agentic-dev tool check_config --args '{}'
```

The output shows active connection state, which discovery source won, and the status of
each optional feature (Server Manager, containers, write gates).

---

## CLI commands

```bash
iris-agentic-dev mcp                     # Start the MCP server
iris-agentic-dev compile MyApp.Foo.cls   # Compile from the terminal
iris-agentic-dev init                    # Generate .iris-agentic-dev.toml from running containers
iris-agentic-dev install                 # Install packages from iris-dev.toml
iris-agentic-dev benchmark --skill <path> --baseline   # Run the skill benchmark harness
iris-agentic-dev --version               # Print version
```

### Shortcut subcommands

Run any IRIS operation directly from the terminal — no MCP client or AI session needed.

| Subcommand | Example                                                                 | What it does                                                               |
| ---------- | ----------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `exec`     | `iris-agentic-dev exec 'write $ZVersion,!'`                             | Execute ObjectScript inline, from `--file`, or from stdin (`-`)            |
| `compile`  | `iris-agentic-dev compile MyApp.Foo.cls`                                | Compile one or more `.cls`/`.mac` files; prints `OK:` or `ERROR:` per file |
| `query`    | `iris-agentic-dev query 'SELECT Name FROM %Dictionary.ClassDefinition'` | Execute SQL; prints TSV (header + rows) to stdout                          |
| `doc`      | `iris-agentic-dev doc get MyApp.Foo`                                    | Read IRIS document UDL; `doc put MyApp.Foo --file f.cls` to write          |
| `tool`     | `iris-agentic-dev tool iris_info --args '{"what":"version"}'`           | Call any MCP tool by name without an MCP client                            |

All shortcuts accept: `--host`, `--web-port`, `--namespace`, `--username`, `--password`,
`--container`. Env vars (`IRIS_HOST`, `IRIS_WEB_PORT`, etc.) are also honored.

```bash
# Print IRIS version
iris-agentic-dev exec 'write $ZVersion,!'

# Execute a file
iris-agentic-dev exec --file myscript.cos

# Pipe script via stdin
echo 'write $namespace,!' | iris-agentic-dev exec -

# Compile with explicit connection
iris-agentic-dev compile MyApp.MyClass.cls --host myserver --namespace PROD

# Query a different namespace
iris-agentic-dev query --namespace %SYS 'SELECT Name FROM Security.Users'

# Read a class definition
iris-agentic-dev doc get %Dictionary.ClassDefinition --namespace %SYS

# Upload a class
iris-agentic-dev doc put MyApp.Foo --file MyApp.Foo.cls

# Call any tool
iris-agentic-dev tool check_config --args '{}'
```

---

## Getting help

Issues and pull requests: [GitHub Issues](https://github.com/intersystems-community/iris-agentic-dev/issues)

Questions: [thomas.dyar@intersystems.com](mailto:thomas.dyar@intersystems.com)
