# GIF Compressor

GIF compression tool that focuses on undoing error diffusion dithering.

## Background

GIFs are a paletted image format with 256 colours maximum per frame, which often leads to colour quantization artifacts. Error diffusion dithering is very commonly applied by default to GIFs to mitigate this.

However, dithering greatly harms compression (such as LZW, which GIFs use) due to irregular pixel patterns, leading to GIFs that barely reduce in file size from regular compression tools. By undoing this dithering, significant reductions in file size can be achieved, at the cost of some colour banding artifacts.

## How it works

A heuristic based on [this project](https://github.com/kornelski/undither) was adapted for GIFs to identify dithering patterns. After all frames are undithered, they are requantized with a newly generated colour palette to produce a valid GIF.

Below is a zoomed in sample of the dithering removal. It's not a perfect process because error diffusion dithering is lossy.
|Before (Dithered)|After (Undithered & requantized)|
| ----------- | ----------- |
|<img width="393" height="393" alt="preditherchilde" src="https://github.com/user-attachments/assets/9e258c0f-2e38-4bc0-a87f-36e0d34ed2be" />|<img width="393" height="393" alt="postditherchilde" src="https://github.com/user-attachments/assets/f9dead43-4fbc-4aca-a0ee-f4d015b940df" />|

## Usage

{{ cli_usage }}

## Showcase

{{ gif_bench }}

<!-- TODO: add build insns -->
