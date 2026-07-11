# Spec 064 — ObjectScript Line Coverage Tool

## Summary

Add two tools to iris-agentic-dev: `iris_coverage_start` / `iris_coverage_stop` + `iris_coverage_report`.
These wrap `%Monitor.System.LineByLine` to measure which executable lines of a target class list were
executed during a test run, and return a structured per-class and total coverage report.

## Background / Hard-Won Findings

This spec captures lessons from fhir-017 (2026-07-11) trying to get line coverage on TBLP 127.

### How IRIS line-by-line monitoring works

- API: `##class(%Monitor.System.LineByLine).Start(routineList, metrics, processList)`
- `routineList` is a `%List` of **INT routine names** (compiled form: `ClassName.1`, not `.cls`)
- `metrics` empty string = default (RtnLine count + time)
- `processList` empty string = all processes
- `Stop()` ends collection
- Results via `##class(%ResultSet).%New("%Monitor.System.LineByLine:Result")` → `Execute(routineName)`
  → each row is a `%List(lineNum, execCount, clockTime, totalTime)`
  → `execCount = -1` means non-executable line (comment/blank/label-only) — skip these
  → `execCount >= 0` means executable; `> 0` means hit

### The `$zu(84)` subsystem

- `%Monitor.System.LineByLine` wraps `$zu(84,...)` kernel calls
- `$zu(84,0)` returns **current collection state**, NOT an error code — value 84 is normal on some builds
- Key call: `$zu(84,0,1,1,1,1,1,1)` must return `1` to allocate monitor shared memory
- If it throws `<FUNCTION>`, the `$zu(84)` subsystem is **not implemented** in this kernel build

### The `bbsiz` CPF parameter

- Lives in `[config]` section of `iris.cpf`
- Controls the line-by-line monitor shared memory buffer size (KB)
- Default: `-1` (disabled — no buffer allocated, `$zu(84)` memory calls fail)
- Correct value: `4096` (4 MB) or larger
- Set via: `##class(Config.config).Open()` → `set cfg.bbsiz = 4096` → `cfg.%Save()` → restart IRIS
- **Requires IRIS restart to take effect** (shared memory allocated at startup)
- The CPF key is `bbsiz` (NOT `MonitorEnabled` — that key is invalid and crashes IRIS on startup)

### Build compatibility

| Build                   | `$zu(84)` status   | Notes                                          |
| ----------------------- | ------------------ | ---------------------------------------------- |
| TBLP 127 (2026.3.0TBLP) | `<FUNCTION>` error | Monitor subsystem not wired in this branch cut |
| SQLT.145+ / sqlt146     | Expected to work   | Standard AI/release builds have it             |
| AI builds (2026.x.0AI)  | Expected to work   | Standard kernel                                |

### Correct `bbsiz` setup sequence (for builds where it works)

```objectscript
// In %SYS namespace
set cfg = ##class(Config.config).Open()
write cfg.bbsiz  // should be -1 (disabled)
set cfg.bbsiz = 4096
do cfg.%Save()
// Then restart IRIS — bbsiz takes effect at startup only
```

After restart, verify:

```objectscript
write $zu(84,0,1,1,1,1,1,1)  // must return 1, not throw <FUNCTION>
```

### Running coverage

```objectscript
// 1. Stop any leftover session
do ##class(%Monitor.System.LineByLine).Stop()

// 2. Build routine list (INT names = ClassName.1)
set routines = $lb("MyApp.MyClass.1", "MyApp.OtherClass.1")
set sc = ##class(%Monitor.System.LineByLine).Start(routines, "", "")

// 3. Run tests
do ##class(%UnitTest.Manager).RunTest("MyApp/Tests", "/nodelete")

// 4. Stop
do ##class(%Monitor.System.LineByLine).Stop()

// 5. Collect results
set rset = ##class(%ResultSet).%New("%Monitor.System.LineByLine:Result")
do rset.Execute("MyApp.MyClass.1")
set total = 0, hit = 0
while rset.Next() {
    set data = rset.GetData(1)
    set execCount = $listget(data, 2)
    if execCount < 0 { continue }   // non-executable line
    set total = total + 1
    if execCount > 0 { set hit = hit + 1 }
}
write hit _ "/" _ total _ " = " _ $fnumber(hit/total*100,"",1) _ "%", !
```

## Proposed Tools

### `iris_coverage` (single tool, mode-based)

```text
iris_coverage(
    mode: "start" | "stop" | "report" | "run",
    classes: ["MyApp.MyClass", ...],  // product class names (without .1)
    test_path: "MyApp/Tests",          // for mode=run only
    namespace: "MYNS"
)
```

**mode=start**: Stops any existing session, calls `Start()` with class list converted to `.1` routines.
Returns `{started: true, routines: [...], bbsiz_ok: bool}`.

**mode=stop**: Calls `Stop()`. Returns `{stopped: true}`.

**mode=report**: Queries `Result` for each routine. Returns per-class and total coverage:

```json
{
  "total_pct": 73.4,
  "classes": [
    {"class": "MyApp.MyClass", "hit": 45, "total": 61, "pct": 73.8},
    ...
  ],
  "meets_target": false,
  "target_pct": 90.0
}
```

**mode=run**: start + RunTest + stop + report in one call.

### `iris_coverage_check`

Pre-flight check: verifies `bbsiz` is set, `$zu(84,0,1,1,1,1,1,1)` returns 1, no other monitor session
active. Returns actionable error if setup is wrong, including the exact CPF fix needed.

## Implementation Notes

- Tools live in a new `coverage.rs` in `crates/iris-agentic-dev-core/src/tools/`
- Use `iris_execute` internally (docker exec path) since Atelier doesn't support streaming
- The `Result` query must be issued in the same IRIS process that called `Stop()` — use a single
  atomic ObjectScript execution that starts, runs tests, stops, and collects results
- Non-executable lines (`execCount = -1`) must be excluded from denominator
- Class names → routine names: append `.1` (handles single inheritance; doesn't need to walk
  superclasses since we only want coverage of the specific modified class)

## Acceptance Criteria

1. `iris_coverage(mode="run", classes=[...], test_path=...)` returns structured JSON with per-class
   line coverage percentages
2. Returns `bbsiz_not_configured` error with fix instructions if `$zu(84)` subsystem not available
3. Handles "monitor already running" gracefully (stop + restart)
4. Works on AI builds and SQLT.145+; returns clear build-incompatibility error on TBLP 127
