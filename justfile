# List the recipes.
[private]
default:
    @just --list --unsorted

# Run all CI jobs.
ci: fmt check clippy test doc build audit

# Build the workspace.
build:
    cargo build --workspace --all-features --locked

# Check formatting.
fmt:
    cargo fmt --all --check

# Check the workspace.
# `full` only combines `plugins` and `hosted`, so it adds nothing to the powerset.
check:
    cargo hack check --workspace --feature-powerset --exclude-features full --all-targets --locked

# Run clippy.
clippy:
    cargo hack clippy --workspace --each-feature --all-targets --locked -- --deny warnings

# Run tests.
# Each feature runs alone, so the `pretix` schema checks also run against the
# plugin and hosted models.
test:
    cargo hack test --workspace --each-feature --locked
    cargo test --package pretix --features schemars,full --locked

# Run rustdoc.
[env("RUSTDOCFLAGS", "--deny warnings")]
doc:
    cargo hack doc --workspace --each-feature --no-deps --locked

# Audit dependencies for security advisories.
audit:
    cargo audit
