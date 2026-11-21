import json
import sys
from pathlib import Path
from typing import Annotated, Dict, Optional

import typer
from rich.console import Console
from rich.table import Table

from kontakt_ctl.commands import batch_resave, launch_and_check, load_nki, resave_all
from kontakt_ctl.config import BatchResaveConfig, LaunchConfig, LoadNkiConfig
from kontakt_ctl.inspect_rack import inspect_running_rack
from kontakt_ctl.logging import configure_logging

app = typer.Typer(no_args_is_help=True, add_completion=False)
console = Console()
json_output_enabled = False


def emit_json(command: str, result: object) -> None:
    payload: Dict[str, object] = {
        "schema": "kontakt-ctl.result/v1",
        "command": command,
        "ok": True,
        "data": vars(result),
    }
    sys.stdout.write(json.dumps(payload, default=str) + "\n")


@app.callback()
def main_callback(
    verbose: bool = typer.Option(False, "--verbose", help="Enable debug logging."),
    json_output: bool = typer.Option(
        False, "--json", help="Emit one stable JSON result object instead of a Rich table."
    ),
) -> None:
    global json_output_enabled
    json_output_enabled = json_output
    configure_logging(verbose=verbose)


@app.command("launch-and-check")
def launch_and_check_command() -> None:
    config = LaunchConfig()
    result = launch_and_check(config)
    if json_output_enabled:
        emit_json("launch-and-check", result)
        return

    table = Table(title="Kontakt Launch Check")
    table.add_column("Field")
    table.add_column("Value")
    table.add_row("PID", str(result.pid))
    table.add_row("Window", result.window_title)
    table.add_row("DXVK_HUD", result.dxvk_hud)
    table.add_row("Capture", f"{result.window_width}x{result.window_height}")
    table.add_row("Full Screenshot", str(result.fullshot_path))
    table.add_row("Top Left Crop", str(result.top_left_path))
    console.print(table)


@app.command("batch-resave")
def batch_resave_command(
    directory: str = typer.Argument(
        ...,
        help="Folder name directly under X:, for example 8Dio.Lacrimosa.Choir.NKS.KONTAKT",
    ),
    check_only: bool = typer.Option(False, "--check", help="Verify completed processing coverage against current instrument paths; never launch Kontakt."),
    close_after_seconds: int | None = typer.Option(
        None,
        "--close-after-seconds",
        min=0,
        help="Optional delay after verified completion before closing the owned Kontakt process.",
    ),
) -> None:
    config = BatchResaveConfig()
    result = batch_resave(config, directory, close_after_seconds=close_after_seconds, check_only=check_only)
    if json_output_enabled:
        emit_json("batch-resave", result)
        return

    table = Table(title="Kontakt Batch Resave")
    table.add_column("Field")
    table.add_column("Value")
    table.add_row("PID", str(result.pid))
    table.add_row("Window", result.window_title)
    table.add_row("Requested Folder", result.requested_folder)
    table.add_row("Target Folder", result.target_folder)
    table.add_row("Missing Files Dialog", str(result.missing_files_dialog_seen))
    table.add_row("Files Changed", str(result.files_changed_count))
    table.add_row("Completion Confirmed", str(result.completion_confirmed))
    table.add_row("Poll Log", result.poll_log_path)
    table.add_row("Receipt", result.receipt_path)
    table.add_row("Skipped (verified receipt)", str(result.skipped))
    console.print(table)


@app.command("windows")
def windows_command() -> None:
    """Read visible host-window identities before the adapter opens its editor."""
    from kontakt_ctl.load_nki import visible_host_windows
    result=visible_host_windows()
    if json_output_enabled:emit_json("windows",result)
    else:console.print(result)


