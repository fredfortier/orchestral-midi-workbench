import time

from loguru import logger
from PIL import ImageGrab, ImageStat

from kontakt_ctl.config import LaunchConfig


def capture_window_image(window):
    return ImageGrab.grab(window=window._hWnd)


def wait_for_window_render(window, config: LaunchConfig):
    deadline = time.time() + config.render_timeout_seconds
    last_mean = 0.0

    while time.time() < deadline:
        try:
            image = capture_window_image(window)
        except OSError as error:
            logger.warning("Kontakt window capture is not ready: {}", error)
            time.sleep(config.render_poll_seconds)
            continue
        gray = image.convert("L")
        last_mean = ImageStat.Stat(gray).mean[0]
        extrema = gray.getextrema()
        logger.info("WindowRender mean={} extrema={}", round(last_mean, 2), extrema)
        if extrema != (0, 0) and last_mean >= config.render_ready_min_mean:
            return image
        time.sleep(config.render_poll_seconds)

    raise RuntimeError(
        "Timed out waiting for the Kontakt window to render: " f"last_mean={last_mean:.2f}"
    )


def capture_window_artifacts(window, config: LaunchConfig) -> tuple[int, int]:
    config.artifacts_dir.mkdir(exist_ok=True)
    full = wait_for_window_render(window, config)
    width, height = full.size
    logger.info("WindowCapture={}x{}", width, height)
    full.save(config.fullshot_path)

    crop_w = min(config.top_left_width, width)
    crop_h = min(config.top_left_height, height)
    top_left = full.crop((0, 0, crop_w, crop_h))
    top_left.save(config.top_left_path)
    logger.info("Saved {}", config.fullshot_path)
    logger.info("Saved {}", config.top_left_path)
    return width, height
