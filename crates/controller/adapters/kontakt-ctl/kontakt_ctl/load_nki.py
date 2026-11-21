import ctypes
import ctypes.wintypes as wt
import re
import time
from dataclasses import dataclass
from pathlib import Path, PureWindowsPath

import pygetwindow as gw
from loguru import logger
from PIL import ImageChops, ImageStat

from kontakt_ctl import resave_native as native
from kontakt_ctl.rack import prepare_empty_rack, verify_one_instrument
from kontakt_ctl.capture import capture_window_image, wait_for_window_render
from kontakt_ctl.config import LoadNkiConfig
from kontakt_ctl.input_ops import (
    click_screen_point,
    click_window_point,
    hotkey,
    post_key,
    press_key,
    type_text,
)
from kontakt_ctl.launcher import launch_kontakt
from kontakt_ctl.ui_ops import click_target_image, ensure_images_exist
from kontakt_ctl.window_ops import (
    client_point_to_screen,
    find_kontakt_window,
    find_named_child_window,
    focus_and_prepare_window,
    get_client_size,
    get_window_process_id,
    wait_for_kontakt_window,
    wait_for_window_title,
)

WM_CLOSE = 0x0010
BM_CLICK = 0x00F5
FILE_NAME_EDIT_ID = 0x0480
user32 = ctypes.windll.user32


class NkiNotSubmitted(RuntimeError):
    """The owned loader failed before opening a Load dialog or submitting an NKI."""


@dataclass(frozen=True)
class LoadNkiResult:
    pid: int
    window_title: str
    requested_relative_path: str
    target_path: str
    dialog_closed: bool
    rack_visual_confirmed: bool
    rack_diff_mean: float
    navigator_confirmed: bool
    navigator_diff_mean: float
    load_confirmed: bool
    screenshot_path: str
    navigator_path: str
    rack_instrument_count: int = 0
    replaced_previous_instrument: bool = False
    observation_only: bool = False
    cancelled_pending_dialog: bool = False
    windows: list[dict] | None = None


