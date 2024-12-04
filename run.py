#!/usr/bin/env python3
"""
Build, flash, and run a project.
Automatically connects to the serial port and manages IO.
"""
import json
import os
from argparse import ArgumentParser
from pathlib import Path
from subprocess import check_output, run

import serial


def get_arg_parser() -> ArgumentParser:
    parser = ArgumentParser()
    parser.add_argument("project_dir", type=Path)
    parser.add_argument("input_file", type=Path)
    parser.add_argument("--target", default="riscv32imc-unknown-none-elf")
    parser.add_argument("--profile", default="release")
    return parser


def main():
    args = get_arg_parser().parse_args()

    meta = get_meta(args.project_dir)
    target_dir = Path(meta["target_directory"])
    pkg_meta = next(pkg for pkg in meta["packages"] if pkg["name"] != "aoc-common")
    artifact_dir = target_dir / args.target / args.profile
    serial_port = find_serial_port()

    print("# Compiling")
    build(args.project_dir, args.target, args.profile)
    print()

    print("# Flashing")
    flash(artifact_dir, pkg_meta["name"], serial_port)
    print()

    print("# Connecting")
    monitor(serial_port, args.input_file)
    print()


def build(proj_root: Path, target: str, profile: str):
    run(
        ["cargo", "build", "--target", target, "--profile", profile],
        cwd=proj_root,
        check=True,
    )


def get_meta(project_dir: Path) -> dict:
    out = check_output(["cargo", "metadata", "--no-deps"], cwd=project_dir)
    return json.loads(out)


def find_serial_port() -> str:
    files = os.listdir("/dev/")
    return "/dev/" + next(f for f in files if f.startswith("tty.usbmodem"))


def flash(artifact_dir: Path, pkg_name: str, serial_port: str):
    firmware_path = artifact_dir / pkg_name
    print("Flashing", firmware_path)
    run(["espflash", "flash", "--port", serial_port, str(firmware_path)], check=True)


def read_and_print_available(ser: serial.Serial):
    while True:
        out = ser.readline()
        print(out.decode(), end="")
        if not out.endswith(b"\n"):
            break


def monitor(serial_port: str, input_file: Path):
    input = input_file.read_text()

    with serial.Serial(serial_port, timeout=1) as ser:
        read_and_print_available(ser)

        print("Sending input")
        ser.write(input.encode())
        ser.write(bytes.fromhex("04"))  # End of Transmission
        ser.flush()

        read_and_print_available(ser)


if __name__ == "__main__":
    main()
