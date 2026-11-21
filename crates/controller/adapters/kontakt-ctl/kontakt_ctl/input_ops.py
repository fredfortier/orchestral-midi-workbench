import ctypes
import time
from typing import List

user32 = ctypes.windll.user32

INPUT_MOUSE = 0
INPUT_KEYBOARD = 1
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_UNICODE = 0x0004
WM_KEYDOWN = 0x0100
WM_KEYUP = 0x0101
WM_LBUTTONDOWN = 0x0201
WM_LBUTTONUP = 0x0202

VK_SHIFT = 0x10
VK_CONTROL = 0x11
VK_MENU = 0x12

KEY_NAME_TO_VK = {
    "alt": VK_MENU,
    "backspace": 0x08,
    "ctrl": VK_CONTROL,
    "delete": 0x2E,
    "down": 0x28,
    "esc": 0x1B,
    "escape": 0x1B,
    "end": 0x23,
    "enter": 0x0D,
    "home": 0x24,
    "left": 0x25,
    "right": 0x27,
    "shift": VK_SHIFT,
    "tab": 0x09,
    "up": 0x26,
    "y": 0x59,
}


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", ctypes.c_long),
        ("dy", ctypes.c_long),
        ("mouseData", ctypes.c_ulong),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.c_void_p),
    ]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", ctypes.c_ushort),
        ("wScan", ctypes.c_ushort),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.c_void_p),
    ]


class INPUT_UNION(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT)]


class INPUT(ctypes.Structure):
    _fields_ = [("type", ctypes.c_ulong), ("union", INPUT_UNION)]


def click_screen_point(x: int, y: int) -> None:
    user32.SetCursorPos(x, y)
    send_inputs(
        INPUT(
            type=INPUT_MOUSE,
            union=INPUT_UNION(mi=MOUSEINPUT(0, 0, 0, MOUSEEVENTF_LEFTDOWN, 0, None)),
        ),
        INPUT(
            type=INPUT_MOUSE,
            union=INPUT_UNION(mi=MOUSEINPUT(0, 0, 0, MOUSEEVENTF_LEFTUP, 0, None)),
        ),
    )


def click_window_point(hwnd: int, x: int, y: int) -> None:
    lparam = (y << 16) | x
    if not user32.PostMessageW(hwnd, WM_LBUTTONDOWN, 1, lparam):
        raise RuntimeError("Could not send mouse-down to Kontakt window")
    if not user32.PostMessageW(hwnd, WM_LBUTTONUP, 0, lparam):
        raise RuntimeError("Could not send mouse-up to Kontakt window")


def post_key(hwnd: int, key_name: str, pause_seconds: float = 0.0) -> None:
    vk = resolve_vk(key_name)
    if not user32.PostMessageW(hwnd, WM_KEYDOWN, vk, 0):
        raise RuntimeError(f"Could not send {key_name!r} key-down to Kontakt window")
    if not user32.PostMessageW(hwnd, WM_KEYUP, vk, 0):
        raise RuntimeError(f"Could not send {key_name!r} key-up to Kontakt window")
    if pause_seconds > 0:
        time.sleep(pause_seconds)


def press_key(key_name: str, pause_seconds: float = 0.0) -> None:
    vk = resolve_vk(key_name)
    key_down(vk)
    key_up(vk)
    if pause_seconds > 0:
        time.sleep(pause_seconds)


def hotkey(*key_names: str) -> None:
    vk_codes = [resolve_vk(name) for name in key_names]
    for vk in vk_codes:
        key_down(vk)
    for vk in reversed(vk_codes):
        key_up(vk)


def type_text(value: str) -> None:
    encoded = value.encode("utf-16-le")
    code_units = [
        int.from_bytes(encoded[index : index + 2], "little") for index in range(0, len(encoded), 2)
    ]
    events: List[INPUT] = []
    for code_unit in code_units:
        events.extend(
            [
                INPUT(
                    type=INPUT_KEYBOARD,
                    union=INPUT_UNION(ki=KEYBDINPUT(0, code_unit, KEYEVENTF_UNICODE, 0, None)),
                ),
                INPUT(
                    type=INPUT_KEYBOARD,
                    union=INPUT_UNION(
                        ki=KEYBDINPUT(
                            0,
                            code_unit,
                            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            0,
                            None,
                        )
                    ),
                ),
            ]
        )
    send_inputs(*events)


def key_down(vk: int) -> None:
    send_inputs(
        INPUT(
            type=INPUT_KEYBOARD,
            union=INPUT_UNION(ki=KEYBDINPUT(vk, 0, 0, 0, None)),
        )
    )


def key_up(vk: int) -> None:
    send_inputs(
        INPUT(
            type=INPUT_KEYBOARD,
            union=INPUT_UNION(ki=KEYBDINPUT(vk, 0, KEYEVENTF_KEYUP, 0, None)),
        )
    )


def send_inputs(*items: INPUT) -> None:
    payload = (INPUT * len(items))(*items)
    sent = user32.SendInput(len(items), ctypes.byref(payload), ctypes.sizeof(INPUT))
    if sent != len(items):
        raise RuntimeError(f"SendInput sent {sent} of {len(items)} events")


def resolve_vk(key_name: str) -> int:
    if key_name in KEY_NAME_TO_VK:
        return KEY_NAME_TO_VK[key_name]
    if len(key_name) == 1:
        key_info = user32.VkKeyScanW(ord(key_name))
        if key_info == -1:
            raise KeyError(key_name)
        return key_info & 0xFF
    raise KeyError(key_name)