def run_load_nki(
    requested_relative_path: str,
    config: LoadNkiConfig,
    *,
    owner_window: int | None = None,
    owner_pid: int | None = None,
    observe_only: bool = False,
    cancel_failed_dialog: bool = False,
) -> LoadNkiResult:
    target_path = build_target_nki_path(requested_relative_path)
    host_path = windows_path_to_host_path(target_path)
    if not host_path.is_file():
        raise FileNotFoundError(f"File does not exist under X:: {host_path}")

    config.artifacts_dir.mkdir(exist_ok=True)
    screenshot_path = build_screenshot_path(target_path, suffix="loaded", config=config)
    navigator_path = build_screenshot_path(target_path, suffix="navigator", config=config)

    if cancel_failed_dialog:
        dialogs=[w for w in gw.getAllWindows() if w.title == "Load" and user32.IsWindowVisible(w._hWnd)]
        if len(dialogs)!=1:
            raise RuntimeError("Expected one visible pending Load dialog; no cancellation")
        dialog=dialogs[0]
        owner=user32.GetWindow(dialog._hWnd,4)
        title=ctypes.create_unicode_buffer(1024);user32.GetWindowTextW(owner,title,len(title))
        owner_pid=wt.DWORD();user32.GetWindowThreadProcessId(owner,ctypes.byref(owner_pid))
        edit=user32.GetDlgItem(dialog._hWnd,FILE_NAME_EDIT_ID)
        text=ctypes.create_unicode_buffer(32768)
        if edit:user32.SendMessageW(edit,0x000D,len(text),ctypes.addressof(text))
        if owner != owner_window or title.value.casefold()!="yabridge plugin" or owner_pid.value!=get_window_process_id(dialog) or not text.value or PureWindowsPath(text.value) != target_path:
            raise RuntimeError("Pending Load owner/filename does not match failed transaction; no cancellation")
        cancel_open_dialog(dialog._hWnd)
        return LoadNkiResult(pid=owner_pid.value,window_title=title.value,requested_relative_path=requested_relative_path,
            target_path=str(target_path),dialog_closed=True,rack_visual_confirmed=False,rack_diff_mean=0,
            navigator_confirmed=False,navigator_diff_mean=0,load_confirmed=False,screenshot_path="",navigator_path="",
            cancelled_pending_dialog=True,windows=[{"hwnd":dialog._hWnd,"owner":owner,"filename":text.value,"cancelled":True}])

    if observe_only:
        records=[]
        for window in gw.getAllWindows():
            if window.title not in ("Load","Content Missing","Browse for Folder","yabridge plugin"):
                continue
            visible=bool(user32.IsWindowVisible(window._hWnd))
            edit=user32.GetDlgItem(window._hWnd, FILE_NAME_EDIT_ID)
            text=ctypes.create_unicode_buffer(32768)
            if edit:user32.SendMessageW(edit,0x000D,len(text),ctypes.addressof(text))
            record={"hwnd":window._hWnd,"title":window.title,"visible":visible,
                    "pid":get_window_process_id(window),"owner":user32.GetWindow(window._hWnd,4),"filename":text.value}
            if visible:
                try:
                    shot=config.artifacts_dir / f"load-observation-{window._hWnd}.png"
                    capture_window_image(window).save(shot)
                    record["screenshot"]=str(shot)
                except OSError as error:record["capture_error"]=str(error)
            records.append(record)
        return LoadNkiResult(pid=0,window_title="load transaction observation",requested_relative_path=requested_relative_path,
            target_path=str(target_path),dialog_closed=False,rack_visual_confirmed=False,rack_diff_mean=0,
            navigator_confirmed=False,navigator_diff_mean=0,load_confirmed=False,screenshot_path="",navigator_path="",
            observation_only=True,windows=records)

    if owner_window is None or not user32.IsWindowVisible(owner_window):
        raise RuntimeError("Exact adapter-opened window is absent; no NKI submitted")
    matches=[w for w in gw.getAllWindows() if w._hWnd == owner_window and w.title.casefold() == "yabridge plugin"]
    if len(matches)!=1:
        raise RuntimeError("Exact adapter-opened yabridge window is unavailable")
    kontakt_window=matches[0]
    pid=get_window_process_id(kontakt_window)
    if owner_pid is None or pid!=owner_pid:raise RuntimeError("Owned editor process changed; no NKI submitted")
    try:
        render_window = get_file_menu_target_window(kontakt_window, config)
        root = user32.GetAncestor(render_window._hWnd, 2)
        if root != owner_window:
            raise RuntimeError("Render root differs from the qualified editor")
        user32.SetForegroundWindow(root)
        if user32.GetForegroundWindow() != root:
            raise RuntimeError("Qualified editor did not acquire foreground")
        if (not user32.IsWindowVisible(owner_window)
                or get_window_process_id(kontakt_window) != owner_pid):
            raise RuntimeError("Qualified editor ownership changed during activation")
        render_window = get_file_menu_target_window(kontakt_window, config)
        if user32.GetAncestor(render_window._hWnd, 2) != owner_window:
            raise RuntimeError("Refreshed render root differs from the qualified editor")
        # RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW.
        if not user32.RedrawWindow(render_window._hWnd, None, None, 0x185):
            raise RuntimeError("Qualified Kontakt render could not be redrawn")
        before_image = wait_for_window_render(render_window, config)
    except (RuntimeError, OSError) as error:
        raise NkiNotSubmitted(str(error)) from error
    try:
        replaced_previous_instrument = prepare_empty_rack(render_window, config)
        before_image = capture_window_image(render_window)
    except (RuntimeError, OSError) as error:
        raise NkiNotSubmitted(str(error)) from error
    navigator_before = crop_navigator_region(before_image, config)
    before_image.save(build_screenshot_path(target_path, suffix="before", config=config))
    sample_root = host_path.parent
    if any(w.title == "Content Missing" for w in gw.getAllWindows()):
        raise RuntimeError("Existing Content Missing transaction requires authenticated controller recovery")
    open_headless_load_dialog(render_window, config)
    dialog = wait_for_owned_dialog(kontakt_window, config)
    edit_hwnd, open_button_hwnd = locate_open_dialog_controls(dialog._hWnd)
    if get_window_process_id(kontakt_window)!=owner_pid or not user32.IsWindowVisible(owner_window):raise RuntimeError("Editor ownership changed before submission")
    set_dialog_text(edit_hwnd, str(target_path))
    time.sleep(config.post_open_settle_seconds)
    click_button(open_button_hwnd)
    dialog_closed = wait_for_dialog_to_close(
        config, timeout_seconds=config.headless_dialog_close_timeout_seconds,
        sample_root=sample_root, dialog_hwnd=dialog._hWnd, owner_window=kontakt_window,
    )
    reject_load_error_windows(sample_root,kontakt_window)
    rack_diff_mean, render_window = wait_for_loaded_visual_change(before_image, config, sample_root=sample_root, owner_window=kontakt_window)
    rack_visual_confirmed = rack_diff_mean >= config.confirmation_diff_mean
    verify_one_instrument(render_window)
    screenshot = capture_window_image(render_window)
    navigator_after = crop_navigator_region(screenshot, config)
    navigator_diff_mean = compute_diff_mean(navigator_before, navigator_after)
    navigator_confirmed = navigator_diff_mean >= config.navigator_confirmation_diff_mean
    screenshot.save(screenshot_path)
    navigator_after.save(navigator_path)
    return LoadNkiResult(
        pid=pid,
        window_title=kontakt_window.title,
        requested_relative_path=requested_relative_path,
        target_path=str(target_path),
        dialog_closed=dialog_closed,
        rack_visual_confirmed=rack_visual_confirmed,
        rack_diff_mean=rack_diff_mean,
        navigator_confirmed=navigator_confirmed,
        navigator_diff_mean=navigator_diff_mean,
        load_confirmed=(dialog_closed and rack_visual_confirmed and navigator_confirmed),
        rack_instrument_count=1,
        replaced_previous_instrument=replaced_previous_instrument,
        screenshot_path=str(screenshot_path),
        navigator_path=str(navigator_path),
    )

