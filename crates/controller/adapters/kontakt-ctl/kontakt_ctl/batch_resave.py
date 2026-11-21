"""Verified standalone batch-resave transaction; no global player discovery."""

import json
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path, PureWindowsPath

from loguru import logger

from kontakt_ctl import resave_native as native
from kontakt_ctl.capture import wait_for_window_render
from kontakt_ctl.config import BatchResaveConfig
from kontakt_ctl.launcher import launch_kontakt
from kontakt_ctl.ui_ops import click_target_image, ensure_images_exist
from kontakt_ctl.window_ops import (
    WindowHandleProxy,
    activate_window,
    focus_and_prepare_window,
)


@dataclass(frozen=True)
class BatchResaveResult:
    pid: int | None
    window_title: str
    requested_folder: str
    target_folder: str
    missing_files_dialog_seen: bool
    files_changed_count: int
    completion_confirmed: bool
    poll_log_path: str
    files_expected_count: int
    process_exit_code: int
    receipt_path: str
    skipped: bool


RECEIPT_NAME = ".kontakt-ctl-batch-resave.json"
RECEIPT_SCHEMA = "kontakt-ctl.batch-resave/v3"


def verified_receipt(folder: Path, config: BatchResaveConfig) -> dict | None:
    """Reuse completed processing with matching covered paths and successful outcome."""
    path = folder / RECEIPT_NAME
    if not path.exists():
        return None
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        inventory = snapshot_resave_files(folder)
        if (
            receipt["schema"] in (RECEIPT_SCHEMA, "kontakt-ctl.batch-resave/v2")
            and receipt["target_folder"] == str(folder)
            and receipt["completion_confirmed"] is True
            and type(receipt["missing_files_dialog_seen"]) is bool
            and isinstance(receipt["poll_log_path"], str)
            and bool(receipt["poll_log_path"])
            and type(receipt["files_expected_count"]) is int
            and type(receipt["files_changed_count"]) is int
            and type(receipt["process_exit_code"]) is int
            and receipt["process_exit_code"] == 0
            and receipt["files_expected_count"]
            == receipt["files_changed_count"]
            == len(inventory)
            and len(inventory) > 0
            and sorted(receipt.get("files", receipt.get("sha256", {})))
            == sorted(path.relative_to(folder).as_posix() for path in inventory)
        ):
            return receipt
    except (OSError, ValueError, KeyError, TypeError):
        return None
    return None


def build_target_folder(requested_folder: str) -> PureWindowsPath:
    raw = requested_folder.strip()
    path = PureWindowsPath(raw)
    if (
        not raw
        or raw in (".", "..")
        or path.drive
        or path.root
        or any(c in raw for c in '\\/:*?"<>|')
        or raw.endswith((".", " "))
        or len(path.parts) != 1
    ):
        raise ValueError(
            "Directory must be one folder name directly under X:, without traversal or separators"
        )
    return PureWindowsPath("X:/") / path


def snapshot_resave_files(folder: Path) -> dict[Path, tuple[int, int]]:
    if folder.is_symlink() or (folder.lstat().st_file_attributes & 0x400):
        raise ValueError("Resave target must not be a symbolic link")
    result = {}
    for path in folder.rglob("*"):
        if path.is_symlink() or (path.lstat().st_file_attributes & 0x400):
            raise ValueError(f"Resave target contains a filesystem redirect: {path}")
        if path.is_file() and path.suffix.lower() in {".nki", ".nkm", ".nkb"}:
            info = path.stat()
            if info.st_size == 0:
                raise ValueError(f"Empty Kontakt file: {path}")
            result[path] = (info.st_size, info.st_mtime_ns)
    return result


def rewritten_files(before: dict, current: dict) -> set[Path]:
    if current.keys() != before.keys():
        raise RuntimeError(
            "Kontakt file inventory changed during resave; outcome is incomplete"
        )
    if any(size == 0 for size, _ in current.values()):
        raise RuntimeError("Kontakt wrote an empty instrument file")
    return {path for path, state in current.items() if state != before[path]}


