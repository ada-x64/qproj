#  𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
#  SPDX-License-Identifier: MIT OR Apache-2.0

#  𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
#  SPDX-License-Identifier: MIT OR Apache-2.0


import shutil
import subprocess
import requests
import common
import argparse

parser = argparse.ArgumentParser(
    prog="ci",
    description="Runs nektos/act to test CI.`",
)

parser.add_argument("--act", action="store", help="Act command to execute.")
parser.add_argument(
    "--no-install",
    action="store_true",
    help="Do not install act if it is not present.",
)

args = common.parse_with_forward(parser, "act")

act: str | None = args.act or shutil.which("act") or shutil.which(".bin/act")


if not act and not args.no_install:
    print("Installing nektos/act to .bin...")
    res = requests.get(
        "https://raw.githubusercontent.com/nektos/act/master/install.sh"
    )

    subprocess.call(["mkdir", "-p", ".bin"])
    with open("./.bin/install-act.sh", mode="x") as f:
        f.write(res.text)
    subprocess.call(["bash", ".bin/install-act.sh", "-b", ".bin"])
    act = ".bin/act"

if not act:
    print("Could not find act! Exiting.")
    exit(1)

# TODO: This could be made more flexible with yaml parsing.
image = "ubuntu-24.04=ghcr.io/catthehacker/ubuntu:act-24.04"
with open("ci.log", mode="w") as f:
    print("Running...\nSee ci.log")
    subprocess.run(
        f"sudo {act} -P {image} --env-file='' {' '.join(args.forward)} 2>&1 | tee ci.log",
        shell=True,
    )
