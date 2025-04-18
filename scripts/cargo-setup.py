#!/.venv/bin/python3

import argparse
import os
import subprocess

parser = argparse.ArgumentParser()

parser.add_argument(
    "-x", "--xwin", action="store_true", help="Install and splat xwin cache."
)

if xwin and not os.path.exists(".xwin-cache"):
    subprocess.run(["cargo", "install", "xwin", "--locked"])
    subprocess.run(["xwin", "--accept-license", "splat"])

if bin and not os.path.exists(".bin"):
    os.rmdir(".bin")
    os.mkdir(".bin")
    clang = subprocess.check_output("which clang-cl | which clang", shell=True)
    llvm_ar = subprocess.check_output("which llvm-lib | which llvm-ar", shell=True)
    lld_link = subprocess.check_output("which lld_link", shell=True)
    if not clang:
        print("WARN: Cannot find clang!")
    if not llvm_ar:
        print("WARN: Cannot find llvm-ar!")
    if not lld_link:
        print("WARN: Cannot find lld_link!")

    os.symlink("")
    subprocess.check_call(["./.bin/"])

    subprocess.run(
        """
        if [[ -z $CLANG ]]; then echo "WARN: Cannot find clang!"; fi
        if [[ -z $LLVM_AR ]]; then echo "WARN: Cannot find llvm-ar!"; fi
        if [[ -z $LLD_LINK ]]; then echo "WARN: Cannot find lld-link!"; fi
        ln -s "$CLANG" .bin/clang-cl
        ln -s "$LLVM_AR" .bin/llvm-lib
        ln -s "$LLD_LINK" .bin/lld-link
        set -e
        ./.bin/clang-cl -v
        ./.bin/llvm-lib -v
        ./.bin/lld-link --version
        set +e
        """,
        shell=True,
    )
