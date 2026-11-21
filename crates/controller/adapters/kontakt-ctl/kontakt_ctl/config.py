from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field


class LaunchConfig(BaseModel):
    model_config = ConfigDict(frozen=True)

    exe_path: str = Field(default=r"C:\Program Files\Native Instruments\Kontakt 7\Kontakt 7.exe")
    window_title_contains: str = Field(default="Kontakt")
    startup_wait_seconds: int = Field(default=10, ge=0)
    window_timeout_seconds: int = Field(default=30, ge=1)
    settle_seconds: int = Field(default=3, ge=0)
    render_timeout_seconds: int = Field(default=15, ge=1)
    render_poll_seconds: float = Field(default=1.0, gt=0.0)
    render_ready_min_mean: float = Field(default=5.0, ge=0.0)
    min_width: int = Field(default=1280, ge=1)
    min_height: int = Field(default=820, ge=1)
    artifacts_dir: Path = Field(default=Path("artifacts"))
    fullshot_name: str = Field(default="kontakt_full.png")
    top_left_name: str = Field(default="kontakt_top_left.png")
    top_left_width: int = Field(default=560, ge=1)
    top_left_height: int = Field(default=150, ge=1)

    @property
    def fullshot_path(self) -> Path:
        return self.artifacts_dir / self.fullshot_name

    @property
    def top_left_path(self) -> Path:
        return self.artifacts_dir / self.top_left_name


class BatchResaveConfig(LaunchConfig):
    file_menu_image: Path = Field(default=Path("assets/targets/file__menu.png"))
    batch_resave_image: Path = Field(default=Path("assets/targets/batch_resave_menu.png"))
    browse_for_folder_image: Path = Field(
        default=Path("assets/targets/browse_for_folder_button.png")
    )
    image_confidence: float = Field(default=0.9, gt=0.0, le=1.0)
    image_grayscale: bool = Field(default=True)
    image_timeout_seconds: int = Field(default=3, ge=1)
    dialog_timeout_seconds: int = Field(default=60, ge=1)
    resave_poll_seconds: float = Field(default=0.1, gt=0.0)
    resave_no_change_timeout_seconds: int = Field(default=20, ge=1)
    final_resave_timeout_seconds: int = Field(default=1800, ge=1)
    file_menu_x_ratio: float = Field(default=220 / 1275, gt=0.0, lt=1.0)
    file_menu_y_ratio: float = Field(default=24 / 808, gt=0.0, lt=1.0)
    batch_resave_x_ratio: float = Field(default=248 / 1275, gt=0.0, lt=1.0)
    batch_resave_y_ratio: float = Field(default=295 / 808, gt=0.0, lt=1.0)
    choose_base_folder_title: str = Field(default="Choose base folder")
    missing_files_title: str = Field(default="files could not be found")
    content_missing_title: str = Field(default="Content Missing")
    select_target_folder_title: str = Field(default="select target folder")


class LoadNkiConfig(LaunchConfig):
    file_menu_image: Path = Field(default=Path("assets/targets/file__menu.png"))
    load_dialog_title: str = Field(default="Load")
    dialog_timeout_seconds: int = Field(default=30, ge=1)
    load_timeout_seconds: int = Field(default=120, ge=1)
    load_poll_seconds: float = Field(default=1.0, gt=0.0)
    post_open_settle_seconds: float = Field(default=2.0, gt=0.0)
    menu_image_timeout_seconds: float = Field(default=3.0, gt=0.0)
    menu_key_pause_seconds: float = Field(default=0.15, gt=0.0)
    load_menu_down_presses: int = Field(default=3, ge=1)
    file_menu_x_ratio: float = Field(default=220 / 1511, gt=0.0, lt=1.0)
    file_menu_y_ratio: float = Field(default=25 / 1109, gt=0.0, lt=1.0)
    headless_load_settle_seconds: float = Field(default=15.0, gt=0.0)
    headless_dialog_close_timeout_seconds: int = Field(default=20, ge=1)
    screenshot_name: str = Field(default="kontakt_loaded.png")
    confirmation_diff_mean: float = Field(default=10.0, ge=0.0)
    instr_nav_x_ratio: float = Field(default=630 / 1510, gt=0.0, lt=1.0)
    instr_nav_y_ratio: float = Field(default=101 / 1108, gt=0.0, lt=1.0)
    navigator_left_ratio: float = Field(default=0.0, ge=0.0, lt=1.0)
    navigator_top_ratio: float = Field(default=545 / 808, gt=0.0, lt=1.0)
    navigator_right_ratio: float = Field(default=425 / 1275, gt=0.0, lt=1.0)
    navigator_bottom_ratio: float = Field(default=705 / 808, gt=0.0, lt=1.0)
    navigator_open_timeout_seconds: float = Field(default=3.0, gt=0.0)
    navigator_open_diff_mean: float = Field(default=4.0, ge=0.0)
    navigator_confirmation_diff_mean: float = Field(default=6.0, ge=0.0)
    navigator_confirmation_timeout_seconds: float = Field(default=6.0, gt=0.0)

    @property
    def navigator_screenshot_path(self) -> Path:
        return self.artifacts_dir / "kontakt_navigator.png"

    @property
    def screenshot_path(self) -> Path:
        return self.artifacts_dir / self.screenshot_name