def log_event(path: Path, phase: str, **fields) -> None:
    with path.open("a", encoding="utf-8") as output:
        output.write(json.dumps({"phase": phase, **fields}, ensure_ascii=False) + "\n")


def open_batch_menu(window: WindowHandleProxy, config: BatchResaveConfig) -> None:
    # Menu positions are hints only; a PID-owned native warning must follow.
    for image, label, ratios in (
        (
            config.file_menu_image,
            "FILE menu",
            (config.file_menu_x_ratio, config.file_menu_y_ratio),
        ),
        (
            config.batch_resave_image,
            "Batch resave menu",
            (config.batch_resave_x_ratio, config.batch_resave_y_ratio),
        ),
    ):
        click_target_image(
            image,
            window,
            label=label,
            timeout_seconds=config.image_timeout_seconds,
            grayscale=config.image_grayscale,
            confidence=config.image_confidence,
            fallback_target=window,
            fallback_ratio=ratios,
        )


def confirm_warning(pid: int, timeout: float, process) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError("Kontakt exited before its resave warning")
        dialogs = [w for w in native.enumerate_windows(pid) if w.class_name == "#32770"]
        if len(dialogs) > 1:
            raise RuntimeError("Batch-resave warning not uniquely identified")
        if dialogs:
            warning = dialogs[0]
            break
        time.sleep(0.1)
    else:
        raise TimeoutError("No PID-owned batch-resave warning")
    texts = [
        native.window_text(w.hwnd) for w in native.enumerate_windows(pid, warning.hwnd)
    ]
    text = " ".join(texts).casefold()
    if "resav" not in text and "re-sav" not in text:
        raise RuntimeError(
            f"Unrecognized Kontakt warning; refused confirmation: {text}"
        )
    native.click_button(warning.hwnd, 6)  # IDYES


def resolve_missing(
    dialog: native.NativeWindow,
    pid: int,
    target: PureWindowsPath,
    config: BatchResaveConfig,
    process,
) -> None:
    # Kontakt's missing-content dialog uses a custom button: image match only
    # inside this PID-owned window. Never search other applications or fallback.
    children = [
        w
        for w in native.enumerate_windows(pid, dialog.hwnd)
        if w.title == "Kontakt 7" and w.class_name.startswith("Qt")
    ]
    if len(children) != 1:
        raise RuntimeError("Missing-content dialog has no unique owned render child")
    render = native.as_proxy(children[0])
    modal = native.as_proxy(dialog)
    activate_window(modal)
    modal.moveTo(100, 100)
    time.sleep(0.2)
    wait_for_window_render(render, config)
    native.assert_owner(render._hWnd, pid)
    click_target_image(
        config.browse_for_folder_image,
        render,
        label="Browse for folder",
        timeout_seconds=config.image_timeout_seconds,
        grayscale=config.image_grayscale,
        confidence=config.image_confidence,
    )
    picker = native.wait_window(
        pid,
        ("Browse for Folder", config.select_target_folder_title),
        config.dialog_timeout_seconds,
        process,
    )
    native.choose_folder(picker.hwnd, target, pid)


