# Roadmap

## Implemented (v1.x)

- QR code generation from text via `-m`, `-f` (file), or stdin
- Output formats: WebP (default), SVG, AVIF
- Extension decided by `-t/--type`, not filename argument
- Custom output size with bounds (8–8192 pixels)
- Reed-Solomon error correction levels: L, M, Q, H (`-e/--ec-level`)
- Overwrite protection with `-y/--force` to skip
- Input size limit (2,000 bytes max) to prevent accidental oversized input
- Default output name: Unix timestamp + extension
- Animated spinner with format emoji during generation
- Unit tests (lib) + CLI integration tests
- 100% pure Rust dependencies (no C compiler needed at runtime)

## Roadmap (v2.x — not yet implemented)

### Custom colors
Allow specifying foreground and background colors instead of black/white.
Useful for branding or matching design guidelines.

### Center logo / image
Embed a small image or logo in the center of the QR code.
Works well with higher error correction levels (Q, H) which can tolerate
the missing modules.

### Batch mode
Read multiple lines from a file and generate one QR code per line.
Output files named after line number or content hash.

## Not planned

None at this time.