@dataclass(frozen=True)
class WindowInventory:
    windows: list[dict]


def visible_host_windows() -> WindowInventory:
    return WindowInventory([
        {"hwnd":window._hWnd,"pid":get_window_process_id(window)}
        for window in gw.getAllWindows()
        if window.title.casefold()=="yabridge plugin" and user32.IsWindowVisible(window._hWnd)
    ])


def wait_for_owned_dialog(owner, config):
    deadline=time.monotonic()+config.dialog_timeout_seconds
    while time.monotonic()<deadline:
        matches=[window for window in gw.getAllWindows()
                 if window.title==config.load_dialog_title and user32.IsWindowVisible(window._hWnd)
                 and user32.GetWindow(window._hWnd,4)==owner._hWnd
                 and get_window_process_id(window)==get_window_process_id(owner)]
        if len(matches)>1:raise RuntimeError("Ambiguous Load dialogs for exact owner; no submission")
        if matches:return matches[0]
        time.sleep(config.load_poll_seconds)
    raise RuntimeError("Exact owner's Load dialog did not appear; no submission")


def open_headless_load_dialog(window, config: LoadNkiConfig) -> None:
    post_key(window._hWnd, "escape", pause_seconds=0.2)
    post_key(window._hWnd, "escape", pause_seconds=0.2)
    width, height = get_client_size(window)
    x, y = round(width * config.file_menu_x_ratio), round(height * config.file_menu_y_ratio)
    # Qt consults the real cursor as well as posted mouse coordinates. Keep both
    # on the owned render window before opening its menu (not the previous dialog).
    root = user32.GetAncestor(window._hWnd, 2)
    point = wt.POINT(x, y)
    if not root or not user32.SetForegroundWindow(root) or not user32.ClientToScreen(window._hWnd, ctypes.byref(point)):
        raise RuntimeError("Cannot activate the owned Kontakt menu window; no NKI submitted")
    if not user32.SetCursorPos(point.x, point.y):
        raise RuntimeError("Cannot position cursor on owned Kontakt menu; no NKI submitted")
    user32.PostMessageW(window._hWnd, 0x200, 0, (y << 16) | x)
    click_window_point(window._hWnd, x, y)
    time.sleep(0.35)
    for _ in range(config.load_menu_down_presses):
        post_key(window._hWnd, "down", pause_seconds=config.menu_key_pause_seconds)
    post_key(window._hWnd, "enter", pause_seconds=0.3)


