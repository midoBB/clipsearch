# ClipSearch - Wayland Clipboard Manager

`ClipSearch` is a tool for managing clipboard history and performing various operations such as storing, listing, deleting, and wiping clipboard data.

## Build and Install

```bash
make
```

To install, run:

```bash
make install
```

By default, it will install to `~/.local/bin`. You can change the install location by setting the `PREFIX` environment variable. For example, to install to `/usr/local/bin`, run:

```bash
PREFIX=/usr/local make install
```

## Uninstall

To uninstall, run:

```bash
make uninstall
```

## Usage

```bash
# Store clipboard content
wl-paste --type text --watch clipsearch store &
wl-paste --type image --watch clipsearch store &

# List clipboard history
clipsearch list

# Wipe clipboard history
clipsearch wipe

# Run the GUI to search through clipboard history
clipsearch gui

# Print version information
clipsearch version
```
