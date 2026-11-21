import ctypes
import time
from dataclasses import dataclass

import pyautogui
import pygetwindow as gw
from loguru import logger

from kontakt_ctl.config import LaunchConfig

pyautogui.FAILSAFE = True
user32 = ctypes.windll.user32

TV_FIRST = 0x1100
TVM_GETNEXTITEM = TV_FIRST + 10


class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]


class RECT(ctypes.Structure):
    _fields_ = [
        ("left", ctypes.c_long),
        ("top", ctypes.c_long),
        ("right", ctypes.c_long),
        ("bottom", ctypes.c_long),
    ]


@dataclass(frozen=True)
class WindowHandleProxy:
    _hWnd: int
    title: str


def wait_for_kontakt_window(config: LaunchConfig):
    deadline = time.time() + config.window_timeout_seconds
    while time.time() < deadline:
        window = find_kontakt_window(config.window_title_contains)
        if window is not None:
            return window
        time.sleep(0.5)
    raise RuntimeError(
        f"Timed out waiting for a hosted Kontakt window containing "
        f"{config.window_title_contains!r}"
    )


def find_kontakt_window(title_contains: str):
    needle = title_contains.casefold()
    hosted_matches = []
    for window in gw.getAllWindows():
        if needle in window.title.casefold():
            return window
        if window.title.casefold() == "yabridge plugin" and any(
            needle in text.casefold() for text in iter_child_window_texts(window._hWnd)
        ):
            hosted_matches.append(window)
    if len(hosted_matches) > 1:
        raise RuntimeError(
            f"Multiple hosted Kontakt windows matched {title_contains!r}; close all but one."
        )
    if hosted_matches:
        return hosted_matches[0]
    return None


def find_window_title(title_contains: str):
    needle = title_contains.casefold()
    for window in gw.getAllWindows():
        if needle in window.title.casefold():
            return window
    return None


def wait_for_window_title(title_contains: str, timeout_seconds: int):
    deadline = time.time() + timeout_seconds
    needle = title_contains.casefold()
    while time.time() < deadline:
        titles = [title for title in gw.getAllTitles() if needle in title.casefold()]
        if titles:
            logger.info("Window titles for {!r}: {}", title_contains, titles)

        for window in gw.getAllWindows():
            if needle not in window.title.casefold():
                continue
            return window

        time.sleep(0.5)

    raise RuntimeError(f"Timed out waiting for a window containing {title_contains!r}")


def focus_and_prepare_window(window, config: LaunchConfig) -> None:
    try:
        if window.isMinimized:
            window.restore()
            time.sleep(0.5)
        window.activate()
        time.sleep(0.5)
        try:
            window.moveTo(0, 0)
            time.sleep(0.5)
        except Exception as exc:
            logger.warning("moveTo failed: {}", exc)
        try:
            window.resizeTo(
                max(window.width, config.min_width),
                max(window.height, config.min_height),
            )
            time.sleep(0.5)
        except Exception as exc:
            logger.warning("resizeTo failed: {}", exc)
    except Exception as exc:
        raise RuntimeError(f"Window activation failed for {window.title!r}: {exc}") from exc


def activate_window(window) -> None:
    try:
        if window.isMinimized:
            window.restore()
            time.sleep(0.5)
        window.activate()
        time.sleep(0.5)
    except Exception as exc:
        raise RuntimeError(f"Window activation failed for {window.title!r}: {exc}") from exc


def client_point_to_screen(window, client_x: int, client_y: int) -> tuple[int, int]:
    point = POINT(client_x, client_y)
    if not user32.ClientToScreen(window._hWnd, ctypes.byref(point)):
        raise RuntimeError(f"ClientToScreen failed for {window.title!r}")
    return point.x, point.y


def get_client_size(window) -> tuple[int, int]:
    rect = RECT()
    if not user32.GetClientRect(window._hWnd, ctypes.byref(rect)):
        raise RuntimeError(f"GetClientRect failed for {window.title!r}")
    return rect.right - rect.left, rect.bottom - rect.top


def get_window_process_id(window) -> int:
    process_id = ctypes.c_ulong()
    user32.GetWindowThreadProcessId(window._hWnd, ctypes.byref(process_id))
    if process_id.value <= 0:
        raise RuntimeError(f"Could not resolve process id for {window.title!r}")
    return int(process_id.value)


def iter_child_window_texts(parent_hwnd: int) -> list[str]:
    texts: list[str] = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_long)
    def enum_child(hwnd, lparam):
        text = ctypes.create_unicode_buffer(512)
        user32.GetWindowTextW(hwnd, text, 511)
        if text.value:
            texts.append(text.value)
        return True

    user32.EnumChildWindows(parent_hwnd, enum_child, 0)
    return texts


def find_named_child_window(parent_window, title_contains: str):
    needle = title_contains.casefold()
    matches: list[int] = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_long)
    def enum_child(hwnd, lparam):
        text = ctypes.create_unicode_buffer(512)
        user32.GetWindowTextW(hwnd, text, 511)
        if needle in text.value.casefold():
            matches.append(int(hwnd))
        return True

    user32.EnumChildWindows(parent_window._hWnd, enum_child, 0)
    if not matches:
        return None
    return WindowHandleProxy(_hWnd=matches[0], title=title_contains)
