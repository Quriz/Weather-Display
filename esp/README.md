# esp

This directory contains esp-idf/esp-rs based firmware to download an image over wifi and display it on a waveshare e-ink display.

## Build

Follow the instruction in the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites) of the `esp-idf-template` to set up your system.

You must first create a `config.json` file containing your Wifi connection credentials and the location of the file to download and display.
See [`example-config.json`](./example-config.json) for reference.

Then you can run `just build` to build assuming you have the rest of your environment set up as in the [embedded rust book](https://docs.rust-embedded.org/book/intro/install.html).

If you have an esp device plugged in over usb, you should be able to use `just run` to upload your code to the device. This is configured in `.cargo/config.toml` in the current directory.
