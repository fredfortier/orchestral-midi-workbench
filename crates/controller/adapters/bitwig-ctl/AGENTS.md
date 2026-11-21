# Bitwig Java adapter

- Read the [native host contract](../../ARCHITECTURE.md#native-host-execution) before changing SDK or private document calls. Use [wire and deployment rules](../../ARCHITECTURE.md#wire-deployment-and-verification) when changing the adapter-service boundary.
- From this directory, use the Maven wrapper and the JDK required by [pom.xml](pom.xml). Supply the qualified installed Bitwig JAR with `-Dbitwig.runtime.jar=/absolute/path/to/bitwig.jar` when it differs from the POM default.
- Run `./mvnw verify` for production changes and `./mvnw -P harness verify` for harness or native-boundary changes. Keep artifacts in their separate Maven output trees.
- Follow the [qualification runbook](../../RUNBOOK.md) for native acceptance. Use a disposable project and keep fault injection in the development harness.
