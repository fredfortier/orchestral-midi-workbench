"""PID-scoped, bounded Win32 operations for standalone batch resave.

Tree messages >= WM_USER require remote memory (SendMessageTimeoutW docs).
The folder edit is not selection evidence: read TVM_GETNEXTITEM/TVM_GETITEMW
ancestry after BFFM_SETSELECTIONW and again immediately before IDOK.
"""

import ctypes as ct
import msvcrt
import os
import re
import time
from contextlib import ExitStack, contextmanager
from ctypes import wintypes as wt
from dataclasses import dataclass
from pathlib import PureWindowsPath

import pygetwindow as gw
from loguru import logger

u = ct.WinDLL("user32", use_last_error=True)
k = ct.WinDLL("kernel32", use_last_error=True)
CALLBACK = ct.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
SIZE = ct.c_size_t


# Declare every pointer-sized boundary; ctypes' default int truncates x64 handles.
def _signature(dll, name, result, *args):
    fn = getattr(dll, name)
    fn.restype, fn.argtypes = result, list(args)
    return fn


_signature(
    u,
    "SendMessageTimeoutW",
    wt.LPARAM,
    wt.HWND,
    wt.UINT,
    wt.WPARAM,
    wt.LPARAM,
    wt.UINT,
    wt.UINT,
    ct.POINTER(SIZE),
)
_signature(u, "PostMessageW", wt.BOOL, wt.HWND, wt.UINT, wt.WPARAM, wt.LPARAM)
_signature(u, "GetDlgItem", wt.HWND, wt.HWND, ct.c_int)
_signature(u, "GetWindow", wt.HWND, wt.HWND, wt.UINT)
_signature(u, "GetWindowThreadProcessId", wt.DWORD, wt.HWND, ct.POINTER(wt.DWORD))
_signature(u, "EnumWindows", wt.BOOL, CALLBACK, wt.LPARAM)
_signature(u, "EnumChildWindows", wt.BOOL, wt.HWND, CALLBACK, wt.LPARAM)
_signature(u, "GetWindowTextW", ct.c_int, wt.HWND, wt.LPWSTR, ct.c_int)
_signature(u, "GetClassNameW", ct.c_int, wt.HWND, wt.LPWSTR, ct.c_int)
for name in ("IsWindow", "IsWindowVisible", "IsWindowEnabled"):
    _signature(u, name, wt.BOOL, wt.HWND)
_signature(k, "OpenProcess", wt.HANDLE, wt.DWORD, wt.BOOL, wt.DWORD)
_signature(k, "CloseHandle", wt.BOOL, wt.HANDLE)
_signature(
    k, "VirtualAllocEx", ct.c_void_p, wt.HANDLE, ct.c_void_p, SIZE, wt.DWORD, wt.DWORD
)
_signature(k, "VirtualFreeEx", wt.BOOL, wt.HANDLE, ct.c_void_p, SIZE, wt.DWORD)
for name in ("ReadProcessMemory", "WriteProcessMemory"):
    _signature(
        k, name, wt.BOOL, wt.HANDLE, ct.c_void_p, ct.c_void_p, SIZE, ct.POINTER(SIZE)
    )
_signature(
    k,
    "CreateFileW",
    wt.HANDLE,
    wt.LPCWSTR,
    wt.DWORD,
    wt.DWORD,
    ct.c_void_p,
    wt.DWORD,
    wt.DWORD,
    wt.HANDLE,
)
_signature(k, "IsWow64Process", wt.BOOL, wt.HANDLE, ct.POINTER(wt.BOOL))
_signature(k, "CreateMutexW", wt.HANDLE, ct.c_void_p, wt.BOOL, wt.LPCWSTR)
_signature(k, "WaitForSingleObject", wt.DWORD, wt.HANDLE, wt.DWORD)
_signature(k, "ReleaseMutex", wt.BOOL, wt.HANDLE)


@contextmanager
def batch_lock():
    mutex = k.CreateMutexW(None, False, "Local\\kontakt-ctl.batch-resave")
    if not mutex:
        raise OSError(ct.get_last_error(), "Could not create batch-resave mutex")
    acquired = False
    try:
        status = k.WaitForSingleObject(mutex, 0)
        if status not in (0, 0x80):
            raise RuntimeError(
                "Another batch-resave transaction is running; no action taken"
            )
        acquired = True
        yield
    finally:
        if acquired:
            k.ReleaseMutex(mutex)
        k.CloseHandle(mutex)


@dataclass(frozen=True)
class NativeWindow:
    hwnd: int
    title: str
    class_name: str


def window_text(hwnd: int) -> str:
    text = ct.create_unicode_buffer(4096)
    send(hwnd, 0xD, len(text), ct.addressof(text))
    return text.value


