# Dither QR
Generate a valid QR code with a dithered background image.

## Usage
To install:
```fish
cargo install dither-qr
```

For options:
```fish
dither-qr --help
```

Example:
```fish
dither-qr -t "https://github.com/peterc-s/dither-qr" -i /path/to/image -o /path/to/output -r 7 -e H -u 3
```
Will generate a QR code that encodes `https://github.com/peterc-s/dither-qr`, with a dithered background image `/path/to/image`, with a ratio of 7 (7x7 pixels per original QR pixels), a high error correction level, and will upscale the output image by 3 times, outputting to `/path/to/output`.

An example output looks like:
![Dithered QR code for this repo with the statue of David as a background image](img/statue-of-david.png?raw=true "Dithered QR")
