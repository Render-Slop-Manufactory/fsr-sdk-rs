# Reviewing distributed content

Review the current repository tree before distributing a release candidate, and
review the exact extracted `.crate` contents before committing those archives.
GitHub source archives and crates.io packages contain different file sets;
passing one review does not cover the other. Candidate files remain in Git
history after later deletion.

## Repeatable screening

From the repository root, run these read-only ripgrep searches. They respect
ignore rules and omit local build/SDK data and Git's object store. Output lists
filenames only; inspect matches locally without pasting sensitive values into
reports. Exit status 0 means matches to review, 1 means no matches, and 2 means
the scan failed and must not be reported clean.

```sh
rg -l --hidden -g '!.git/**' -g '!target/**' -g '!external/**' -- '-----BEGIN ([A-Z ]+ )?PRIVATE KEY-----' .
rg -l --hidden -g '!.git/**' -g '!target/**' -g '!external/**' -- '\b(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,}|AKIA[0-9A-Z]{16})\b' .
rg -l --hidden -g '!.git/**' -g '!target/**' -g '!external/**' -- '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' .
rg -l -i --hidden -g '!.git/**' -g '!target/**' -g '!external/**' -- '(/home/[^/<>[:space:]]+|/Users/[^/<>[:space:]]+|\b[A-Z]:\\+Users\\+[^\\<>[:space:]]+|\b[A-Z]:[\\/]+[^<>"[:space:]]*fsr-sdk-rs[\\/])' .
rg -l --hidden -g '!.git/**' -g '!target/**' -g '!external/**' -- '/run/media/[^/<>[:space:]]+/[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}' .
```

For packages, use the verification coordinator's validated extraction and repeat
the same searches with `.` replaced by the extracted candidate directory. Scan
each exact selected pair, record archive SHA-256 values, tool version, commands,
exit statuses and the disposition of matches. Do not scan compressed tarball
bytes as a substitute for inspecting their members. Inspect notices, file lists
and unexpected binary/generated content through the existing package checks too.

The patterns screen common private-key/token forms, email addresses and local
workspace paths and local external-volume UUID mount roots. They are review aids,
not a complete secret detector or a proof
that no personal information exists. Quoted test fixtures, public upstream
contacts, licence attribution and generic system paths can be intentional;
review their purpose instead of deleting them indiscriminately. Retain hardware,
driver, provider and SDK identities needed to interpret verification.

## Corrections and history

For unnecessary data in historical evidence, create a documented sanitized
derivative and retain original/sanitized file hashes, replacement scope and
unchanged-outcome checks. Do not silently change native/source identities or
rewrite historical inventories to pretend they hashed the new bytes. The
[October 29 correction](release/2026-10-29-evidence-privacy-correction.md) is an
example. Future exports should omit unrelated Cargo author/contact metadata.
The [Deck volume correction](release/2026-10-29-deck-volume-privacy-correction.md)
similarly preserves historical identities while replacing unnecessary volume
identifiers in retained Wine drive mappings.

The commands above inspect current text, not Git history, author identities,
binary metadata or all possible credential formats. Review deliberate author/
committer identities separately before publication; `git log --all` can enumerate
them locally but is not a content scan of every historical revision or remote
ref. Any credential finding needs its own assessment, including revocation where
applicable. History rewriting and publication require separate authorisation.
