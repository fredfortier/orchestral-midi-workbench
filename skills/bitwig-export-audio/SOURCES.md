# Audio export workflow sources

Bitwig’s export and bounce guides are cited in [the skill rationale](SPEC.md).

Adopted: exact observed paths, separate group/leaf/master/return choices, explicit
options/range, new directories, original operation polling, independent audio
verification. Rejected: dialog scraping, destructive bounce, solo-and-render
loops, inferring success from a nonempty WAV or engine completion alone.
Deferred: other UI codecs, arbitrary rates, automatic tails and folder-opening
preference control; the command rejects unsupported input rather than ignoring it.

Author walkthroughs: filtered analysis exports preserve common range/relative
levels; a stale owner refuses before dispatch; cancellation retains a failed
native outcome despite a partial file; a general mixing request returns to its
caller after audio verification. Advice-only and MIDI-export requests are not
audio-export invocations. Structural validation and live command smoke are
separate from skill-discovery or fresh-context model-behavior claims.
