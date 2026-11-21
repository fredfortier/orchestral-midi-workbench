SLUG ?=

.PHONY: new-adr new-plan

new-adr:
	@if [ -z "$(SLUG)" ]; then echo "usage: make new-adr SLUG=decision-slug"; exit 1; fi
	./scripts/new-adr.sh "$(SLUG)"

new-plan:
	@if [ -z "$(SLUG)" ]; then echo "usage: make new-plan SLUG=my-plan"; exit 1; fi
	./scripts/new-exec-plan.sh "$(SLUG)"