def reject_load_error_windows(sample_root=None, owner_window=None) -> None:
    if sample_root is not None and any(w.title == "Content Missing" for w in gw.getAllWindows()):
        window = owner_window
        if window is None:
            raise RuntimeError("Owned Kontakt window unavailable during pending sample search")
        render = find_named_child_window(window, "Kontakt") if window.title.casefold() == "yabridge plugin" else window
        if render is None:
            raise RuntimeError("Owned Kontakt render unavailable during pending sample search")
        resolve_content(sample_root, render._hWnd)
    error_titles = ("content missing", "files could not be found", "demo timeout")
    titles = [window.title for window in gw.getAllWindows() if window.title]
    matches = [
        title for title in titles if any(value in title.casefold() for value in error_titles)
    ]
    if matches:
        raise RuntimeError(f"Kontakt reported a load error: {matches}")


def get_file_menu_target_window(window, config: LoadNkiConfig):
    if window.title.casefold() != "yabridge plugin":
        return window
    child_window = find_named_child_window(window, config.window_title_contains)
    if child_window is None:
        raise RuntimeError("Could not find the embedded Kontakt child window inside yabridge.")
    return child_window


def click_file_menu(window, config: LoadNkiConfig) -> None:
    click_target_image(
        config.file_menu_image,
        window,
        label="FILE menu",
        timeout_seconds=config.menu_image_timeout_seconds,
        grayscale=True,
        confidence=0.9,
    )


def build_target_nki_path(requested_relative_path: str) -> PureWindowsPath:
    value = requested_relative_path.strip()
    if not value:
        raise ValueError("NKI path must not be empty.")

    relative_path = PureWindowsPath(value)
    if relative_path.is_absolute() or relative_path.drive:
        raise ValueError("NKI path must be relative to X:, not an absolute path.")
    if ".." in relative_path.parts:
        raise ValueError("NKI path must not contain '..'.")
    if relative_path.suffix.lower() != ".nki":
        raise ValueError("NKI path must end with .nki.")

    return PureWindowsPath("X:/") / relative_path


def windows_path_to_host_path(path: PureWindowsPath) -> Path:
    if path.drive.upper() != "X:":
        raise ValueError(f"Expected an X: path, got {path}")
    return Path(str(path))


def build_screenshot_path(target_path: PureWindowsPath, suffix: str, config: LoadNkiConfig) -> Path:
    safe_name = re.sub(r"[^A-Za-z0-9._-]+", "_", target_path.stem).strip("_") or "kontakt_loaded"
    return config.artifacts_dir / f"{safe_name}.{suffix}.png"


