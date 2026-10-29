# M9 public native integration — retained summary

This compact summary replaces the superseded 2026-10-10 M9 release projection
and its verbose verification export, by maintainer direction on 2026-10-29.
The URL remains because current package Rustdoc links to the public integration
observations. The original command/log collection is in Git history; this page
is neither a current release recipe nor fresh verification. Publication facts
and the later movable-inclusive release are in [release history](../ROADMAP_HISTORY.md#020--published-2026-10-10).

## Result and scope

The projected candidate and independently built extracted-package native arms
each passed three repetitions of the selected public sharpening integration
protocol: two scenes, three contexts/scene, eight fenced outputs/context,
**144 outputs per arm**. Both arms also passed five disabled lifecycle/dispatch/
motion-jitter-reset children. Every required integration child exited zero;
the separate original [effect experiment](../research/records/2026-09-30-exp-d014-native-sharpening.md)
remained failed. Execution and mapping acceptance added no image-quality claim.

The test cell was Windows 11 x64/MSVC, RX 9060 XT (`1002:7590`), driver
`32.0.31041.1004`, SDK v2.3.0 and provider 4.1.1 (`0xf5a5ca1e01001001`).
Pinned header commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`.
Both DLLs had Valid Windows Authenticode status; SHA-256 values were loader
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`
and upscaler `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Ordinary debug validation was active; GBV was excluded.

## Integration protocol

The selected [D014](../adr/D014.md)/[D016](../adr/D016.md) gate checked direct
native control rather than a guessed sharpening algorithm. Deterministic scalar/
descriptor/controlled-FFI checks separately established exact enablement and
`f32` forwarding, including signed zero, before interpreting GPU results.

Inputs were 320x180 render, 640x360 output, zero motion/creation/dispatch flags,
explicit exposure 1, depth 0.5, frame time 16.667 ms, near/far 0.1/100, FOV
pi/3 and metres factor 1. The two half-float scenes used unjittered coordinates
`u=x+0.5-jitter_x`, `v=y+0.5-jitter_y`: frequency RGB
`0.5+0.4*sin(0.8*(u+0.37*v))`; slanted-edge RGB 0.1 below `u<128+0.37*v`,
otherwise 0.9; alpha 1.

Each scene used switching contexts A/B with identical histories and an
always-Disabled control. The same switching contexts ran Disabled for frames
0–3, Native(0) at 4, Native(0.35) at 5, Native(1) at 6 and Disabled at 7.
Only frame 0 reset history; no context recreation or extra mode-change reset
was allowed. Jitters were `(0.25,-0.25)`, `(-0.25,0.25)`, `(0.125,-0.375)`,
`(-0.375,0.125)`, `(0.375,-0.125)`, `(-0.125,0.375)`, `(0.0625,-0.0625)`
and `(-0.0625,0.0625)`. Three isolated process repetitions used a 180-s child
watchdog, without automatic retry or dropping failed repetitions.

Each successful recording was submitted once and fenced/read back before list,
allocator or texture reuse. Acceptance required all 230,400 output pixels finite
and written, completed fences, zero ordinary debug errors, no device removal,
successful terminal destruction and a clean post-cleanup device check. Native
errors, timeouts or unknown completion could not pass: never submit a failed
recording and preserve possibly in-flight dependencies when completion is unknown.

## Observations and limits

For frames 3–7, the independent audit compared A/B and switching/Disabled red
half-float data over `x=[32,608), y=[32,328)`, using bit equality and f64 MAE.
These were diagnostics with no minimum effect, identity, brightness, gradient,
monotonicity or image-quality threshold. Enabled zero differed from Disabled
on both scenes. That is a bounded observation, not universal filter semantics;
returning to Disabled did not promise identity with a different temporal history.

Source and artefact runs passed those execution/cleanup criteria. Exact descriptor
mapping came from controlled FFI evidence, not from inferring native parameters
from an image difference or an OK return. The original failed effect assertions,
GBV stall and caller-established native/threading/retention assumptions remain
unchanged. No other GPU/provider, universal access safety, image quality or
performance claim follows.

## Historical candidate boundary

The original M9 projection **excluded movable** despite its presence in the
development source. Its local archives were raw 0.1.1
`4f29475b2aef3527072111bd0f4f5e7c1828c4881eaa72a3ba96d7e275f54465`
and wrapper 0.2.0 `a8b293ec70c0b6e52897fba12e04b105154cd12caa95f18277ec6ed9dacd1276`.
Package, Stable/MSRV consumer, dependency-tree and joint dry-run checks passed;
both uploads were aborted. Initial restricted file/network and fully offline
registry failures were unsuccessful setup attempts, not successful gates.
The later [complete 0.2.0 release](../ROADMAP_HISTORY.md#release-qualification)
included movable and used different archives. Neither historical pair qualifies
0.3.0.

The [indexed script archive](../../tools/verification/archive/2026-10-10/README.md)
retains the frozen collectors/auditors. Their original source paths and intake
references remain historical metadata. The removed logs/JSON inventories are
available in Git history; no current full-image reanalysis is promised.
