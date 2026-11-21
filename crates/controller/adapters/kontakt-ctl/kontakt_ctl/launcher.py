import subprocess

from loguru import logger


def launch_kontakt(exe_path: str) -> subprocess.Popen:
    logger.info("Launching {}", exe_path)
    process = subprocess.Popen([exe_path])
    logger.info("Kontakt PID={}", process.pid)
    return process


def terminate_kontakt(image_name: str = "Kontakt 7.exe") -> None:
    logger.info("Terminating {}", image_name)
    subprocess.run(["taskkill", "/IM", image_name, "/F"], check=False)
