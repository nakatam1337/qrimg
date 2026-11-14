# qrimg

Generate QR code images (`.webp`, `.svg`, `.avif`) from text on the command line.

Pure Rust CLI tool — no C compiler or system libraries needed at runtime.

## Install

```bash
# From crates.io (once published)
cargo install qrimg

# Or build from source
git clone https://github.com/nakatam1337/qrimg.git
cd qrimg
cargo install --path . --force
```

## Usage

```bash
# Read from stdin (Ctrl+D to create, Ctrl+C to cancel)
qrimg

# From message argument
qrimg -m "hello world"

# From a text file
qrimg -f notes.txt

# Custom output file and format
qrimg -m "scan me" -t svg -o myqr.svg
qrimg -m "scan me" -t avif -o myqr.avif

# Custom size (square, one side only)
qrimg -m "small qr" -s 256 -t webp -o small.webp

# Error correction level
qrimg -m "important" -e H -t webp -o robust.webp

# Force overwrite without confirmation
qrimg -m "new" -t webp -o existing.webp -y
```

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-m, --message` | — | Text to encode into a QR code |
| `-f, --file` | — | Read message from a text file |
| `-t, --type` | `webp` | Output format: `webp`, `svg`, or `avif` |
| `-o, --output` | timestamp + ext | Output filename (extension decided by `-t`) |
| `-s, --size` | `512` | Square image size in pixels (8–8192) |
| `-e, --ec-level` | `M` | Error correction: `L`, `M`, `Q`, `H` |
| `-y, --force` | — | Skip overwrite confirmation |
| `-h, --help` | — | Print help |
| `-V, --version` | — | Print version |

## Behavior

- **Input priority:** `-m` message > `-f` file > stdin (interactive prompt)
- **Extension:** decided by `-t/--type`, not by the filename you pass to `-o`
- **Default output name:** Unix timestamp + extension (e.g. `1794695770.webp`)
- **Overwrite:** if the output file exists and `-y` is not passed, you'll be prompted (interactive) or it will fail (non-interactive)
- **Input limit:** 2,000 bytes max — QR codes top out around 2,953 bytes, so larger input is rejected with a clear message

## Examples

This section uses the Monero payment address stored in `xmr.txt` (also included in this repo):

```text
monero:84UmH6cimhPMt24owNTJBtdBh2SQjZmDbQswMEZsQN7gDkX4Du9MdKYGnoMUgHW91jAcywZeCiXVgbq7Yz7PGWVTFtAuKbn
```

### Encode via `-f` file input (reads from `xmr.txt`)

```bash
# WebP at 256×256
qrimg -f xmr.txt -t webp -s 256 -o xmr-256.webp

# WebP at 512×512 (default size)
qrimg -f xmr.txt -t webp -s 512 -o xmr-512.webp

# WebP at 1024×1024
qrimg -f xmr.txt -t webp -s 1024 -o xmr-1024.webp

# SVG at 256×256
qrimg -f xmr.txt -t svg -s 256 -o xmr-256.svg

# SVG at 512×512
qrimg -f xmr.txt -t svg -s 512 -o xmr-512.svg

# SVG at 1024×1024
qrimg -f xmr.txt -t svg -s 1024 -o xmr-1024.svg

# AVIF at 256×256
qrimg -f xmr.txt -t avif -s 256 -o xmr-256.avif

# AVIF at 512×512
qrimg -f xmr.txt -t avif -s 512 -o xmr-512.avif

# AVIF at 1024×1024
qrimg -f xmr.txt -t avif -s 1024 -o xmr-1024.avif
```

<figure>
  <img src="examples/xmr-256.webp" width="150" alt="QR code (WebP 256×256)" />
  <img src="examples/xmr-512.webp" width="300" alt="QR code (WebP 512×512)" />
  <img src="examples/xmr-1024.webp" width="450" alt="QR code (WebP 1024×1024)" />
  <figcaption>WebP output at different sizes</figcaption>
</figure>

<figure>
  <img src="examples/xmr-256.svg" width="150" alt="QR code (SVG 256×256)" />
  <img src="examples/xmr-512.svg" width="300" alt="QR code (SVG 512×256)" />
  <img src="examples/xmr-1024.svg" width="450" alt="QR code (SVG 1024×1024)" />
  <figcaption>SVG output at different sizes</figcaption>
</figure>

<figure>
  <img src="examples/xmr-256.avif" width="150" alt="QR code (AVIF 256×256)" />
  <img src="examples/xmr-512.avif" width="300" alt="QR code (AVIF 512×256)" />
  <img src="examples/xmr-1024.avif" width="450" alt="QR code (AVIF 1024×1024)" />
  <figcaption>AVIF output at different sizes</figcaption>
</figure>

### Other input methods

```bash
# Via -m message argument
qrimg -m "$(cat xmr.txt)" -t webp -s 512 -o xmr-message.webp

# Via stdin (Ctrl+D to create)
cat xmr.txt | qrimg -t webp -s 512 -o xmr-stdin.webp

# Custom error correction (higher = more damage-tolerant)
qrimg -f xmr.txt -e H -t svg -s 512 -o xmr-high-ec.svg

# Skip overwrite confirmation (useful in scripts)
qrimg -f xmr.txt -t webp -s 512 -o xmr-512.webp -y
```

### More examples

```bash
# Encode a URL as SVG
qrimg -m "https://example.com" -t svg -o link.svg

# Encode file contents as AVIF
qrimg -f secret.txt -t avif -o secret.avif

# Small QR code for printing
qrimg -m "WiFi password" -s 256 -t webp -o wifi.webp

# High error correction (scannable even if damaged)
qrimg -m "fragile data" -e H -t svg -o robust.svg
```

## Donate

If you find `qrimg` useful, you can donate to the Monero address below:

```
monero:84UmH6cimhPMt24owNTJBtdBh2SQjZmDbQswMEZsQN7gDkX4Du9MdKYGnoMUgHW91jAcywZeCiXVgbq7Yz7PGWVTFtAuKbn
```

Or scan the QR code directly:

<img src="examples/xmr-512.webp" width="200" alt="Monero donation QR code" />

## License

This tool is dedicated to the public domain under **CC0-1.0**. See `LICENSE` for details.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for planned features (quiet zone, custom colors, center logo, batch mode).