def process_id(hwnd: int) -> int:
    pid = wt.DWORD()
    if not u.GetWindowThreadProcessId(hwnd, ct.byref(pid)):
        raise RuntimeError("Window disappeared while resolving its PID")
    return pid.value


def assert_owner(hwnd: int, pid: int) -> None:
    if process_id(hwnd) != pid:
        raise RuntimeError("Window ownership changed; refused native operation")


def enumerate_windows(pid: int, parent: int | None = None) -> list[NativeWindow]:
    found = []
    errors = []

    @CALLBACK
    def collect(hwnd, _):
        owner = wt.DWORD()
        if not u.GetWindowThreadProcessId(hwnd, ct.byref(owner)):
            if u.IsWindow(hwnd):
                errors.append("Could not identify a live enumerated window")
                return False
            return True  # Destroyed during enumeration; no longer a dialog.
        if owner.value == pid and u.IsWindowVisible(hwnd):
            text, cls = ct.create_unicode_buffer(1024), ct.create_unicode_buffer(256)
            u.GetWindowTextW(hwnd, text, len(text))
            if not u.GetClassNameW(hwnd, cls, len(cls)):
                errors.append("Could not identify an owned window class")
                return False
            found.append(NativeWindow(int(hwnd), text.value, cls.value))
        return True

    if parent is None:
        if not u.EnumWindows(collect, 0):
            raise RuntimeError(
                "Native top-level window enumeration failed; state unproven"
            )
    else:
        assert_owner(parent, pid)
        # Microsoft documents EnumChildWindows' return value as unused. Check
        # callback errors and parent lifetime/ownership instead.
        u.EnumChildWindows(parent, collect, 0)
        assert_owner(parent, pid)
    if errors:
        raise RuntimeError("; ".join(errors))
    return found


def send(
    hwnd: int, message: int, wparam: int = 0, lparam: int = 0, timeout_ms: int = 2000
) -> int:
    result = SIZE()
    ct.set_last_error(0)
    if not u.SendMessageTimeoutW(
        hwnd, message, wparam, lparam, 0x22, timeout_ms, ct.byref(result)
    ):
        if ct.get_last_error() in (0, 1460):
            raise TimeoutError(f"Native message {message:#x} timed out for HWND {hwnd}")
        raise RuntimeError(
            f"Native message {message:#x} failed/timed out for HWND {hwnd}"
        )
    return result.value


def post(hwnd: int, message: int, wparam: int = 0, lparam: int = 0) -> None:
    if not u.PostMessageW(hwnd, message, wparam, lparam):
        raise RuntimeError(f"Could not post native message {message:#x}")


def wait_window(
    pid: int, titles: tuple[str, ...], timeout: float, process=None
) -> NativeWindow:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process is not None and process.poll() is not None:
            raise RuntimeError(f"Kontakt exited unexpectedly: {process.returncode}")
        matches = [w for w in enumerate_windows(pid) if w.title in titles]
        if len(matches) > 1:
            raise RuntimeError(f"Ambiguous owned windows: {matches}")
        if matches:
            return matches[0]
        time.sleep(0.1)
    raise TimeoutError(f"No owned window {titles!r} for Kontakt PID {pid}")


@contextmanager
def remote_buffer(pid: int, size: int):
    process = k.OpenProcess(0x438, False, pid)
    if not process:
        raise OSError(ct.get_last_error(), "OpenProcess failed")
    address = None
    completed = False
    try:
        wow = wt.BOOL()
        if (
            ct.sizeof(ct.c_void_p) != 8
            or not k.IsWow64Process(process, ct.byref(wow))
            or wow.value
        ):
            raise RuntimeError(
                "Batch resave requires x64 Windows Python and x64 Kontakt"
            )
        address = k.VirtualAllocEx(process, None, size, 0x3000, 0x04)
        if not address:
            raise OSError(ct.get_last_error(), "VirtualAllocEx failed")
        yield process, address
        completed = True
    finally:
        try:
            # A timed-out message may still reference this allocation. On error
            # let owned-process cleanup reclaim it instead of freeing it early.
            if completed and not k.VirtualFreeEx(process, address, 0, 0x8000):
                raise OSError(ct.get_last_error(), "VirtualFreeEx failed")
        finally:
            k.CloseHandle(process)


def transfer(process, address: int, value, *, write: bool) -> None:
    count = SIZE()
    fn = k.WriteProcessMemory if write else k.ReadProcessMemory
    if not fn(
        process, address, ct.byref(value), ct.sizeof(value), ct.byref(count)
    ) or count.value != ct.sizeof(value):
        raise OSError(ct.get_last_error(), "Incomplete remote memory transfer")


class TreeItem(ct.Structure):
    _fields_ = [
        ("mask", wt.UINT),
        ("item", ct.c_void_p),
        ("state", wt.UINT),
        ("state_mask", wt.UINT),
        ("text", ct.c_void_p),
        ("text_max", ct.c_int),
        ("image", ct.c_int),
        ("selected_image", ct.c_int),
        ("children", ct.c_int),
        ("param", wt.LPARAM),
    ]


