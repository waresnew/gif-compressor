import logging
import shutil
import subprocess
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory

import requests
from github import Github
from github.GitReleaseAsset import GitReleaseAsset


class RequirementsError(BaseException):
    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(self.message)


@dataclass
class GifExample:
    input: str
    input_size: float
    gifsicle_size: float
    output: str
    output_size: float


logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

TEMPLATE_FILE = "README.tmpl.md"
CLI_USAGE_MARKER = "{{ cli_usage }}"
GIF_BENCH_MARKER = "{{ gif_bench }}"
OUTPUT_FILE = "README.md"


def main() -> None:
    try:
        check_requirements()
    except RequirementsError:
        logger.exception("Requirements check failed")

    output = None
    with Path.cwd() / TEMPLATE_FILE as template:
        output = template.read_text()
    logger.info("Generating CLI help message")
    output = output.replace(CLI_USAGE_MARKER, gen_cli_usage())
    logger.info("Generating GIF examples")
    output = output.replace(GIF_BENCH_MARKER, gen_gif_bench())
    logger.info("Writing to %s", Path.cwd() / OUTPUT_FILE)
    with Path.cwd() / OUTPUT_FILE as file:
        file.write_text(output)
    logger.info("Done")


def check_requirements() -> None:
    if shutil.which("gifsicle") is None:
        err = "gifsicle was not found in PATH"
        raise RequirementsError(err)
    if shutil.which("cargo") is None:
        err = "cargo was not found in PATH"
        raise RequirementsError(err)


def gen_gif_bench() -> str:
    gifsicle_args = ["-O3", "--lossy=200"]

    def download_gifs(tmp_dir: str) -> list[GitReleaseAsset]:
        logger.info("Downloading GIFs from Github")
        gh = Github()
        repo = gh.get_repo("waresnew/gif-compressor")
        assets = repo.get_release("examples").assets
        ret = []
        for asset in assets:
            logger.info("Downloading %s", asset.name)
            with Path(tmp_dir) / asset.name as file:
                file.write_bytes(
                    requests.get(asset.browser_download_url, timeout=10).content,
                )
                if not asset.name.endswith("_output.gif"):
                    ret.append(asset)
        return ret

    def run_gifsicle(inputs: Iterable[str], tmp_dir: str) -> None:
        logger.info("Running gifsicle on GIFs")
        for file_name in inputs:
            logger.info("Running gifsicle on %s", file_name)
            subprocess.run(  # noqa: S603
                ["gifsicle", *gifsicle_args, file_name, "-o", f"{file_name}.gifsicle"],
                cwd=tmp_dir,
                check=True,
            )

    with TemporaryDirectory() as tmp_dir:
        input_releases = download_gifs(tmp_dir)
        run_gifsicle((x.name for x in input_releases), tmp_dir)
        rows = []
        for release in input_releases:
            input_size = (Path(tmp_dir) / release.name).stat().st_size / 1000000
            gifsicle_size = (
                Path(tmp_dir) / f"{release.name}.gifsicle"
            ).stat().st_size / 1000000
            output_size = (
                Path(tmp_dir) / f"{release.name.removesuffix('.gif') + '_output.gif'}"
            ).stat().st_size / 1000000
            gifsicle_compression = round(
                (gifsicle_size - input_size) * 100 / input_size,
            )
            output_compression = round((output_size - input_size) * 100 / input_size)
            output_url = (
                release.browser_download_url.removesuffix(".gif") + "_output.gif"
            )
            rows.append(
                f"""
<tr>
<td width="45%">{input_size:.1f} MB <br><img src="{release.browser_download_url}" alt="{release.browser_download_url}" width="100%"></td>
<td width="10%">{gifsicle_size:.1f} MB ({gifsicle_compression:+d}%)</td>
<td width="45%">{output_size:.1f} MB ({output_compression:+d}%) <br><img src="{output_url}" alt="{output_url}" width="100%"></td>
</tr>
                """,  # noqa: E501
            )

        return "\n".join(
            [
                f"""
The program output is compared to the file size of running [gifsicle](https://github.com/kohler/gifsicle) with the `{" ".join(gifsicle_args)}` arguments on the same input.

<table>
<tr>
<th width="45%">Input</th>
<th width="10%">Gifsicle</th>
<th width="45%">Program Output</th>
</tr>
        """,  # noqa: E501
                *rows,
                "</table>",
            ],
        )


def gen_cli_usage() -> str:
    output = (
        subprocess.run(
            ["cargo", "run", "--release", "--", "-h"],
            check=True,
            capture_output=True,
        )
        .stdout.decode()
        .strip()
    )
    return f"""
```
{output}
```
"""


if __name__ == "__main__":
    main()
