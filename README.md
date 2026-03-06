# hermit

A package manager management tool that unifies package management across multiple package managers.

- **Multi-package manager support**: Works with bun, npm, pnpm, deno, cargo, pip, uv, brew, gem, and go
- **Lockfile support**: Generates `hermit.lock` to ensure reproducible installations
- **Version verification**: Check that installed packages match expected versions
- **Easy package management**: Add, remove, and sync packages with simple commands

```bash
cargo install hermit-rs
```

Or build from source:

```bash
git clone https://github.com/tropicaloss/hermit-rs.git
cd hermit-rs
cargo build --release
```

## Configuration

Create a `.hermit` file in your project root:

```toml
manager = "npm"

[packages]
react = "18.3.0"
lodash = "4.17.21"
```

### Supported Package Managers

| Manager Value | Config | Package Format |
|---------|--------------|----------------|
| Bun     | `bun`        | `package@version` |
| npm     | `npm`        | `package@version` |
| pnpm    | `pnpm`       | `package@version` |
| Deno    | `deno`       | `package@version` |
| Cargo   | `cargo`      | Uses Cargo.toml   |
| pip     | `pip`        | `package==version` |
| uv      | `uv`         | `package@version` |
| brew    | `brew`       | `package` (latest) |
| gem     | `gem`        | `package` (latest) |
| go      | `go`         | `package` (latest) |

## Usage

### Sync Packages

Install all packages defined in `.hermit`:

```bash
hermit sync
```

With verbose output:

```bash
hermit sync -v
```

### Add a Package

Add a new package to `.hermit`:

```bash
hermit add <package> <version>
```

Example:

```bash
hermit add react 18.3.0
```

### Remove a Package

Remove a package from `.hermit`:

```bash
hermit remove <package>
```

Example:

```bash
hermit remove lodash
```

### Regenerate Lockfile

Regenerate `hermit.lock` without installing packages:

```bash
hermit lock
```

### Verify Installations

Check that installed packages match the lockfile:

```bash
hermit check
```

### Clean Up

Remove all hermit-managed installations:

```bash
hermit clean
```

## Files

### `.hermit`

Configuration file defining the package manager and packages:

```toml
manager = "npm"

[packages]
package1 = "1.0.0"
package2 = "2.0.0"
```

### `hermit.lock`

Auto-generated lockfile that tracks installed package versions and their sources:

```toml
[packages.package1]
version = "1.0.0"
resolved = "https://registry.npmjs.org/package1/-/package1-1.0.0.tgz"
hash = "sha512-..."
```

## Cargo Support

When using `manager = "cargo"`, hermit reads dependencies directly from your `Cargo.toml` and uses `cargo fetch` to download them:

```toml
manager = "cargo"
```

No `[packages]` section is needed - hermit will automatically sync all dependencies from `Cargo.toml`.

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
RUST_LOG=debug cargo run -- sync
```

## License

MIT