def wait_for_completion(
    process,
    main: native.NativeWindow,
    target: PureWindowsPath,
    before: dict,
    config: BatchResaveConfig,
    log: Path,
) -> tuple[bool, int]:
    started = time.monotonic()
    deadline = started + config.final_resave_timeout_seconds
    missing_seen = False
    missing_hwnd = None
    last_changed = -1
    folder = Path(str(target))
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(
                f"Kontakt exited before completion: {process.returncode}"
            )
        windows = native.enumerate_windows(process.pid)
        missing = [
            w
            for w in windows
            if w.title in (config.content_missing_title, config.missing_files_title)
        ]
        if len(missing) > 1:
            raise RuntimeError("Multiple owned missing-content dialogs")
        if missing:
            if missing_hwnd == missing[0].hwnd:
                raise RuntimeError(
                    "Missing samples remain after exact-library search; refused retry"
                )
            missing_seen, missing_hwnd = True, missing[0].hwnd
            log_event(log, "missing-content", target=str(target))
            resolve_missing(missing[0], process.pid, target, config, process)
            # Wait for this exact dialog to close; don't submit another search.
            search_deadline = min(
                deadline, time.monotonic() + config.dialog_timeout_seconds
            )
            while native.u.IsWindowVisible(missing_hwnd):
                if process.poll() is not None or time.monotonic() >= search_deadline:
                    raise TimeoutError(
                        "Missing-content search did not close its dialog"
                    )
                time.sleep(config.resave_poll_seconds)
            continue
        current = snapshot_resave_files(folder)
        changed = rewritten_files(before, current)
        if len(changed) != last_changed:
            log_event(log, "writes", rewritten=len(changed), expected=len(before))
            logger.info("Rewritten Kontakt files: {}/{}", len(changed), len(before))
            last_changed = len(changed)
        idle = native.main_is_idle(process.pid, main.hwnd)
        # Recheck inventory after the main window returns from its modal
        # operation. Every file is required, never merely inactivity.
        if (
            idle
            and len(changed) == len(before)
            and native.files_are_closed(before)
            and len(rewritten_files(before, snapshot_resave_files(folder)))
            == len(before)
        ):
            log_event(log, "verified", rewritten=len(changed), idle=True)
            return missing_seen, len(changed)
        if (
            idle
            and time.monotonic() - started >= config.resave_no_change_timeout_seconds
        ):
            raise RuntimeError(
                f"Kontakt returned idle but only {len(changed)}/{len(before)} files were rewritten"
            )
        unknown = [
            w
            for w in windows
            if w.hwnd != main.hwnd
            and w.class_name == "#32770"
            and w.title
            not in ("Browse for Folder", config.choose_base_folder_title, "Progress")
        ]
        if unknown:
            raise RuntimeError(
                f"Unexpected owned Kontakt dialog: {[w.title for w in unknown]}"
            )
        time.sleep(config.resave_poll_seconds)
    raise TimeoutError("Batch resave deadline exceeded; completion unproven")


def close_owned_process(
    process, main: native.NativeWindow | None, *, success: bool
) -> int:
    forced = False
    try:
        if process.poll() is None and main is not None and native.u.IsWindow(main.hwnd):
            if native.process_id(main.hwnd) == process.pid:
                native.post(main.hwnd, 0x10)
            deadline = time.monotonic() + 15
            reported = set()
            while process.poll() is None and time.monotonic() < deadline:
                for window in native.enumerate_windows(process.pid):
                    if window.hwnd != main.hwnd and window.hwnd not in reported:
                        reported.add(window.hwnd)
                        texts = [
                            native.window_text(w.hwnd)
                            for w in native.enumerate_windows(process.pid, window.hwnd)
                        ]
                        logger.warning("Owned shutdown dialog: {} {}", window, texts)
                time.sleep(0.1)
    except (OSError, RuntimeError) as error:
        logger.warning("Graceful owned-process close failed: {}", error)
    finally:
        if process.poll() is None:
            forced = True
            process.terminate()
            process.wait(timeout=5)
    if success and forced:
        raise RuntimeError("Kontakt required forced termination; clean exit unproven")
    if success and process.returncode != 0:
        raise RuntimeError(f"Kontakt did not exit cleanly: {process.returncode}")
    return process.returncode


def run_batch_resave(
    requested_folder: str,
    config: BatchResaveConfig,
    close_after_seconds: int | None = None,
    *,
    check_only: bool = False,
) -> BatchResaveResult:
    with native.batch_lock():
        return _run_batch_resave(
            requested_folder, config, close_after_seconds, check_only=check_only
        )