def selected_folder(dialog: int) -> PureWindowsPath:
    pid = process_id(dialog)
    trees = [
        w for w in enumerate_windows(pid, dialog) if w.class_name == "SysTreeView32"
    ]
    if len(trees) != 1:
        raise RuntimeError("Folder picker must expose exactly one native tree")
    tree = trees[0].hwnd
    item = send(tree, 0x110A, 9)  # TVM_GETNEXTITEM, TVGN_CARET
    parts = []
    seen = set()
    with remote_buffer(pid, 16384) as (process, address):
        while item:
            if item in seen or len(parts) >= 64:
                raise RuntimeError("Invalid folder tree ancestry")
            seen.add(item)
            text = ct.create_unicode_buffer(4096)
            info = TreeItem(mask=1, item=item, text=address + 1024, text_max=len(text))
            transfer(process, address, info, write=True)
            if not send(tree, 0x113E, 0, address):  # TVM_GETITEMW
                raise RuntimeError("Could not read selected native folder")
            transfer(process, address, info, write=False)
            if info.text != address + 1024:
                raise RuntimeError("Unexpected tree text pointer")
            transfer(process, info.text, text, write=False)
            label = text.value
            drive = re.fullmatch(r"(?:.*\()?([A-Za-z]:)\)?", label)
            if drive:
                return PureWindowsPath(drive.group(1) + "\\", *reversed(parts))
            if not label or label in (".", "..") or any(c in label for c in "\\/:"):
                raise RuntimeError(f"Invalid folder tree label {label!r}")
            parts.append(label)
            item = send(tree, 0x110A, 3, item)  # TVGN_PARENT
    raise RuntimeError("Selected folder has no filesystem drive ancestor")


def verify_folder(dialog: int, target: PureWindowsPath) -> None:
    observed = selected_folder(dialog)
    if observed != target:
        raise RuntimeError(
            f"Native folder mismatch: requested {target}, selected {observed}; refused OK"
        )


def choose_folder(dialog: int, target: PureWindowsPath, pid: int) -> None:
    assert_owner(dialog, pid)
    value = ct.create_unicode_buffer(str(target))
    with remote_buffer(process_id(dialog), ct.sizeof(value)) as (process, address):
        transfer(process, address, value, write=True)
        send(dialog, 0x467, 1, address, timeout_ms=30000)  # BFFM_SETSELECTIONW
    verify_folder(dialog, target)
    button = u.GetDlgItem(dialog, 1)
    if not button or not u.IsWindowEnabled(button):
        raise RuntimeError("Folder picker OK is unavailable")
    verify_folder(dialog, target)
    assert_owner(button, pid)
    logger.info("Verified native folder selection: {}", target)
    post(button, 0xF5)  # BM_CLICK; do not block while Kontakt resaves


def click_button(dialog: int, button_id: int) -> None:
    button = u.GetDlgItem(dialog, button_id)
    if not button or not u.IsWindowEnabled(button):
        raise RuntimeError(f"Owned dialog button {button_id} unavailable")
    post(button, 0xF5)


def main_is_idle(pid: int, main: int) -> bool:
    if not u.IsWindow(main):
        raise RuntimeError("Owned Kontakt main window disappeared")
    assert_owner(main, pid)
    windows = enumerate_windows(pid)
    if any(w.hwnd != main for w in windows):
        return False
    if not u.IsWindowEnabled(main):
        return False
    try:
        send(main, 0, timeout_ms=250)  # Busy processing is not terminal failure.
    except TimeoutError:
        return False
    return True


def files_are_closed(paths) -> bool:
    for path in paths:
        handle = k.CreateFileW(str(path), 0x80000000, 0, None, 3, 0, None)
        if handle == ct.c_void_p(-1).value:
            error = ct.get_last_error()
            if error in (32, 33):
                return False
            raise OSError(error, f"Could not exclusively read resaved file {path}")
        k.CloseHandle(handle)
    return True


@contextmanager
def exclusive_files(paths):
    """Hold deny-write/delete handles throughout receipt hashing."""
    with ExitStack() as stack:
        files = {}
        for path in sorted(paths):
            handle = k.CreateFileW(str(path), 0x80000000, 0, None, 3, 0, None)
            if handle == ct.c_void_p(-1).value:
                raise OSError(
                    ct.get_last_error(),
                    f"Cannot lock instrument for verification: {path}",
                )
            try:
                fd = msvcrt.open_osfhandle(handle, os.O_RDONLY | os.O_BINARY)
            except BaseException:
                k.CloseHandle(handle)
                raise
            files[path] = stack.enter_context(os.fdopen(fd, "rb"))
        yield files


def as_proxy(window: NativeWindow):
    return gw.Win32Window(window.hwnd)
