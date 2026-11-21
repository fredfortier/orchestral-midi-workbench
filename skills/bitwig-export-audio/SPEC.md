# Audio-export skill design rationale

Authoring-only rationale. [SKILL.md](SKILL.md) owns the complete runtime procedure;
[controller MIXING.md](../../controller/MIXING.md) owns the implementation contract.

Native task ownership was chosen over dialog scraping, destructive bounce or
solo-and-render loops because the task supplies exact scope, common timing and a
retained asynchronous result. Comparative renders and delivery exports have
different purposes, so their format choices were kept explicit in runtime.

The publisher's [export guide](https://www.bitwig.com/userguide/latest/exporting_audio/)
and [bounce guide](https://www.bitwig.com/userguide/latest/bouncing_to_audio/)
supplied UI and tap semantics. They do not establish which options the private
controller has qualified. [SOURCES.md](SOURCES.md) records that implementation
lineage and the original author walkthroughs. No operating rules reside here.
