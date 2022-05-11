#!/usr/bin/env python3

import glob
from pathlib import Path
import re
import subprocess
import sys
import shutil
import os
from typing import Dict, List

# ------ Parameters ------

CWD = Path(__file__).parent.resolve()
OPENTITAN_REPO = "https://github.com/harshanavkis/opentitan.git"

OPENTITAN_PATH = CWD.joinpath("target", "opentitan")
TOOLCHAIN_PATH = CWD.joinpath("target", "riscv_toolchain")

VERILATOR_SIM_PATH = OPENTITAN_PATH.joinpath(
    "build-bin/hw/top_earlgrey/Vchip_earlgrey_verilator")
VERILATOR_ROM_PATH = OPENTITAN_PATH.joinpath(
    "build-bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem")
VERILATOR_OTP_PATH = OPENTITAN_PATH.joinpath(
    "build-bin/sw/device/otp_img/otp_img_sim_verilator.vmem")

SUITE_RUNNER_PATH = CWD.joinpath("suite/.cargo/runner.sh")


# ------ Output Formatting ------

def color_print(msg: str, color_num: str, bold: bool = False) -> None:
    try:
        assert(sys.stderr.isatty())

        FNT_COLOR = subprocess.check_output(
            ['tput', 'setaf', color_num]).decode("utf-8")
        FNT_BOLD = subprocess.check_output(['tput', 'bold']).decode("utf-8")
        FNT_NORMAL = subprocess.check_output(['tput', 'sgr0']).decode("utf-8")

        if bold:
            print(f"{FNT_COLOR}{FNT_BOLD}{msg}{FNT_NORMAL}", file=sys.stderr)
        else:
            print(f"{FNT_COLOR}{msg}{FNT_NORMAL}", file=sys.stderr)
    except (FileNotFoundError, AssertionError) as _:
        print(f"{msg}", file=sys.stderr)


def info(msg: str) -> None:
    color_print(msg, '2')


def error(msg: str) -> None:
    color_print(msg, '1', bold=True)


# ------ Utility Functions ------

def run(cmd: List[str], cwd: str = str(CWD), extra_env: Dict[str, str] = {}) -> None:
    env = os.environ.copy()
    env.update(extra_env)

    env_string = []
    for k, v in extra_env.items():
        env_string.append(f"{k}={v}")

    info(f"$ {' '.join(env_string)} {' '.join(cmd)}")
    subprocess.run(cmd, cwd=cwd, env=env, check=True)


# ------ Building ------

