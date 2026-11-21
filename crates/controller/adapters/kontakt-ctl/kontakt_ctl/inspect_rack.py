import re
from dataclasses import dataclass
from pathlib import Path

from PIL import ImageStat

from kontakt_ctl.capture import wait_for_window_render
from kontakt_ctl.config import LoadNkiConfig
from kontakt_ctl.load_nki import click_instr_nav, get_file_menu_target_window
from kontakt_ctl.window_ops import activate_window, wait_for_kontakt_window


@dataclass(frozen=True)
class InspectRackResult:
    window_title: str
    screenshot_path: Path
    navigator_path: Path
    width: int
    height: int


def inspect_running_rack(output_stem: str, config: LoadNkiConfig) -> InspectRackResult:
    if not re.fullmatch(r"[A-Za-z0-9._-]+", output_stem):
        raise ValueError("Output stem may contain only letters, digits, dot, underscore, and dash.")

    window = wait_for_kontakt_window(config)
    if window.title.casefold() != "yabridge plugin":
        raise RuntimeError("Rack inspection requires the sole yabridge-hosted Kontakt window.")

    config.artifacts_dir.mkdir(exist_ok=True)
    activate_window(window)
    render_window = get_file_menu_target_window(window, config)
    screenshot = wait_for_window_render(render_window, config)
    navigator = crop_instrument_navigator(screenshot)
    if ImageStat.Stat(navigator.convert("L")).stddev[0] < 5.0:
        click_instr_nav(window, config)
        screenshot = wait_for_window_render(render_window, config)
        navigator = crop_instrument_navigator(screenshot)
    if ImageStat.Stat(navigator.convert("L")).stddev[0] < 5.0:
        raise RuntimeError("Instrument Navigator did not open with visible rack rows.")
    screenshot_path = config.artifacts_dir / f"{output_stem}.rack.png"
    navigator_path = config.artifacts_dir / f"{output_stem}.navigator.png"
    screenshot.save(screenshot_path)
    navigator.save(navigator_path)
    return InspectRackResult(
        window_title=window.title,
        screenshot_path=screenshot_path,
        navigator_path=navigator_path,
        width=screenshot.width,
        height=screenshot.height,
    )


def crop_instrument_navigator(image):
    width, height = image.size
    return image.crop((0, int(height * 0.75), int(width * 0.32), int(height * 0.92)))
