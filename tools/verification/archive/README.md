# Verification archives

Frozen verification scripts belong to dated, indexed archives. Active tests and
their current commands are described in [development](../../../docs/DEVELOPMENT.md).

| Archive | Contents |
|---|---|
| [2026-10-10](2026-10-10/README.md) | Retired milestone wrappers, standalone native experiments, dated release verifiers/collectors and independent audits; exact intake bytes, dependencies, historical references and reconstruction limits |

Payloads use a final `.txt` suffix so they are source artefacts, not ordinary
executable entry points. Each archive records its intake date and provenance;
being referenced by a historical report does not establish that the intake copy
matches the exact version used by that earlier run.

Run `python tools/verification/archive/check.py` from the repository root to check
archive identity, navigation, dependency inventory and Python/JSON syntax without
executing historical workflows. These checks require Python 3.11+ and no shell.
PowerShell syntax is checked when PowerShell 7 (`pwsh`) is available. A missing
parser is explicitly reported as `not_run` / `not_evaluated`, with its reason;
passing integrity checks does not establish PowerShell syntax validity.

Use `python tools/verification/archive/check.py --require-powershell` to require
that syntax check as well; a missing or failed parser then prevents acceptance.
Windows `ci` and `release` use this strict mode. Syntax errors or an unusable
installed parser fail in either mode. `--report <new-json-file>` writes separate
subcheck outcomes, including unavailable or failed parsing, to a new file. The
coordinator retains those outcomes with its evidence. Ordinary Cargo builds
acquire no new dependency.
