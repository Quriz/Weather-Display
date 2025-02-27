# Weather Display

<p align="center">
	<image src="https://github.com/user-attachments/assets/e84c1bba-f061-47e3-adb1-d1f186970022" alt="Weather Display Output" width="60%">
</p>

An e-ink weather display based on [Harry Stern's Halldisplay](https://github.com/user-attachments/assets/e84c1bba-f061-47e3-adb1-d1f186970022).

## Overview

The weather display displays the following:

- Current date and time
- Current temperature in celsius
- Current humidity
- Current wind speeds
- Current weather state. 
- Temperature and precipitation forecast for the current and next three days
- Latest meme from the [KnowYourMeme](https://knowyourmeme.com/) home page

The weather location, timezone, weather state texts and more are customizable in the [config](./renderer/example-config.json).

## Architecture

This project consists of two main components:

- Weather Display: An ESP-based display that shows the current weather. Every hour, it downloads a freshly rendered image from a locally hosted WebDAV server.
- Image Renderer: A continuously running service that renders and uploads the image to the WebDAV server every hour.

## Hardware

The weather display hardware consists of the [Waveshare 7.5" e-Paper B display](https://www.waveshare.com/7.5inch-e-paper-b.htm) and the [ESP 32 e-Paper driver board](https://www.waveshare.com/e-paper-esp32-driver-board.htm).

## Project Structure

`renderer/`: Image rendering service.

`esp/`: ESP firmware that downloads and displays the rendered image.

`brightsky/`: [Bright Sky](https://brightsky.dev/) API wrapper to get the weather forecasts. Used in the `renderer` package.

`knowyourmeme/`: Library for getting the KnowYourMeme feed via web scraping. Used in the `renderer` package.

`build-utils/`: Small helper library for generating the config from `config.json` for the `esp` and `renderer` packages.

## GitHub Actions

There is a CI workflow for linting and testing the code.
There are also two actions for building `esp` and `renderer`.

All workflows need your config for `esp` and `renderer` to compile the project.
Here is how you can add them:

1. Navigate to: Repository Settings → Secrets and variables → Actions
2. Create a secret named `RENDERER_CONFIG_JSON` and paste the contents of your [`config.json`](./renderer/example-config.json) from `renderer` in it.
3. Create a secret named `ESP_CONFIG_JSON` and paste the contents of your [`config.json`](./renderer/example-config.json) from `esp` in it.

### Build Renderer Docker Image

There is a [workflow](./.github/workflows/build-renderer-docker.yml) to build and publish a `renderer` docker image to the GitHub Container Registry. In a private repository, only you have access to the image in the registry.
I use it to self-host the renderer on my [TrueNAS Scale](https://www.truenas.com/truenas-scale/) system.

### Build ESP Binary

There is a [workflow](./.github/workflows/build-esp.yml) to build the `esp` binary.
I use it so that I don't need to have a devcontainer or WSL distro to build it.
Then you can download the artifact and flash it to your ESP.
