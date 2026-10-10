# Experiment: D013 transferred failure state and terminal cleanup

Investigation date: 2026-10-07 (Europe/Berlin).

Question: Does the existing wrapper core preserve dispatch eligibility, exact
errors, dependency ownership and terminal cleanup across serialized thread
transfers, including failure and Rust unwinding?

Scope: controlled, locally compiled fixtures; no AMD DLL, provider or GPU work.
This experiment does not accept D013 or implement a public movable family.

## Predeclared matrix and oracle

This matrix was written before the first experiment execution. Every new owner
scenario runs in an isolated test child with a 60-second watchdog and an exact
completion marker. Each has a control and transfer variant. A missing marker,
unexpected panic/exit, timeout, wrong event/thread/code/count, pointer change,
retry or premature dependency/library release is a failure. Expected Rust
panics must match the declared payload and end with a successful child exit.
Fixture exports, callbacks and dependency destructors must not unwind.

`create`, `inspect`, `dispatch`, `destroy`, `state-release`, `device-release`
are denoted Cr/I/P/D/S/V below. For transfer cases, loading/creation occur on
named A, rejected/native dispatch on B, and terminal cleanup on C. Joins order
exclusive moves. Controls use the child test thread for these actions.
The runtime's external owner is dropped immediately after creation.

| Cases | Expected ordered events and outcomes |
|---|---|
| known-poison / unknown-poison, control + transfer | Cr(A), I(B), P(B), D(C), S(C), V(C). Native error is Dispatch/code=1 (known ERROR) or 4294967295; later attempt is DispatchPoisoned, without inspection/FFI; one dispatch, one destroy; pointers stable, final runtime/DLL released. |
| preflight, control + transfer | Cr(A), I(B), I(B), I(C), P(C), D(C), S(C), V(C). CommandListType and CommandList/GetDevice HRESULT 0x80004005 preserved exactly, zero native calls on rejections, no poison; recovery dispatch returns 0. |
| destroy-explicit / destroy-drop, control + transfer | Cr(A), I(B), P(B), D(C). Dispatch error=1; rejection after transfer precedes inspection; destroy error=4294967295. One dispatch/destroy, no retry, state/device/runtime/DLL retained. Explicit destroy reports Destroy/code; Drop discards it. |
| unwind-ready / unwind-poisoned, control + transfer | Two Cr(A); poisoned variant adds I(B), P(B) returning 1. Deliberate Rust panic on destination B; D/S/V for first context run on B under family mutex. Surviving sibling waits on a channel, then recovers the poisoned mutex on named sibling-C, performs I/P returning 0 and D/S/V; runtime survives first cleanup, releases finally. Control destination is child test thread; sibling-C remains a coordinated worker. |
| worker missing file | Named worker returns Path error with owned io/NotFound source, no runtime and no acquired module. |
| worker missing each required export | Named worker returns Symbol error with exact ffxCreateContext, ffxDestroyContext, ffxConfigure, ffxQuery or ffxDispatch and owned libloading source. Module absent before load and after failure, no observer retained across cleanup; no runtime. |
| existing D013 and local controls | Repeat eight earlier probes, trait assertions and compiler rejection controls; local ownership/inspection/dispatch regressions pass unchanged. |

There are 14 owner child scenarios and six loader failure observations. Module
observations establish residency after calls, not a DLL termination callback's
executing thread. Retention children terminate without an invented cleanup.

## Setup, implementation, execution and observations

Pending execution. Final commands, fingerprints, observations, unsuccessful
attempts and limits will be appended without changing the predeclared oracle.
