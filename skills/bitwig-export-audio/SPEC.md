# Audio-export skill design rationale

Authoring rationale for the [production procedure](SKILL.md).

Native task ownership was chosen over dialog scraping, destructive bounce or
solo-and-render loops because the task supplies exact scope, common timing and a
retained asynchronous result. Comparative renders and delivery exports have
different purposes, so their format choices were kept explicit in runtime.

The publisher's [export guide](https://www.bitwig.com/userguide/latest/exporting_audio/)
and [bounce guide](https://www.bitwig.com/userguide/latest/bouncing_to_audio/)
supplied UI and tap semantics. [SOURCES.md](SOURCES.md) records the scope
considered in the author walkthroughs.
