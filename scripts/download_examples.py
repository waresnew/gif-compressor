import logging
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests
from github import Github
from github.GitReleaseAsset import GitReleaseAsset

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


EXAMPLES_PATH = Path.cwd() / "examples"


def main() -> None:
    download_gifs()


def download_gifs() -> None:
    EXAMPLES_PATH.mkdir(parents=True)
    logger.info("Downloading GIFs from Github")
    gh = Github()
    repo = gh.get_repo("waresnew/gif-compressor")
    assets = repo.get_release("examples").assets

    def download_gif(asset: GitReleaseAsset) -> None:
        logger.info("Downloading %s", asset.name)
        with (EXAMPLES_PATH / asset.name).open("wb") as file:
            file.write(
                requests.get(asset.browser_download_url, timeout=10).content,
            )

    with ThreadPoolExecutor(4) as executor:
        executor.map(download_gif, assets)

    logger.info("Done")


if __name__ == "__main__":
    main()
