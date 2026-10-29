# Declared Proton temporal regression selection

Declared on 2026-10-13 before GPU execution. V07d only; V08/V09 excluded.

Run `proton-temporal-quality`, `proton-temporal-long`, `proton-temporal-reset`,
and `proton-temporal-jitter` on the available SteamOS host. Each repeats the
existing Proton query/single-frame control, an independent input-only oracle,
three process blocks and the existing analyser. Require pinned SDK bytes,
observed loaded module paths, fence/readback/finite/sentinel/device checks,
explicit context destruction and Vulkan core/synchronization validation without
errors in each child. Windows DirectX debug/info-queue/GBV equivalence is excluded;
missing info-queue counts remain UINT64_MAX. Numeric budgets, control sensitivity
predictions and confirmation windows are frozen from the existing Windows
protocols and must not be adjusted after observing this run.

Jitter selects stationary phase 0 and moving phase 0, all six controls and
128 frames per arm. The three other stationary starts are excluded to bound
raw temporary storage on this host. Existing group criteria are unchanged.
Quality, long and reset retain their original matrices. A missing prerequisite,
nonzero child exit, marker failure, invalid input/output, Vulkan error or failed
frozen image criterion must remain failed or blocked. Each profile is independent:
a failure in one does not prevent running the other selected protocols.

This establishes selected regression observations on one GPU/provider, not
full native/temporal portability or universal quality. Preserve every failed
attempt and compact original diagnostics after independent evaluation.