def build_opentitan(nix_shell: str) -> None:
    install_script_path = OPENTITAN_PATH.joinpath("util", "get-toolchain.py")
    meson_script_path = OPENTITAN_PATH.joinpath("meson_init.sh")
    chip_script_path = OPENTITAN_PATH.joinpath(
        "ci", "scripts", "build-chip-verilator.sh")

    # Clone Repo
    if not OPENTITAN_PATH.exists():
        run([nix_shell, "--run",
            f"git clone {OPENTITAN_REPO} {OPENTITAN_PATH}"])
    else:
        info("Opentitan repository detected, skipping clone")

    # Install Toolchain
    if not TOOLCHAIN_PATH.exists():
        run([nix_shell, "--run",
            f"{install_script_path} --install-dir={TOOLCHAIN_PATH}"], cwd=OPENTITAN_PATH)
    else:
        info("Riscv Toolchain detected, skipping installation")

    # Run meson_init.sh
    if not VERILATOR_ROM_PATH.exists() or not VERILATOR_OTP_PATH.exists() or not VERILATOR_SIM_PATH.exists():
        run([nix_shell, "--run", f"{meson_script_path}", "--keep", "TOOLCHAIN_PATH"], cwd=OPENTITAN_PATH,
            extra_env={"TOOLCHAIN_PATH": str(TOOLCHAIN_PATH)})

    # Build rom
    if not VERILATOR_ROM_PATH.exists():
        run([nix_shell, "--run", f"ninja -C build-out sw/device/lib/testing/test_rom/test_rom_export_sim_verilator", "--keep", "TOOLCHAIN_PATH"], cwd=OPENTITAN_PATH,
            extra_env={"TOOLCHAIN_PATH": str(TOOLCHAIN_PATH)})
    else:
        info("Verilator test rom detected, skipping build")

    # Build otp
    if not VERILATOR_OTP_PATH.exists():
        run([nix_shell, "--run", f"ninja -C build-out sw/device/otp_img/otp_img_export_sim_verilator", "--keep", "TOOLCHAIN_PATH"], cwd=OPENTITAN_PATH,
            extra_env={"TOOLCHAIN_PATH": str(TOOLCHAIN_PATH)})
    else:
        info("Verilator otp image detected, skipping build")

    # Build earlgrey chip for verilator
    if not VERILATOR_SIM_PATH.exists():
        run([nix_shell, "--run", f"{chip_script_path} earlgrey", "--keep", "TOOLCHAIN_PATH"], cwd=OPENTITAN_PATH,
            extra_env={"TOOLCHAIN_PATH": str(TOOLCHAIN_PATH)})
    else:
        info("Verilator earlgrey simulator detected, skipping build")

    success = True
    if not VERILATOR_SIM_PATH.exists():
        error(f"Error: Earlgrey chip binary not found at {VERILATOR_SIM_PATH}")
        success = False
    if not VERILATOR_OTP_PATH.exists():
        error(f"Error: Opentitan OTP not found at {VERILATOR_OTP_PATH}")
        success = False
    if not VERILATOR_ROM_PATH.exists():
        error(f"Error: Opentitan ROM not found at {VERILATOR_ROM_PATH}")
        success = False

    if not success:
        error("Aborting due to previous errors")
        exit(1)


def build_suite(nix_shell: str) -> None:
    run([nix_shell, "--run", f"cd suite && cargo build --release"])


def build_cli(nix_shell: str) -> None:
    run([nix_shell, "--run", f"cd cli && cargo build --release"])


# ------ Benchmarking ------

def start_suite(nix_shell: str) -> subprocess.Popen[bytes]:
    env = os.environ.copy()
    env.update({
        "VERILATOR_SIM": str(VERILATOR_SIM_PATH),
        "VERILATOR_ROM": str(VERILATOR_ROM_PATH),
        "VERILATOR_OTP": str(VERILATOR_OTP_PATH),
    })

    info("Starting suite in background")
    return subprocess.Popen(
        [nix_shell, "--command",
            'cd suite && cargo run-verilator-opt'],
        cwd=str(CWD),
        env=env,
        stdout=subprocess.PIPE,
        stdin=subprocess.PIPE,
        preexec_fn=os.setsid
    )


def get_suite_pty(suite_process: subprocess.Popen[bytes]) -> str:
    while True:
        line = suite_process.stdout.readline().decode("utf-8")
        pty_match = re.search("UART: Created (.*) for uart0.", line)

        if pty_match:
            return pty_match.group(1)


def stop_suite(suite_process: subprocess.Popen[bytes]) -> None:
    info("Stopping suite")
    os.killpg(os.getpgid(suite_process.pid), 2)


def run_all_benchmarks(nix_shell: str, pty: str) -> None:
    files = glob.glob(str(CWD.joinpath("benchmarks")) +
                      '/**/*.bench', recursive=True)
    files_str = ' '.join(files)

    # TODO: disable raw mode, once implemented
    info("Running all benchmarks inside 'benchmarks/' folder")
    run([nix_shell, "--run",
         f"cd cli && cargo run --release -- -r --tty {pty} -f {files_str}"])


# ------ Graphing ------

# TODO: Graph results

# ------ Main Logic ------

def main():
    nix_shell = shutil.which("nix-shell", mode=os.F_OK | os.X_OK)
    if nix_shell is None:
        error("Error: could not find nix-shell")
        sys.exit(1)

    build_opentitan(nix_shell)

    suite_process = start_suite(nix_shell)
    try:
        pty = get_suite_pty(suite_process)

        run_all_benchmarks(nix_shell, pty)
    finally:
        stop_suite(suite_process)


if __name__ == "__main__":
    main()
