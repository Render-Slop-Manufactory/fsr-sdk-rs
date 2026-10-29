# Oxford English

Oxford English (`en-GB-oxendict`) uses British English with Oxford dictionary
spelling, following the Oxford University Press (OUP) convention of `-ize` and
`-ization` forms where appropriate. See [OUP house-style guidance, section
2.2][oup-spelling]. The
[language policy](language_policy.md) defines which repository content uses it.

The [University of Oxford's communications guide][university-spelling] chooses
`-ise` for its own institutional style and explicitly distinguishes that choice
from OUP's `-ize` convention. This repository follows Oxford dictionary/OUP
spelling.

## Common spellings

| Oxford English prose | American English code names |
|---|---|
| colour | color |
| behaviour | behavior |
| centre | center |
| licence (noun) | license |
| artefact | artifact |
| analyse | analyze |
| organize | organize |
| initialize | initialize |
| normalization | normalization |

Use `licence` for the noun and `license` for the verb in Oxford prose. Use
`organize`, `initialize`, `serialize` and their corresponding `-ization` forms;
do not mechanically replace `-ize` with `-ise`. Words such as `analyse` and
`paralyse` retain their British spelling, while `advertise` and `exercise` retain
`-ise`.

The [University's British/American spelling comparison][university-spelling]
also records the `licence`/`license` noun/verb distinction. Oxford's
[`artefact` dictionary entry][artefact] identifies `artifact` as especially North
American.

These examples describe word spelling. Identifiers still follow their language's
naming style, such as `color_buffer` for a Rust variable or `ColorBuffer` for a
Rust type. In surrounding prose, write "the colour buffer" and preserve the
identifier when referring to it directly.

[oup-spelling]: https://oxrep.web.ox.ac.uk/sites/default/files/oxrep/documents/media/osre_house_style.pdf
[university-spelling]: https://www.ox.ac.uk/about/the-university/brand/style-guide/words
[artefact]: https://www.oxfordlearnersdictionaries.com/definition/english/artefact
