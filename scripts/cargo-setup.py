#!/.venv/bin/python3

import argparse
import os
import shutil
import subprocess

parser = argparse.ArgumentParser()

parser.add_argument(
    "-x", "--xwin", action="store_true", help="Install and splat xwin cache."
)
parser.add_argument("-b", "--bin", action="store_true", help="Link binaries.")
parser.add_argument(
    "-D",
    "--no-deps",
    action="store_true",
    help="Do not use apt to install dependencies.",
)
parser.add_argument(
    "-f",
    "--force",
    action="store_true",
    help="Ignore existing setup and rebuild.",
)

args = parser.parse_args()
print(args)

if not args.no_deps:
    subprocess.run(
        "sudo apt update && sudo apt install -y pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 llvm clang lld",
        shell=True,
    )

if args.xwin and (args.force or not os.path.exists(".xwin-cache")):
    try:
        os.rmdir(".xwin")
    except:
        pass
    subprocess.run(["cargo", "install", "xwin", "--locked"])
    subprocess.run(["xwin", "--accept-license", "splat"])

if args.bin and (args.force or not os.path.exists(".bin")):
    shutil.rmtree(".bin", ignore_errors=True)
    clang = shutil.which("clang-cl") or shutil.which("clang")
    llvm_ar = shutil.which("llvm-lib") or shutil.which("llvm-ar")
    lld_link = shutil.which("lld-link")
    if not clang:
        print("WARN: Cannot find clang!")
    if not llvm_ar:
        print("WARN: Cannot find llvm-ar!")
    if not lld_link:
        print("WARN: Cannot find lld_link!")

    os.mkdir(".bin")
    os.symlink(f"{clang}", ".bin/clang-cl")
    os.symlink(f"{llvm_ar}", ".bin/llvm-lib")
    os.symlink(f"{lld_link}", ".bin/lld-link")
    subprocess.check_call(["./.bin/clang-cl", "-v"])
    subprocess.check_call(["./.bin/llvm-lib", "-v"])
    subprocess.check_call(["./.bin/lld-link", "--version"])

IMPORTANT = "\x1b[1m!IMPORTANT!\x1b[22m"
print(
    """
-------------------------------------------------------------------------------
\x1b[1mSuccessfully set up.\x1b[22m
To build, run `cargo wsl.`

{IMPORTANT} You will need to set Clang as your C(++) compiler. If you are on a
Debian-based distro, run the following:

    update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100;
    update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100;
""".format(
        IMPORTANT=IMPORTANT
    )
)
