# Authoring evidence

Authored 2026-11-18 under explicit user authority to implement native filtered
audio export and its associated skill. Shape: inline runtime workflow in SKILL.md;
SPEC.md retains authoring rationale only. Repository skills/ is the owning
root; discovery uses the established user-skill symlink convention.

Official source lineage appears in SPEC.md; operating consequences are inline
in SKILL.md.
Bitwig's rolling export/bounce guides were inspected on 2026-11-18. The installed
6.1 native DXq Export Audio action constructs Zv with TQb task-owned settings;
the implementation uses that same task and captures its final success/error
callback. Private mappings are host-specific evidence, not public API promises.
Native qualification outcomes belong to controller/WORKPAD.md.

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
