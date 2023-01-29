# Loempia

Loempia is a Rust tool set for controlling the pen plotter from [Evil Mad Scientist](https://shop.evilmadscientist.com/).
This plotter implements the [EBB (EiBotBoard)](https://evil-mad.github.io/EggBot/ebb.html#) command set.

One of my first goals for this project plot hiking trails.

## Quick start

To print the firmware version of the board:

```bash
$ cargo run --example version
```

To draw a track recorded in a GPX file:

```bash
$ cargo run --example gpx -- examples/data/spitzstein.gpx
```

To draw a square:

```bash
$ cargo run --example square
```

Or draw a triangle:

``` bash
$ cargo run --example triangle
```
<p align="center">

https://user-images.githubusercontent.com/1565144/212554005-43d56c1a-167c-402c-a504-ec8a87d451be.mp4
</p>

# License

[MIT](LICENSE)
