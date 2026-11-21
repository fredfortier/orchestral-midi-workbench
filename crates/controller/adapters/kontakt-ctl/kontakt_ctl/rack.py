"""Qualified Kontakt 7 classic rack replacement; no patch configuration."""
import ctypes
import time
from pathlib import Path

import pyscreeze
from PIL import ImageStat

from kontakt_ctl.capture import capture_window_image
from kontakt_ctl.input_ops import click_window_point
from kontakt_ctl.window_ops import client_point_to_screen

NAVIGATOR = Path("assets/targets/instrument-navigator.png")
REMOVE = (Path("assets/targets/remove-instrument.png"),
          Path("assets/targets/remove-instrument-dark.png"))
user32 = ctypes.windll.user32


def _locate(template, image, binary=False):
    try:
        if binary:
            # Kontakt's header texture/theme changes background pixels; the
            # stock 5px white X glyph stays invariant in this qualified layout.
            from PIL import Image
            template = Image.open(template).convert("L").point(lambda v: 255 if v > 180 else 0)
            image = image.convert("L").point(lambda v: 255 if v > 180 else 0)
        else:
            template = str(template)
        return pyscreeze.locate(template, image, grayscale=False)
    except pyscreeze.ImageNotFoundException:
        return None


def _navigator(image):
    title = _locate(NAVIGATOR, image)
    if title is None or title.left > image.width // 2:
        raise RuntimeError("Visible classic Instrument Navigator required; reconcile any preceding removal")
    return title


def _rows(image):
    title = _navigator(image)
    # Qualified 1209px classic layout: the scrollbar thumb must begin at the
    # rail's top. Never infer rack cardinality from a scrolled partial list.
    if image.width != 1209 or title.left != 14:
        raise RuntimeError("Unqualified Navigator geometry; no NKI submitted")
    edge = [image.getpixel((531, title.top + offset))[:3] for offset in (26, 27, 28)]
    if edge != [(47, 47, 47), (69, 69, 69), (71, 71, 71)]:
        raise RuntimeError("Navigator top position unconfirmed; no NKI submitted")
    start = title.top + title.height + 12
    # Qualified classic-view rows are 18 pixels; the footer begins 90 pixels
    # above the render bottom. Normalize the list to its beginning before use.
    states = []
    for y in range(start + 9, image.height - 90, 18):
        colors = [image.getpixel((title.left - 4 + x, y))[:3] for x in (0, 2, 4)]
        states.append(all(min(c) > 75 and max(c) - min(c) < 5 for c in colors))
    if len(states) < 2 or any(states[i] and not states[i-1] for i in range(1, len(states))):
        raise RuntimeError("Unqualified Navigator row layout; no NKI submitted")
    return sum(states)


def _empty(image):
    area = image.crop((round(image.width * .46), round(image.height * .095),
                       image.width - 22, round(image.height * .695)))
    stats = ImageStat.Stat(area.convert("RGB"))
    return all(15 < mean < 55 for mean in stats.mean) and max(stats.stddev) < 1


def _top(window):
    image = capture_window_image(window)
    title = _navigator(image)
    x, y = client_point_to_screen(window, title.left + 40, title.top + 35)
    root = user32.GetAncestor(window._hWnd, 2)
    if not root or user32.GetForegroundWindow() != root or not user32.SetCursorPos(x, y):
        raise RuntimeError("Owned Kontakt Navigator is not foreground")
    # Kontakt supports at most 64 rack positions; 100 wheel steps bound scrolling.
    if not user32.PostMessageW(window._hWnd, 0x020A, (120 * 100) << 16,
                              ((y & 0xffff) << 16) | (x & 0xffff)):
        raise RuntimeError("Could not normalize the owned Navigator")
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        image = capture_window_image(window)
        try:
            _rows(image)  # Includes independent scrollbar-top readback.
            return image
        except RuntimeError:
            time.sleep(.1)
    raise RuntimeError("Navigator normalization unconfirmed; no rack changed")


def prepare_empty_rack(window, config):
    """Refuse multiple instruments; replace the sole instrument without resetting a Multi."""
    image = _top(window)
    image.save(config.artifacts_dir / "rack-before-replacement.png")
    rows = _rows(image)
    if rows == 0 and _empty(image):
        return False
    if rows != 1 or _empty(image):
        raise RuntimeError(f"Expected empty or sole-instrument Kontakt rack; observed {rows} Navigator rows")
    # Restrict this stock Remove Instrument button to the first rack header.
    header = image.crop((image.width // 2, 65, image.width - 15, 145))
    button = next((box for target in REMOVE if (box := _locate(target, header, binary=True)) is not None), None)
    if button is None:
        raise RuntimeError("Sole instrument Remove button unavailable; no rack changed")
    x, y = image.width // 2 + button.left + button.width // 2, 65 + button.top + button.height // 2
    if abs(x - (image.width - 34)) > 4:
        raise RuntimeError("Remove button is outside the qualified rack header")
    root = user32.GetAncestor(window._hWnd, 2)
    if not root or user32.GetForegroundWindow() != root:
        raise RuntimeError("Owned Kontakt lost foreground before instrument replacement")
    screen_x, screen_y = client_point_to_screen(window, x, y)
    if not user32.SetCursorPos(screen_x, screen_y):
        raise RuntimeError("Cannot position cursor on owned Remove Instrument button")
    click_window_point(window._hWnd, x, y)
    deadline = time.monotonic() + config.dialog_timeout_seconds
    while time.monotonic() < deadline:
        image = capture_window_image(window)
        if _rows(image) == 0 and _empty(image):
            image.save(config.artifacts_dir / "rack-empty.png")
            return True
        time.sleep(.2)
    raise RuntimeError("Instrument removal unconfirmed; no NKI submitted, reconcile original operation")


def verify_one_instrument(window):
    image = _top(window)
    if _rows(image) != 1 or _empty(image):
        raise RuntimeError("Exact NKI load did not yield one Navigator instrument; do not replay")
