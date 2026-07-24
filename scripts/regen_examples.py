import logging
import subprocess
import sys

from download_examples import EXAMPLES_PATH

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def main() -> None:
    confirm = input(
        f"Warning: this will overwrite the output files in {EXAMPLES_PATH}. Continue? (y/n)",  # noqa: E501
    )
    if confirm == "y":
        for file in EXAMPLES_PATH.iterdir():
            if not file.name.endswith("_output.gif"):
                logger.info("Running on %s", file.name)

                subprocess.run(  # noqa: S603
                    [
                        "cargo",
                        "run",
                        "--release",
                        "--",
                        "-i",
                        file,
                        "-o",
                        EXAMPLES_PATH / f"{file.name.removesuffix('.gif')}_output.gif",
                    ],
                    check=True,
                )
    else:
        logger.info("Aborting")
        sys.exit()


if __name__ == "__main__":
    main()