def locate_open_dialog_controls(dialog_hwnd: int) -> tuple[int, int]:
    edit_hwnd = user32.GetDlgItem(dialog_hwnd, FILE_NAME_EDIT_ID)
    if edit_hwnd == 0:
        raise RuntimeError("Native Load dialog did not expose its file-name Edit control.")
    edit_class = ctypes.create_unicode_buffer(256)
    user32.GetClassNameW(edit_hwnd, edit_class, 255)
    if edit_class.value != "Edit":
        raise RuntimeError(
            f"Native Load dialog file-name control is {edit_class.value!r}, expected 'Edit'."
        )

    found: dict[str, int | None] = {"open": None}

    @ctypes.WINFUNCTYPE(ctypes.c_bool, wt.HWND, wt.LPARAM)
    def enum_child(hwnd, lparam):
        cls = ctypes.create_unicode_buffer(256)
        txt = ctypes.create_unicode_buffer(512)
        user32.GetClassNameW(hwnd, cls, 255)
        user32.GetWindowTextW(hwnd, txt, 511)
        if cls.value == "Button" and txt.value == "&Open":
            found["open"] = hwnd
        return True

    user32.EnumChildWindows(dialog_hwnd, enum_child, 0)
    if found["open"] is None:
        raise RuntimeError("Native Load dialog did not expose its &Open button.")
    return int(edit_hwnd), int(found["open"])


def set_dialog_text(edit_hwnd: int, value: str) -> None:
    text = ctypes.create_unicode_buffer(value)
    user32.SendMessageW(edit_hwnd, 0x000C, 0, ctypes.addressof(text))
    readback = ctypes.create_unicode_buffer(32768)
    user32.SendMessageW(edit_hwnd, 0x000D, len(readback), ctypes.addressof(readback))
    if readback.value != value:
        raise RuntimeError("Native NKI filename readback differs; no NKI submitted")
    logger.info("Native NKI filename readback verified: {}", value)


def click_button(button_hwnd: int) -> None:
    user32.SendMessageW(button_hwnd, BM_CLICK, 0, 0)


def wait_for_dialog_to_close(config: LoadNkiConfig, *, timeout_seconds: int | None = None, sample_root=None, dialog_hwnd=None, owner_window=None) -> bool:
    deadline = time.time() + (timeout_seconds or config.load_timeout_seconds)
    while time.time() < deadline:
        if sample_root is not None and any(w.title == "Content Missing" for w in gw.getAllWindows()):
            window = owner_window
            if window is None:
                raise RuntimeError("Kontakt render unavailable during sample resolution")
            resolve_content(sample_root, get_file_menu_target_window(window, config)._hWnd)
        titles = [window.title for window in gw.getAllWindows() if window.title]
        if (dialog_hwnd is not None and not user32.IsWindowVisible(dialog_hwnd)) or (dialog_hwnd is None and config.load_dialog_title not in titles):
            return True
        time.sleep(config.load_poll_seconds)
    if dialog_hwnd is not None:
        cancel_open_dialog(dialog_hwnd)
    raise RuntimeError(f"Timed out waiting for the {config.load_dialog_title!r} dialog to close.")


def wait_for_loaded_visual_change(before_image, config: LoadNkiConfig, *, sample_root=None, owner_window=None):
    deadline = time.time() + config.load_timeout_seconds
    best_diff = 0.0
    render_window = None
    previous = None
    while time.time() < deadline:
        reject_load_error_windows(sample_root,owner_window)
        kontakt_window = owner_window
        if kontakt_window is None:
            time.sleep(config.load_poll_seconds)
            continue
        render_window = get_file_menu_target_window(kontakt_window, config)
        try:
            current = capture_window_image(render_window)
        except OSError as error:
            logger.warning("Kontakt render window changed while loading: {}", error)
            time.sleep(config.load_poll_seconds)
            continue
        diff_mean = compute_rack_diff_mean(before_image, current)
        stable = previous is not None and compute_rack_diff_mean(previous, current) < 1.0
        logger.info(
            "Rack confirmation local_mean={} whole_mean={} stable={}",
            round(diff_mean, 2), round(compute_diff_mean(before_image, current), 2), stable,
        )
        if stable:
            best_diff = max(best_diff, diff_mean)
            if diff_mean >= config.confirmation_diff_mean:
                return diff_mean, render_window
        previous = current
        time.sleep(config.load_poll_seconds)
    if render_window is None:
        raise RuntimeError("Kontakt did not expose a render window after loading the NKI.")
    return best_diff, render_window


