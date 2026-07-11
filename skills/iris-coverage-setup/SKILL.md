---
name: iris-coverage-setup
description: Enable the IRIS line-by-line monitor (bbsiz CPF parameter). Run once per container before measuring ObjectScript test coverage. Verifies $zu(84) is functional and diagnoses build incompatibility.
---

# /iris-coverage-setup — Enable IRIS Line-by-Line Monitor

Configure the IRIS instance so `%Monitor.System.LineByLine` can collect execution counts.
Required before running `/iris-coverage-run`. **Needs an IRIS restart.**

## Background

`%Monitor.System.LineByLine` wraps `$zu(84,...)` kernel calls. The shared memory buffer
for line counting is controlled by `bbsiz` in `iris.cpf [config]`. It defaults to `-1`
(disabled). Without it set, `Start()` appears to succeed but collects nothing.

**Build compatibility:**

| Build                   | Works? | Notes                                    |
| ----------------------- | ------ | ---------------------------------------- |
| AI builds (2026.x.0AI)  | ✓      | Standard kernel                          |
| SQLT.145+ / sqlt146     | ✓      | Standard kernel                          |
| TBLP 127 (2026.3.0TBLP) | ✗      | `$zu(84)` not wired — `<FUNCTION>` error |

## Step 1 — Check if monitor is already functional

```objectscript
// In any namespace — if this returns 1, skip to Step 3
write $zu(84,0,1,1,1,1,1,1), !   // returns 1 = OK, throws <FUNCTION> = not available
do $zu(84,0,0)                    // stop/cleanup after check
```

If it throws `<FUNCTION>`: this IRIS build does not support `$zu(84)`. Coverage measurement
is not possible. Stop here and document the gap.

## Step 2 — Set bbsiz and restart

Run in `%SYS` namespace (or via MCP tool with namespace="%SYS"):

```objectscript
set cfg = ##class(Config.config).Open()
write "Current bbsiz: ", cfg.bbsiz, !   // expect -1
set cfg.bbsiz = 4096                     // 4 MB buffer
set sc = cfg.%Save()
write $system.Status.GetErrorText(sc), !
```

Verify it wrote to `iris.cpf`:

```bash
grep bbsiz /usr/irissys/iris.cpf   # expect: bbsiz=4096
```

**IMPORTANT: Do NOT add `MonitorEnabled=1` to iris.cpf — that key is invalid and crashes
IRIS on startup with "Invalid parameter name 'MonitorEnabled'".**

Restart IRIS:

```bash
docker restart <container-name>
```

## Step 3 — Verify monitor is functional after restart

```objectscript
// Verify $zu(84) subsystem is live
write $zu(84,0,1,1,1,1,1,1), !   // must return 1

// Verify Start/Stop cycle works
set sc = ##class(%Monitor.System.LineByLine).Start($lb("MyClass.1"), "", "")
write $system.Status.GetErrorText(sc), !   // expect empty = OK
do ##class(%Monitor.System.LineByLine).Stop()
```

If `Start()` returns `ERROR #6060: Somebody else is using the Monitor` — another process has
the monitor. Stop it:

```objectscript
do ##class(%Monitor.System.LineByLine).Stop()
```

## Step 4 — Verify result query returns data

```objectscript
set sc = ##class(%Monitor.System.LineByLine).Start($lb("MyClass.1"), "", "")
// ... run some code that calls MyClass methods ...
do ##class(%Monitor.System.LineByLine).Stop()

set rset = ##class(%ResultSet).%New("%Monitor.System.LineByLine:Result")
do rset.Execute("MyClass.1")
set n = 0
while rset.Next() { set n = n + 1 }
write "Lines returned: ", n, !   // expect > 0 if MyClass was exercised
```

If `n = 0` despite code running — the monitor buffer allocated but `$zu(84,0)` returns
84 (state value, not error) and no counts were captured. This indicates the build doesn't
fully support in-process line counting even with `bbsiz` set. Use dpgenai1 or an AI build.

## Notes

- `bbsiz` is in `[config]` section, not `[Monitor]` section (which has no standard parameters)
- `$zu(84,0)` returning 84 is NOT an error code — it's the current state bitmask
- Only one process can hold the monitor at a time — always `Stop()` before `Start()`
- Results must be queried in the **same IRIS process** that called `Stop()`
- INT routine name = `ClassName.1` (not `.cls`, not `ClassName.INT`)
