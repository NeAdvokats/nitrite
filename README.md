# nitrite

A minimal package manager: builds packages from source and installs them into `~/.local`.

## Installation

Using make:

```sh
git clone https://github.com/NeAdvokats/nitrite
cd nitrite
make
```

Or grab a pre-built binary from the repository releases and move it to `/usr/bin`:

```sh
chmod +x nit
sudo mv nit /usr/bin/nit
```

## Usage

```sh
nit --sync              # download the index
nit --install PACKAGE   # install a package
```

`nit` downloads the archive, verifies its `sha256`, extracts it, validates `PKGINFO`,
runs `build.sh`, and copies the output into `~/.local`. Temporary files are removed
after a successful installation.

`build.sh` receives `SRCDIR` containing the source code and `PKGDIR` for the built files.
The contents of `PKGDIR` mirror the structure of `~/.local`—for example,
`$PKGDIR/bin/sifetch` will be installed to `~/.local/bin/sifetch`.

Building packages requires `sh` and build tools such as `gcc` and `make`.

<a href="https://github.com/NeAdvokats/nitrite/blob/main/README-ru.md">По-русски</a>
