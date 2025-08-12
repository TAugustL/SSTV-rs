# SSTV-rs
Convert images into a SSTV signal.

## What?
> Slow-scan television (SSTV) is a picture transmission method, used mainly by amateur radio operators, to transmit and receive static pictures via radio in monochrome or color.

<a href="https://en.wikipedia.org/wiki/Slow-scan_television">Wikipedia: 'Slow Scan Television' (12.08.25, 17:45)</a>

Basically, you can convert images into a sound that can be decoded back into images. As mentioned in the quote above, this was developed to send images over radio, but you can use it for whatever you want (I got the idea to make this because I heard about SSTV being used in an easter egg in a video game).

## How?
There are many modes for SSTV signals, but my program covers the most used:
<table>
  <tr>
    <th>Scottie</th>
    <th>Martin</th>
    <th>Robot</th>
    <th>Wraase</th>
  </tr>
  <tr>
    <th>Scottie S1</th>
    <th>Martin M1</th>
    <th>Robot BW8</th>
    <th>Wraase SC-2 180</th>
  </tr>
  <tr>
    <th>Scottie S2</th>
    <th>Martin M2</th>
    <th></th>
    <th></th>
  </tr>
  <tr>
    <th>Scottie S3</th>
    <th>Martin M3</th>
    <th></th>
    <th></th>
  </tr>
  <tr>
    <th>Scottie S4</th>
    <th>Martin M4</th>
    <th></th>
    <th></th>
  </tr>
  <tr>
    <th>Scottie DX</th>
    <th></th>
    <th></th>
    <th></th>
  </tr>
</table>

Each mode group (Scottie, Martin, ...) has their own unique way to encode images, but you could just say that there are certain sounds that play at certain times in the signal to tell the decoder where a row ends or what coller to decode. My program generates these sounds and saves them in a uncompressed WAV file. You can then do anything with that file, but I recommend converting it to another format with the converter of your choice. Since SSTV is relatively resistant against some noise in the signal, you can also convert the file into a lossy format like MP3 and it still works.

> [!WARNING]
> Increasing the compression of the file will cause the decoded image to look worse.

If you want to test your files, I recommend installing this <a href="https://f-droid.org/packages/om.sstvencoder/">SSTV Encoder</a> from F-Droid to your Android phone.

To use my program do the following:

```
cargo run --release [PATH TO IMAGE] [SAVE DESTINATION] [SSTV MODE]
```

You can also just use ```cargo run --release``` to get all the help you need.

## Why?
Because it's fun. I even encourage you to make your own SSTV encoder in the programming language of your choice. Just know that you may look like an insane person to people around you if you sit alone in your room with loud beeping noise coming from your computer.
The sources I used for this project are <a href="https://www.sstv-handbook.com/">this extremely helpful PDF</a> and <a href="https://github.com/BenderBlog/rust-sstv">this repository by BenderBlog</a>.

<hr>

This project was written in Rust 1.87 with the <a href="https://crates.io/crates/image">image crate</a> (0.25.6) as the only dependency.
