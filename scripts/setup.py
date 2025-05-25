#          •
#  ┏┓┏┓┏┓┏┓┓
#  ┗┫┣┛┛ ┗┛┃
# --┗┛-----┛------------------------------------------ (c) 2025 contributors ---

import argparse
import os
import shutil
import subprocess
import os_release  # type: ignore

thispath = os.path.dirname(__file__)

parser = argparse.ArgumentParser()

parser.add_argument(
    "-D",
    "--no-deps",
    action="store_true",
    help="Do not install system dependencies.",
)
parser.add_argument(
    "-f",
    "--force",
    action="store_true",
    help="Ignore existing setup and rebuild.",
)
parser.add_argument(
    "-y",
    "--non-interactive",
    action="store_true",
    help="Install system deps non-interactively.",
)

args = parser.parse_args()
print(args)

if not args.no_deps:
    os_like: tuple[str]
    if os.environ.get("CI") == "true":
        os_like = ("ubuntu",)
    else:
        release = os_release.current_release()
        os_like = release.id_like or release.id

    if "debian" in os_like or "ubuntu" in os_like:
        y = "-y" if args.non_interactive else ""
        subprocess.run(
            f"sudo apt update && sudo apt install {y} mold pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 llvm clang lld",
            shell=True,
        )
    elif "arch" in os_like:
        confirm = "--noconfirm" if args.non_interactive else "--confirm"
        subprocess.run(
            f"sudo pacman -S {confirm} mold clang lld libx11 pkgconf alsa-lib",
            shell=True,
        )
        print(
            "You may need some more packages.\nSee https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md#arch--manjaro"
        )
    elif "fedora" in os_like:
        # i don't use dnf so idk if this works :)
        y = "-y" if args.non_interactive else ""
        subprocess.run(
            f"sudo dnf {y} install mold lld gcc-c++ libX11-devel alsa-lib-devel systemd-devel",
            shell=True,
        )
        print(
            "You may need some more packages.\nSee https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md#fedora"
        )
    else:
        print(
            "Could not determine packages to install.\nSee https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md "
        )

bevy_lint = shutil.which("bevy_lint")
if not bevy_lint or args.force:
    subprocess.run(
        "cargo install --git https://github.com/TheBevyFlock/bevy_cli --tag cli-v0.1.0-alpha.1 --locked bevy_cli",
        shell=True,
    )

xwin_path = os.path.abspath(os.path.join(thispath, "..", ".xwin-cache"))
if args.force or not os.path.exists(xwin_path):
    shutil.rmtree(xwin_path, ignore_errors=True)
    subprocess.run(["cargo", "install", "xwin", "--locked"])
    subprocess.run(["xwin", "--accept-license", "splat"])

bin_path = os.path.abspath(os.path.join(thispath, "..", ".bin"))
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
        print("WARN: Cannot find lld-link!")

    os.mkdir(bin_path)
    os.symlink(f"{clang}", f"{bin_path}/clang-cl")
    os.symlink(f"{llvm_ar}", f"{bin_path}/llvm-lib")
    os.symlink(f"{lld_link}", f"{bin_path}/lld-link")
    subprocess.check_call([f"{bin_path}/clang-cl", "-v"])
    subprocess.check_call([f"{bin_path}/llvm-lib", "-v"])
    subprocess.check_call([f"{bin_path}/lld-link", "--version"])

# hooks
with open(".git/hooks/pre-push", "w+") as f:
    f.write("#!/bin/bash\njust check\n")

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
