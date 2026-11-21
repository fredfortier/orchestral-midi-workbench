import json
import os
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from loguru import logger

from kontakt_ctl.batch_resave import BatchResaveResult, run_batch_resave
from kontakt_ctl.capture import capture_window_artifacts
from kontakt_ctl.config import BatchResaveConfig, LaunchConfig, LoadNkiConfig
from kontakt_ctl.launcher import launch_kontakt, terminate_kontakt
from kontakt_ctl.load_nki import LoadNkiResult, run_load_nki
from kontakt_ctl.window_ops import focus_and_prepare_window, wait_for_kontakt_window


@dataclass(frozen=True)
class LaunchAndCheckResult:
    pid: int
    window_title: str
    fullshot_path: Path
    top_left_path: Path
    window_width: int
    window_height: int
    dxvk_hud: str


RESAVE_ALL_STATUS_RESAVED = "RESAVED"
RESAVE_ALL_STATUS_UNCHANGED = "UNCHANGED"
RESAVE_ALL_STATUS_ERROR = "ERROR"


@dataclass(frozen=True)
class ResaveAllResult:
    total_jobs: int
    resaved_jobs: int
    unchanged_jobs: int
    error_jobs: int
    jsonl_path: Path
    skipped_jobs: int


def launch_and_check(config: LaunchConfig) -> LaunchAndCheckResult:
    logger.info("DXVK_HUD={}", os.environ.get("DXVK_HUD", "<unset>"))
    process = launch_kontakt(config.exe_path)
    try:
        time.sleep(config.startup_wait_seconds)
        window = wait_for_kontakt_window(config)
        focus_and_prepare_window(window, config)
        logger.info("Focused window: {}", window.title)
        time.sleep(config.settle_seconds)
        width, height = capture_window_artifacts(window, config)
        return LaunchAndCheckResult(
            pid=process.pid,
            window_title=window.title,
            fullshot_path=config.fullshot_path,
            top_left_path=config.top_left_path,
            window_width=width,
            window_height=height,
            dxvk_hud=os.environ.get("DXVK_HUD", "<unset>"),
        )
    finally:
        time.sleep(2)
        terminate_kontakt()


def batch_resave(
    config: BatchResaveConfig,
    directory: str,
    close_after_seconds: int | None = None,
    check_only: bool = False,
) -> BatchResaveResult:
    return run_batch_resave(directory, config, close_after_seconds=close_after_seconds, check_only=check_only)


def load_nki(config: LoadNkiConfig, relative_path: str, *, owner_window: int, owner_pid:int,
             observe_only: bool=False, cancel_failed_dialog: bool=False) -> LoadNkiResult:
    return run_load_nki(relative_path,config,owner_window=owner_window,owner_pid=owner_pid,
                        observe_only=observe_only,cancel_failed_dialog=cancel_failed_dialog)


def resave_all(
    config: BatchResaveConfig,
    *,
    jsonl_path: Path | None = None,
) -> ResaveAllResult:
    folders = list_top_level_x_folders()
    output_path = jsonl_path or build_resave_all_jsonl_path(config)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    load_jsonl_records(output_path)  # Reject a corrupt ledger before mutation.
    skipped_jobs = 0
    current_records = []

    for index, folder_name in enumerate(folders, start=1):
        started_at = time.time()
        logger.info("Starting resave-all job {}/{} for {}", index, len(folders), folder_name)
        try:
            result = run_batch_resave(folder_name, config)
            skipped_jobs += int(result.skipped)
            payload = {
                "timestamp_utc": iso_timestamp(),
                "job_index": index,
                "job_count": len(folders),
                "folder_name": folder_name,
                "target_folder": result.target_folder,
                "status_code": classify_resave_all_status(result),
                "duration_seconds": round(time.time() - started_at, 3),
                "pid": result.pid,
                "window_title": result.window_title,
                "missing_files_dialog_seen": result.missing_files_dialog_seen,
                "files_changed_count": result.files_changed_count,
                "completion_confirmed": result.completion_confirmed,
                "files_expected_count": result.files_expected_count,
                "process_exit_code": result.process_exit_code,
                "receipt_path": result.receipt_path,
                "skipped": result.skipped,
                "verification_schema": "kontakt-ctl.batch-resave/v2",
                "poll_log_path": result.poll_log_path,
            }
        except Exception as exc:
            payload = {
                "timestamp_utc": iso_timestamp(),
                "job_index": index,
                "job_count": len(folders),
                "folder_name": folder_name,
                "status_code": RESAVE_ALL_STATUS_ERROR,
                "duration_seconds": round(time.time() - started_at, 3),
                "error_type": type(exc).__name__,
                "error_message": str(exc),
            }
            logger.exception("resave-all job failed for {}", folder_name)

        append_jsonl(output_path, payload)
        current_records.append(payload)

    summary = summarize_resave_all(current_records)

    return ResaveAllResult(
        total_jobs=len(folders),
        resaved_jobs=summary[RESAVE_ALL_STATUS_RESAVED],
        unchanged_jobs=summary[RESAVE_ALL_STATUS_UNCHANGED],
        error_jobs=summary[RESAVE_ALL_STATUS_ERROR],
        jsonl_path=output_path,
        skipped_jobs=skipped_jobs,
    )


def classify_resave_all_status(result: BatchResaveResult) -> str:
    if not result.completion_confirmed:
        return RESAVE_ALL_STATUS_ERROR
    if result.files_changed_count > 0:
        return RESAVE_ALL_STATUS_RESAVED
    return RESAVE_ALL_STATUS_UNCHANGED


def list_top_level_x_folders() -> list[str]:
    return sorted(
        [
            path.name
            for path in Path("X:/").iterdir()
            if path.is_dir() and not path.name.startswith(".")
        ],
        key=str.casefold,
    )


def build_resave_all_jsonl_path(config: BatchResaveConfig) -> Path:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return config.artifacts_dir / f"resave_all.{stamp}.jsonl"


def append_jsonl(path: Path, payload: dict) -> None:
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(payload, sort_keys=True) + "\n")


def load_jsonl_records(path: Path) -> list[dict]:
    if not path.exists():
        return []
    records: list[dict] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                payload = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise ValueError(f"Invalid JSONL in {path} at line {line_number}") from exc
            if isinstance(payload, dict):
                records.append(payload)
    return records


def summarize_resave_all(records: list[dict]) -> dict[str, int]:
    summary = {
        RESAVE_ALL_STATUS_RESAVED: 0,
        RESAVE_ALL_STATUS_UNCHANGED: 0,
        RESAVE_ALL_STATUS_ERROR: 0,
    }
    for record in records:
        status_code = record.get("status_code")
        if status_code in summary:
            summary[status_code] += 1
    return summary


def iso_timestamp() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")
