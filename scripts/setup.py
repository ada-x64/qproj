#  𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
#  SPDX-License-Identifier: MIT OR Apache-2.0


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

## SYSTEM DEPS ################################################################

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

## CARGO DEPS #################################################################

bevy_lint = shutil.which("bevy_lint")
if not bevy_lint or args.force:
    # https://github.com/TheBevyFlock/bevy_cli/releases/tag/lint-v0.3.0
    subprocess.run(
        """
        rustup toolchain install nightly-2025-04-03 \
            --component rustc-dev \
            --component llvm-tools-preview

        rustup run nightly-2025-04-03 cargo install \
            --git https://github.com/TheBevyFlock/bevy_cli.git \
            --tag lint-v0.3.0 \
            --locked \
            bevy_lint
        """,
        shell=True,
    )

deny = shutil.which("cargo-deny")
if not deny or args.force:
    subprocess.run("cargo install --locked cargo-deny", shell=True)

## BUILD DEPS #################################################################

dirname = os.path.dirname(__file__)
binpath = os.path.abspath(os.path.join(dirname, "../.bin"))


def mk_symlink(cmd: str, alias: str):
    bin = shutil.which(cmd)
    if not bin:
        raise Exception(f"Could not find {cmd}!")
    subprocess.run(["mkdir", "-p", ".bin"])
    alias_path = os.path.abspath(os.path.join(binpath, alias))
    try:
        os.symlink(bin, alias_path)
    except Exception:
        pass
    return alias_path


clang_cl = (
    shutil.which("clang-cl")
    or shutil.which("clang-cl-19")
    or shutil.which("clang-cl-18")
    or mk_symlink("clang", "clang-cl")
)

llvm_ar = (
    shutil.which("llvm-ar")
    or shutil.which("llvm-ar-19")
    or shutil.which("llvm-ar-18")
    or mk_symlink("llvm-lib", "llvm-ar")
)

lld_link = (
    shutil.which("lld-link")
    or shutil.which("lld-link-19")
    or shutil.which("lld-link-18")
    or mk_symlink("lld-link", "lld-link")
)  # unsure what to put here

subprocess.check_call([clang_cl, "--version"])
subprocess.check_call([llvm_ar, "--version"])
subprocess.check_call([lld_link, "--version"])

xwin_path = os.path.abspath(os.path.join(thispath, "..", ".xwin-cache"))
if args.force or not os.path.exists(xwin_path):
    shutil.rmtree(xwin_path, ignore_errors=True)
    subprocess.run(["cargo", "install", "xwin", "--locked"])
    subprocess.run(["xwin", "--accept-license", "splat"])

env = os.path.join(os.path.join(thispath, "..", ".env"))
if args.force or not os.path.exists(env):
    with open(env, "w") as f:
        xwin_path = xwin_path + "/splat"
        dirs = os.listdir(f"{xwin_path}/sdk/include")
        cflags = ""
        for dir in dirs:
            cflags = f"{cflags} /imsvc {xwin_path}/sdk/include/{dir}"
        cflags = f"{cflags} /imsvc {xwin_path}/crt/include"
        rustflags = " ".join(
            [
                f"-Clink-arg=/libpath:{xwin_path}/crt/lib/x86_64",
                f"-Clink-arg=/libpath:{xwin_path}/sdk/lib/um/x86_64",
                f"-Clink-arg=/libpath:{xwin_path}/sdk/lib/ucrt/x86_64",
            ]
        )

        vars = [
            f"XWIN_DIR='{xwin_path}'",
            f"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS='{rustflags}'",
            f"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER='{lld_link}'",
            f"CC_x86_64_pc_windows_msvc='{clang_cl}'",
            f"CXX_x86_64_pc_windows_msvc='{clang_cl}'",
            f"AR_x86_64_pc_windows_msvc='{llvm_ar}'",
            f"CFLAGS_x86_64_pc_windows_msvc='{cflags}'",
            f"TRACY_CLIENT_SYS_CXXFLAGS_x86_64_pc_windows_msvc='{cflags}'",
            "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER='x86_64-w64-mingw32-gcc'",
            f"PATH=$PATH:{binpath}",
        ]
        f.write("\n".join(vars))


## HOOKS ######################################################################
with open(".git/hooks/pre-push", "w+") as f:
    f.write("#!/bin/bash\njust check\n")

IMPORTANT = "\x1b[1m!IMPORTANT!\x1b[22m"
print(
    """
-------------------------------------------------------------------------------
\x1b[1mSuccessfully set up.\x1b[22m

{IMPORTANT} You will need to set Clang as your C(++) compiler. If you are on a
Debian-based distro, run the following:

    update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100;
    update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100;
""".format(IMPORTANT=IMPORTANT)
)
