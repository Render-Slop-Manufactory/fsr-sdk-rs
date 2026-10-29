# Language policy

This document is authoritative for the repository's English-language conventions.
It applies to new files. The maintainer handles language changes in existing
files separately.

## Prose and code

Use Oxford English (`en-GB-oxendict`) for prose and American English (`en-US`)
for code names and machine-facing tokens, regardless of the programming language
or the file containing them.

| Content | Convention |
|---|---|
| Documentation, including Markdown prose and Rustdoc | Oxford English |
| Comments and docstrings in every programming language, including `SAFETY` comments | Oxford English |
| Error messages, CLI help, warnings, logs and other human-readable messages | Oxford English |
| Source-code identifiers, including test function names | American English |
| Filenames and directory names | American English |
| Configuration keys, CLI command names and option names | American English |

Apply the distinction within mixed content: code names, paths, commands and keys
retain their exact spelling when embedded in prose; comments and human-readable
messages use Oxford English when embedded in code or documentation examples.
For example, prose can describe the colour data held in `color_buffer`.

Reproduce upstream API names, established interfaces and technical names exactly
when referring to them.

## Oxford English guidance

See [Oxford English](OXFORD_ENG.md) for spelling guidance and examples. This policy
owns the conventions and their scope; that guide explains the prose spelling.
