import time
from pathlib import Path

import pyautogui
import pygetwindow as gw
import pyscreeze
from loguru import logger

from kontakt_ctl.capture import capture_window_image
from kontakt_ctl.input_ops import click_screen_point
from kontakt_ctl.window_ops import client_point_to_screen

pyautogui.FAILSAFE = True


def ensure_images_exist(*image_paths: Path) -> None:
    for image_path in image_paths:
        if not image_path.exists():
            raise FileNotFoundError(f"Missing target image: {image_path}")


def locate_target_image_in_window(
    image_path: Path,
    window,
    *,
    grayscale: bool,
    confidence: float,
):
    locate_kwargs = {"grayscale": grayscale}
    if pyscreeze._useOpenCV:
        locate_kwargs["confidence"] = confidence

    image = capture_window_image(window)
    try:
        return pyscreeze.locate(str(image_path), image, **locate_kwargs)
    except pyscreeze.ImageNotFoundException:
        return None


def click_target_image(
    image_path: Path,
    window,
    *,
    label: str,
    timeout_seconds: float,
    grayscale: bool,
    confidence: float,
    retry_seconds: float = 0.2,
    fallback_target=None,
    fallback_ratio: tuple[float, float] | None = None,
) -> None:
    deadline = time.time() + timeout_seconds
    last_error = None

    while time.time() < deadline:
        box = locate_target_image_in_window(
            image_path,
            window,
            grayscale=grayscale,
            confidence=confidence,
        )
        if box is not None:
            point = pyautogui.center(box)
            screen_x, screen_y = client_point_to_screen(window, point.x, point.y)
            logger.info("{} at {} -> ({}, {})", label, box, screen_x, screen_y)
            click_screen_point(screen_x, screen_y)
            return

        last_error = f"{image_path} not found"
        time.sleep(retry_seconds)

    if fallback_target is not None and fallback_ratio is not None:
        logger.warning(
            "Falling back to relative coordinates for {} because image lookup failed.",
            label,
        )
        click_relative(fallback_target, fallback_ratio)
        return

    raise RuntimeError(f"Timed out locating {label}: {last_error}")


def click_relative(target_window, ratio: tuple[float, float]) -> None:
    client_width, client_height = capture_window_image(target_window).size
    x, y = client_point_to_screen(
        target_window,
        int(client_width * ratio[0]),
        int(client_height * ratio[1]),
    )
    logger.info("Clicking fallback coordinate at {}, {}", x, y)
    click_screen_point(x, y)


def find_window_with_target_image(
    image_path: Path,
    *,
    grayscale: bool,
    confidence: float,
):
    for window in gw.getAllWindows():
        if not window.title:
            continue
        try:
            if (
                locate_target_image_in_window(
                    image_path,
                    window,
                    grayscale=grayscale,
                    confidence=confidence,
                )
                is not None
            ):
                return window
        except Exception:
            continue
    return None
