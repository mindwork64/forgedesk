# Local Jenkins CI

ForgeDesk uses the local Jenkins instance for CI. Credentials are supplied only
through environment variables; they are never committed, logged, or written to
the repository.

## Why a local git mirror

Jenkins runs as user `jenkins` and cannot traverse `/home/eloraam` on this host
(`drwxr-x---+`), and its `/tmp` is a private systemd namespace. The working tree is
therefore published to a bare mirror inside the workspace and served read-only over
`git://127.0.0.1:9418`:

```text
.tmp/ci-mirror/forgedesk-ci.git  ← scripts/ci-sync.sh ← ForgeDesk working tree
        │
        └─ scripts/ci-serve.sh (git daemon)
                │
                └─ Jenkins job ForgeDesk-CI → Jenkinsfile → scripts/ci.sh
```

The `.tmp` mirror is git-ignored and survives tool invocations, but not a machine
reboot. Keep the git daemon running for Jenkins builds. When the repository gets a
real remote (Gitea/GitHub), point the Jenkins SCM configuration at it and retire
the mirror. The mirror path can be overridden with `FORGEDESK_CI_MIRROR`; the daemon
root/port with `FORGEDESK_CI_MIRROR_ROOT` / `FORGEDESK_CI_GIT_PORT`.

## One-time setup

```bash
# 1. Initialize git if needed (the project working tree is a repository).
git init -b main
git config user.name "ForgeDesk CI"
git config user.email "ci@forgedesk.local"

# 2. Publish the current tree to the Jenkins-readable mirror.
./scripts/ci-sync.sh "Stage snapshot"

# 3. Serve the mirror over git:// (keep this process running).
./scripts/ci-serve.sh

# 4. Create or update the pipeline job from the versioned XML.
export JENKINS_URL=http://127.0.0.1:8080
export JENKINS_USER=eloraam
export JENKINS_TOKEN=...        # never commit this value
./scripts/jenkins-create-job.sh
```

## Run

```bash
./scripts/ci-sync.sh "Before CI"
# git daemon must already be running (scripts/ci-serve.sh)
export JENKINS_URL=http://127.0.0.1:8080
export JENKINS_USER=eloraam
export JENKINS_TOKEN=...
./scripts/jenkins-trigger.sh
```

The job runs `scripts/ci.sh`:

- `npm ci`, typecheck, 66 Vitest tests, production build;
- `cargo fmt --check`, core clippy/tests, shell `cargo check`/clippy;
- UI Gate 0 with Playwright at 1440×900, 1280×800 and 1100×700;
- archives `docs/ui/current/**` screenshots and measurements.

If WebKitGTK is absent, the shell stage uses the explicit typecheck shims in
`ci/pkgconfig/` and says so in the log. A real native build/link still requires a
machine with WebKitGTK/WebView2.

## Security notes

- Do not put the token in `Jenkinsfile`, job XML, scripts, commit messages or
  documentation. The provided token was used only through environment variables
  and is not stored in the repository.
- Use a short-lived API token and revoke it when rotating access.
- The job archive contains screenshots and sanitized metadata only. The visual
  harness uses fake paths (`/home/dev/...`), not real user paths or secrets.
- The local mirror contains source code under the workspace `.tmp` directory; keep
  the daemon bound to `127.0.0.1` as configured. Use a private remote when available.

## Current status

- Jenkins controller: 2.580.2, built-in Linux node, 2 executors.
- Job: `ForgeDesk-CI`, pipeline from `Jenkinsfile`, SCM
  `git://127.0.0.1:9418/forgedesk-ci.git`.
- Build 2 (Stage 1 snapshot): **SUCCESS** in 234 s; frontend/core/shell typecheck
  and UI Gate 0/1 screenshots archived.
- Build 3 (Stage 2 snapshot): run after `ci-sync.sh`; record the result here.
