import sys

from loguru import logger


def configure_logging(verbose: bool = False) -> None:
    logger.remove()
    logger.add(
        sys.stderr,
        level="DEBUG" if verbose else "INFO",
        format="<level>{level: <8}</level> {message}",
    )