def capture_instrument_navigator(window, config: LoadNkiConfig):
    before = crop_navigator_region(capture_window_image(window), config)
    click_instr_nav(window, config)

    deadline = time.time() + config.navigator_open_timeout_seconds
    best_crop = before
    best_diff = 0.0
    while time.time() < deadline:
        current = crop_navigator_region(capture_window_image(window), config)
        diff_mean = compute_diff_mean(before, current)
        if diff_mean > best_diff:
            best_diff = diff_mean
            best_crop = current
        if diff_mean >= config.navigator_open_diff_mean:
            return current
        time.sleep(config.load_poll_seconds)

    return best_crop


def wait_for_navigator_change(window, before_crop, initial_crop, config: LoadNkiConfig):
    deadline = time.time() + config.navigator_confirmation_timeout_seconds
    best_crop = initial_crop
    best_diff = compute_diff_mean(before_crop, initial_crop)
    if best_diff >= config.navigator_confirmation_diff_mean:
        logger.info("Instrument Navigator diff_mean={}", round(best_diff, 2))
        return initial_crop, best_diff
    while time.time() < deadline:
        current = crop_navigator_region(capture_window_image(window), config)
        diff_mean = compute_diff_mean(before_crop, current)
        if diff_mean > best_diff:
            best_diff = diff_mean
            best_crop = current
        logger.info("Instrument Navigator diff_mean={}", round(diff_mean, 2))
        if diff_mean >= config.navigator_confirmation_diff_mean:
            return current, diff_mean
        time.sleep(config.load_poll_seconds)
    return best_crop, best_diff


def click_instr_nav(window, config: LoadNkiConfig) -> None:
    client_width, client_height = capture_window_image(window).size
    x, y = client_point_to_screen(
        window,
        int(client_width * config.instr_nav_x_ratio),
        int(client_height * config.instr_nav_y_ratio),
    )
    logger.info("Clicking Instr Nav at {}, {}", x, y)
    click_screen_point(x, y)


def crop_navigator_region(image, config: LoadNkiConfig):
    width, height = image.size
    left = int(width * config.navigator_left_ratio)
    top = int(height * config.navigator_top_ratio)
    right = int(width * config.navigator_right_ratio)
    bottom = int(height * config.navigator_bottom_ratio)
    return image.crop((left, top, right, bottom))


def compute_diff_mean(before_image, after_image) -> float:
    before_gray = before_image.convert("L")
    after_gray = after_image.convert("L")
    diff = ImageChops.difference(before_gray, after_gray)
    return float(ImageStat.Stat(diff).mean[0])


def compute_rack_diff_mean(before_image, after_image) -> float:
    """Measure a coherent rendered change without dilution by unused window area.

    A fixed 256x128 tile covers an instrument header/body region. Edge tiles
    retain that denominator so a tiny cursor at an image edge cannot pass.
    Unlike cropping to the changed pixels, isolated noise remains insignificant.
    """
    if before_image.size != after_image.size:
        raise ValueError("Kontakt render dimensions changed during load")
    diff = ImageChops.difference(before_image.convert("L"), after_image.convert("L"))
    width, height = diff.size
    return max(
        (ImageStat.Stat(diff.crop((x, y, x + 256, y + 128))).sum[0] / (256 * 128)
         for y in range(0, height, 128) for x in range(0, width, 256)),
        default=0.0,
    )


CALLBACK = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
user32.SendMessageW.argtypes = [wt.HWND, wt.UINT, wt.WPARAM, wt.LPARAM]
user32.SendMessageW.restype = wt.LPARAM
user32.GetDlgItem.argtypes = [wt.HWND, ctypes.c_int]
user32.GetDlgItem.restype = wt.HWND
user32.GetAncestor.argtypes = [wt.HWND, wt.UINT]
user32.GetAncestor.restype = wt.HWND

def windows(parent=None):
    found = []

    @CALLBACK
    def collect(hwnd, _):
        text = ctypes.create_unicode_buffer(1024)
        user32.GetWindowTextW(hwnd, text, len(text))
        if user32.IsWindowVisible(hwnd):
            found.append((hwnd, text.value))
        return True

    if parent is None:
        user32.EnumWindows(collect, 0)
    else:
        user32.EnumChildWindows(parent, collect, 0)
    return found