def _run_batch_resave(
    requested_folder: str,
    config: BatchResaveConfig,
    close_after_seconds: int | None,
    *,
    check_only: bool,
) -> BatchResaveResult:
    target = build_target_folder(requested_folder)
    folder = Path(str(target))
    if not folder.is_dir():
        raise FileNotFoundError(f"Folder does not exist: {target}")
    before = snapshot_resave_files(folder)
    if not before:
        raise ValueError("Target contains no Kontakt instrument, multi or bank files")
    receipt_path = folder / RECEIPT_NAME
    receipt = verified_receipt(folder, config)
    if receipt is not None:
        logger.info(
            "Current verified resave receipt; skipping Kontakt: {}", receipt_path
        )
        return BatchResaveResult(
            None,
            "not launched (verified receipt)",
            requested_folder,
            str(target),
            receipt["missing_files_dialog_seen"],
            0,
            True,
            receipt["poll_log_path"],
            len(before),
            0,
            str(receipt_path),
            True,
        )
    if check_only:
        raise RuntimeError(
            "No current verified batch-resave receipt; no Kontakt process launched"
        )
    ensure_images_exist(
        config.file_menu_image,
        config.batch_resave_image,
        config.browse_for_folder_image,
    )
    config.artifacts_dir.mkdir(parents=True, exist_ok=True)
    log = config.artifacts_dir / f"batch-resave-{time.time_ns()}.jsonl"
    log_event(log, "requested", target=str(target), expected=len(before))
    process = launch_kontakt(config.exe_path)
    main = None
    success = False
    try:
        main = native.wait_window(
            process.pid, ("Kontakt 7",), config.window_timeout_seconds, process
        )
        focus_and_prepare_window(native.as_proxy(main), config)
        wait_for_window_render(native.as_proxy(main), config)
        open_batch_menu(native.as_proxy(main), config)
        confirm_warning(process.pid, config.dialog_timeout_seconds, process)
        picker = native.wait_window(
            process.pid,
            ("Browse for Folder", config.choose_base_folder_title),
            config.dialog_timeout_seconds,
            process,
        )
        native.choose_folder(picker.hwnd, target, process.pid)
        log_event(log, "submitted", verified_target=str(target))
        # Do not interpret the picker still closing as a second operation.
        deadline = time.monotonic() + config.dialog_timeout_seconds
        while native.u.IsWindowVisible(picker.hwnd):
            if time.monotonic() >= deadline or process.poll() is not None:
                raise TimeoutError("Verified folder picker did not close")
            time.sleep(config.resave_poll_seconds)
        missing, changed = wait_for_completion(
            process, main, target, before, config, log
        )
        if close_after_seconds is not None:
            time.sleep(close_after_seconds)
        exit_code = close_owned_process(process, main, success=True)
        after = snapshot_resave_files(folder)
        if len(rewritten_files(before, after)) != len(before):
            raise RuntimeError(
                "Final instrument verification failed after Kontakt exit"
            )
        log_event(log, "closed", exit_code=exit_code)
        receipt = {
            "schema": RECEIPT_SCHEMA,
            "target_folder": str(target),
            "completed_at": datetime.now(UTC).isoformat(),
            "completion_confirmed": True,
            "process_exit_code": exit_code,
            "files_expected_count": len(before),
            "files_changed_count": changed,
            "missing_files_dialog_seen": missing,
            "poll_log_path": str(log.resolve()),
            "files": sorted(path.relative_to(folder).as_posix() for path in after),
        }
        temp_receipt = receipt_path.with_suffix(f".{time.time_ns()}.tmp")
        try:
            temp_receipt.write_text(
                json.dumps(receipt, indent=2, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            temp_receipt.replace(receipt_path)
        finally:
            temp_receipt.unlink(missing_ok=True)
        success = True
        return BatchResaveResult(
            process.pid,
            main.title,
            requested_folder,
            str(target),
            missing,
            changed,
            True,
            str(log),
            len(before),
            exit_code,
            str(receipt_path),
            False,
        )
    except BaseException as error:
        log_event(log, "error", type=type(error).__name__, message=str(error))
        raise
    finally:
        if not success:
            close_owned_process(process, main, success=False)
