# Complete infrastructure tests with NumPy and PowerShell

Date: 2026-10-13, Europe/Paris. This ordinary local verification follows the
[Proton proof](../research/records/2026-10-13-exp-proton-linux-cross-build.md),
whose infrastructure check had explicitly skipped 31 tests. That historical
observation remains unchanged.

The source baseline for this run was
`9532598528cb2ba93765866a89bbc1650b99fedb`, with an initially clean worktree.
No test or production source changed for this check. The source contains 28
NumPy-dependent image-evaluator tests and three real PowerShell-parser tests.
NumPy and PowerShell were absent from the previous Python/PATH environment;
there was no platform blocker to installing them on the Steam Deck.

## Command and inputs

```sh
nix shell --impure --expr 'let p = (builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux; in [ (p.python3.withPackages (ps: [ ps.numpy ])) p.powershell ]' --command python3 -m unittest discover -s tools/verification/tests -v
```

The Nix registry resolved `nixpkgs` to revision
`7a0f122f5090cf4c2ade2a13a0e229d4e19ba71f`. It supplied Python 3.14.7 with
NumPy 2.5.2 and PowerShell 7.6.6. Nix fetched 125.8 MiB, 410.5 MiB unpacked,
including numerical-library and PowerShell dependencies. The tools are available
in that command's shell; the user's default Python/profile was not replaced.
The mutable registry alias is convenient for local setup, not a version pin;
this record identifies what actually ran.

Execution occurred outside the agent sandbox because the existing subprocess
watchdog tests need local socket/process access. The earlier sandbox permission
failures are environment restrictions, not accepted test outcomes.

## Result and limits

The command exited **0**:

```text
Ran 125 tests in 118.016s

OK
```

All 125 tests passed; none were skipped. This includes the image-evaluator
negative controls and the real PowerShell parser's valid/invalid-input checks.
It closes the 31 missing local infrastructure checks from the earlier run.

This is verification of test infrastructure and numerical evaluators, not a
statement about measured Rust code coverage or another GPU workload. PowerShell
on Linux does not supply MSVC, Windows Authenticode policy or the DirectX debug
layer. The separately recorded Proton GPU proof still covers public queries and
one single-frame dispatch/readback on its recorded configuration, with its
explicit missing Windows diagnostics and untested temporal-regression scope.

The current CI workflow does not explicitly install NumPy. This local complete
run does not prove that a hosted CI invocation has all these optional test
dependencies, and does not change the workflow's dependency policy.
