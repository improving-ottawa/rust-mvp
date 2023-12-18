# rust-mvp

## Developer's README

### Configure local git hooks

There are local Git hooks in this repo; they are also used in the remote CI pipeline.

To configure Git to automatically run these hooks in response to Git events, you must change the default hook directory with

```shell
git config --local core.hooksPath .githooks
```

You also need to make all hooks executable with

```shell
git update-index --chmod=+x .githooks/*
```

We use `git` above, instead of `chmod` directly, for cross-platform compatibility.

To run (the bulk of) the CI pipeline locally, simply execute the `pre-push` hook as a script

```shell
sh .githooks/pre-push
```

See https://github.com/rust-sketches/ci-github-actions for more information.