@app.command("load-nki")
def load_nki_command(
    relative_path: str = typer.Argument(
        ...,
        help=(
            "NKI path relative to X:, for example "
            "'8Dio.Lacrimosa.Choir.NKS.KONTAKT/01 Lacrimosa Arcs.nki'"
        ),
    ),
    owner_window: int = typer.Option(...,"--owner-window",help="Exact adapter-opened Win32 owner HWND."),
    owner_pid:int=typer.Option(...,"--owner-pid",help="Qualified Win32 owner process."),
    observe_only: bool = typer.Option(False, "--observe-only", help="Observe pending owned load windows without taking input or submitting an NKI."),
    cancel_failed_dialog: bool = typer.Option(False, "--cancel-failed-dialog", help="Cancel only the exact owned failed Load transaction; never submits an NKI."),
) -> None:
    config = LoadNkiConfig()
    result = load_nki(
        config,
        relative_path,
        owner_window=owner_window,
        owner_pid=owner_pid,
        observe_only=observe_only,
        cancel_failed_dialog=cancel_failed_dialog,
    )
    if not result.load_confirmed and not result.observation_only and not result.cancelled_pending_dialog:
        raise RuntimeError(
            "Kontakt did not confirm the requested NKI load: "
            f"dialog_closed={result.dialog_closed}, "
            f"rack_visual_confirmed={result.rack_visual_confirmed}, "
            f"navigator_confirmed={result.navigator_confirmed}"
        )
    if json_output_enabled:
        emit_json("load-nki", result)
        return

    table = Table(title="Kontakt Load NKI")
    table.add_column("Field")
    table.add_column("Value")
    table.add_row("PID", str(result.pid))
    table.add_row("Window", result.window_title)
    table.add_row("Requested Path", result.requested_relative_path)
    table.add_row("Target Path", result.target_path)
    table.add_row("Dialog Closed", str(result.dialog_closed))
    table.add_row("Rack Confirmed", str(result.rack_visual_confirmed))
    table.add_row("Rack Diff Mean", f"{result.rack_diff_mean:.2f}")
    table.add_row("Navigator Confirmed", str(result.navigator_confirmed))
    table.add_row("Navigator Diff Mean", f"{result.navigator_diff_mean:.2f}")
    table.add_row("Load Confirmed", str(result.load_confirmed))
    table.add_row("Screenshot", result.screenshot_path)
    table.add_row("Navigator Crop", result.navigator_path)
    console.print(table)


@app.command("resave-all")
def resave_all_command(
    jsonl_path: Annotated[
        Optional[Path],
        typer.Option(
            "--jsonl-path",
            help="Optional JSONL output path. Defaults to artifacts/resave_all.<timestamp>.jsonl",
        ),
    ] = None,
) -> None:
    config = BatchResaveConfig()
    result = resave_all(config, jsonl_path=jsonl_path)
    if result.error_jobs:
        raise RuntimeError(f"{result.error_jobs} batch-resave jobs failed; see {result.jsonl_path}")
    if json_output_enabled:
        emit_json("resave-all", result)
        return

    table = Table(title="Kontakt Resave All")
    table.add_column("Field")
    table.add_column("Value")
    table.add_row("Total Jobs", str(result.total_jobs))
    table.add_row("Resaved", str(result.resaved_jobs))
    table.add_row("Unchanged", str(result.unchanged_jobs))
    table.add_row("Errors", str(result.error_jobs))
    table.add_row("Skipped Existing", str(result.skipped_jobs))
    table.add_row("JSONL", str(result.jsonl_path))
    console.print(table)


@app.command("inspect-rack")
def inspect_rack_command(
    output_stem: str = typer.Option(
        "kontakt-rack",
        "--output-stem",
        help="Safe stem for the rack and Instrument Navigator evidence images.",
    ),
) -> None:
    result = inspect_running_rack(output_stem, LoadNkiConfig())
    if json_output_enabled:
        emit_json("inspect-rack", result)
        return
    table = Table(title="Kontakt Rack Inspection")
    table.add_column("Field")
    table.add_column("Value")
    table.add_row("Window", result.window_title)
    table.add_row("Capture", f"{result.width}x{result.height}")
    table.add_row("Rack Screenshot", str(result.screenshot_path))
    table.add_row("Navigator Crop", str(result.navigator_path))
    console.print(table)


def main() -> None:
    try:
        app()
    except Exception as error:
        if json_output_enabled:
            payload: Dict[str, object] = {
                "schema": "kontakt-ctl.result/v1",
                "command": "kontakt-ctl",
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "message": str(error),
                },
            }
            sys.stdout.write(json.dumps(payload) + "\n")
            raise SystemExit(1) from None
        raise