def resolve_content(sample_root, render):
    """Admitted 7.10.1 modal only; search one explicit library, never ignore files."""
    root = Path(sample_root)
    if not root.is_dir():
        raise RuntimeError("Documented sample root absent")
    if any(t == "Browse for Folder" for _, t in windows()):
        raise RuntimeError("Existing folder dialog must be reconciled first")
    dialogs = [h for h, t in windows() if t == "Content Missing"]
    if len(dialogs) != 1:
        raise RuntimeError("Expected sole Content Missing dialog")
    render_pid, dialog_pid = wt.DWORD(), wt.DWORD()
    user32.GetWindowThreadProcessId(render, ctypes.byref(render_pid))
    user32.GetWindowThreadProcessId(dialogs[0], ctypes.byref(dialog_pid))
    if render_pid.value != dialog_pid.value:
        raise RuntimeError("Missing-content dialog belongs to another process")
    children = [h for h, t in windows(dialogs[0]) if t == "Kontakt 7"]
    if len(children) != 1:
        raise RuntimeError("Expected admitted Kontakt missing-content surface")
    hwnd = children[0]
    rect = wt.RECT()
    user32.GetClientRect(hwnd, ctypes.byref(rect))
    if (rect.right, rect.bottom) != (600, 440):
        raise RuntimeError("Unadmitted Content Missing geometry")
    # Observed Browse for folder button on this version's 600x440 modal.
    point = wt.POINT(225, 337)
    if not user32.ClientToScreen(hwnd, ctypes.byref(point)) or not user32.SetForegroundWindow(dialogs[0]):
        raise RuntimeError("Cannot activate owned content dialog")
    if not user32.SetCursorPos(point.x, point.y):
        raise RuntimeError("Cannot position pointer on content dialog")
    user32.WindowFromPoint.argtypes = [wt.POINT]
    user32.WindowFromPoint.restype = wt.HWND
    if user32.GetAncestor(user32.WindowFromPoint(point), 2) != dialogs[0]:
        raise RuntimeError("Content button is obscured; no pointer input")
    # Win32 pointer input is necessary for this Qt modal; posted messages did not
    # activate its button. Foreground and hit-test fence it to the owned dialog.
    user32.mouse_event(2, 0, 0, 0, 0)
    user32.mouse_event(4, 0, 0, 0, 0)
    deadline = time.monotonic() + 15
    while True:
        folders = [h for h, t in windows() if t == "Browse for Folder"]
        if len(folders) == 1:
            break
        if len(folders) > 1 or time.monotonic() >= deadline:
            raise RuntimeError("Exact native sample-folder dialog unavailable")
        time.sleep(0.1)
    dialog = folders[0]
    native.choose_folder(dialog, PureWindowsPath(str(root)), render_pid.value)
    deadline = time.monotonic() + 120
    while any(t in ("Browse for Folder", "Content Missing") for _, t in windows()):
        if time.monotonic() >= deadline:
            raise RuntimeError("Sample resolution incomplete; no ignore or global filesystem search")
        time.sleep(0.25)
    logger.info("Resolved missing samples in exact NKI parent: {}", root)



def cancel_open_dialog(hwnd):
    button=user32.GetDlgItem(hwnd,2)
    cls=ctypes.create_unicode_buffer(256)
    if not button or not user32.GetClassNameW(button,cls,len(cls)) or cls.value!="Button":
        raise RuntimeError("Exact pending Load Cancel control unavailable")
    if not user32.PostMessageW(button,BM_CLICK,0,0):
        raise RuntimeError("Could not request cancellation of exact pending Load dialog")
    deadline=time.monotonic()+10
    while user32.IsWindowVisible(hwnd):
        if time.monotonic()>=deadline:raise RuntimeError("Pending Load cancellation outcome unknown")
        time.sleep(0.1)
