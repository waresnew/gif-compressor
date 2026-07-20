import logging
from pathlib import Path

import requests
from github import Github

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


OUTPUT_PATH = Path.cwd() / "examples"


def main() -> None:
    OUTPUT_PATH.mkdir(parents=True)
    download_gifs()
    logger.info("Done")


def download_gifs() -> None:
    logger.info("Downloading GIFs from Github")
    gh = Github()
    repo = gh.get_repo("waresnew/gif-compressor")
    assets = repo.get_release("examples").assets
    for asset in assets:
        logger.info("Downloading %s", asset.name)
        with OUTPUT_PATH / asset.name as file:
            file.write_bytes(
                requests.get(asset.browser_download_url, timeout=10).content,
            )


if __name__ == "__main__":
    main()
