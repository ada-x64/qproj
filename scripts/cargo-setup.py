#!/.venv/bin/python3

import argparse
import os
import shutil
import subprocess

thispath = os.path.dirname(__file__)

parser = argparse.ArgumentParser()

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

xwin_path=os.path.abspath(os.path.join(thispath, "..", ".xwin-cache"))
if args.force or not os.path.exists(xwin_path):
    shutil.rmtree(xwin_path, ignore_errors=True)
    subprocess.run(["cargo", "install", "xwin", "--locked"])
    subprocess.run(["xwin", "--accept-license", "splat"])

bin_path = os.path.abspath(os.path.join(thispath, "..",".bin"))
if args.force or not os.path.exists(bin_path):
    shutil.rmtree(bin_path, ignore_errors=True)
    clang = shutil.which("clang-cl") or shutil.which("clang")
    llvm_ar = shutil.which("llvm-lib") or shutil.which("llvm-ar")
    lld_link = shutil.which("lld-link")
    if not clang:
        print("WARN: Cannot find clang!")
    if not llvm_ar:
        print("WARN: Cannot find llvm-ar!")
    if not lld_link:
        print("WARN: Cannot find lld_link!")

    os.mkdir(bin_path)
    os.symlink(f"{clang}", f"{bin_path}/clang-cl")
    os.symlink(f"{llvm_ar}", f"{bin_path}/llvm-lib")
    os.symlink(f"{lld_link}",f"{bin_path}/lld-link")
    subprocess.check_call([f"{bin_path}/clang-cl", "-v"])
    subprocess.check_call([f"{bin_path}/llvm-lib", "-v"])
    subprocess.check_call([f"{bin_path}/lld-link", "--version"])

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
