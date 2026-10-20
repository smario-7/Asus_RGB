# Releasing a-rgb

**English** | [Polski](RELEASING.pl.md)

Instructions for the repository maintainer: how to build the packages locally, how to test them and how to publish a Release on GitHub.

## What gets built

| File | Script | Contents |
|---|---|---|
| `a-rgb_VERSION_amd64.deb` | `packaging/build-deb.sh` | binary, udev rule, `restore` service (see [PACKAGE.md](PACKAGE.md)) |
| `a-rgb-VERSION-x86_64-linux.tar.gz` | `packaging/build-tarball.sh` | binary, `install.sh`, rule, unit, READMEs, LICENSE |
| `SHA256SUMS` | workflow | checksums of both files |

Both files contain the same **static** binary built for the `x86_64-unknown-linux-musl` target, so it does not depend on the glibc version of the system. The package version comes from `version` in `Cargo.toml`.

## Building locally

Requirements: Rust ≥ 1.85, `dpkg-deb`, the musl target.

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --locked --target x86_64-unknown-linux-musl
BIN=target/x86_64-unknown-linux-musl/release/a-rgb
ldd "$BIN"                                   # expected: "not a dynamic executable"
packaging/build-deb.sh     "$BIN" dist
packaging/build-tarball.sh "$BIN" dist
(cd dist && sha256sum -- *.deb *.tar.gz > SHA256SUMS)
```

The `dist/` directory is in `.gitignore`, so the packages will not end up in a commit.

Package contents and metadata: `dpkg-deb -I dist/a-rgb_*.deb` and `dpkg-deb -c dist/a-rgb_*.deb`.

## Testing the installation in containers

Installing in a container does not touch your system. **Warning:** a container shares `/sys` with the host, so do not run commands that write (`a-rgb red`, etc.) in it. `/sys` is read-only in a container by default, but do not rely on that. Only check installation, `--version`, `-h` and removal.

```bash
for img in ubuntu:22.04 ubuntu:24.04; do
  docker run --rm -v "$PWD/dist:/dist:ro" "$img" bash -c '
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq && apt-get install -y -qq /dist/a-rgb_*_amd64.deb
    a-rgb --version && a-rgb -h | head -3
    dpkg -L a-rgb
    apt-get remove -y -qq a-rgb && ls /usr/bin/a-rgb 2>&1'
done
```

The base images have no `systemctl`, so `systemctl --global enable` in `postinst` is skipped there (`|| true`). You can check the `/etc/systemd/user/default.target.wants/a-rgb-restore.service` symlink by installing the `systemd` package before the package under test.

Test the `tar.gz` archive the same way (`alpine:3`, `debian:12`): unpack it, run `./install.sh`, check the files, then `./install.sh --uninstall`.

## Releasing

1. Bump `version` in `Cargo.toml` and run `cargo build` (this updates `Cargo.lock`).
2. Check locally:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   ```
3. Commit the version change and push it to `main`. Wait for green CI (`.github/workflows/ci.yml`).
4. Create the tag, **check that it points at the latest commit**, and only then push it:
   ```bash
   git tag -a vX.Y.Z -m "a-rgb X.Y.Z"
   [ "$(git rev-parse "vX.Y.Z^{commit}")" = "$(git rev-parse HEAD)" ] && echo MATCH || echo MISMATCH   # must say MATCH
   git push origin vX.Y.Z
   ```
   The Release is built from the commit the tag points to. A tag created before the last changes were committed releases the old version (that is what happened with `v0.1.0`, whose packages lacked the fix from `edd3421` — which is why `v0.1.1` exists).
5. The `.github/workflows/release.yml` workflow then:
   - checks that the tag matches `version` in `Cargo.toml` (tag `v0.1.0-rc1` matches version `0.1.0`),
   - runs fmt, clippy and the tests,
   - builds the static binary and checks that it really is static,
   - builds the `.deb` and `tar.gz` and computes `SHA256SUMS`,
   - creates the Release with the description from `.github/release-notes.md` (tags containing a hyphen, e.g. `-rc1`, are marked as pre-releases).
6. Verify the published Release:
   ```bash
   gh release view vX.Y.Z
   mkdir /tmp/rel && cd /tmp/rel && gh release download vX.Y.Z
   sha256sum -c SHA256SUMS
   ```
   and install the downloaded `.deb` in a clean container as above.

### Workflow changes: pre-release first

The `ci.yml` and `release.yml` workflows have passed on GitHub (releases `v0.1.0-rc1` and `v0.1.0`). After any change to `release.yml` or to the scripts in `packaging/`, release a candidate first, e.g. `vX.Y.Z-rc1`: if something fails, you fix the workflow without leaving a trace in a stable release.

### A failed release

Deleting a Release and its tag is irreversible for anyone who has already downloaded it. Do it only for a release that should not exist:

```bash
gh release delete vX.Y.Z --yes
git push --delete origin vX.Y.Z && git tag -d vX.Y.Z
```

## Workflow security

- The only third-party action is the official `actions/checkout`, **pinned to a SHA** (the comment next to the SHA gives the version). To update it: `gh api repos/actions/checkout/releases/latest --jq .tag_name`, then `gh api repos/actions/checkout/commits/TAG --jq .sha`, and replace the SHA in both files.
- Only the release job has the `contents: write` permission; everything else runs with `contents: read`.
- The tag name reaches the scripts through the `TAG` environment variable, not by inserting the expression directly, so that a tag name cannot inject commands.
- The Release is published by `gh`, which is preinstalled on the runner, with no third-party actions.

## Compatibility statement in the Release notes

The `.github/release-notes.md` template says plainly that the software is tested only on the ASUS TUF Gaming F15 FX506LHB (Ubuntu 26.04, GNOME/Wayland). Do not broaden that claim until someone has checked another model on real hardware.
