#          •
#  ┏┓┏┓┏┓┏┓┓
#  ┗┫┣┛┛ ┗┛┃
# --┗┛-----┛------------------------------------------ (c) 2025 contributors ---

import argparse
import os
import subprocess
import xxhash
import tomllib
import common

# SETUP #######################################################################

temp = None
try:
    temp = (
        subprocess.check_output(
            "wslpath -au $(pwsh.exe -c 'echo $env:TEMP')",
            shell=True,
            stderr=subprocess.DEVNULL,
        )
        .decode("ascii")
        .replace("\r\n", "")
    )
except:
    if not os.environ.get("HOSTPATH"):
        print("WARN: wsl not detected and HOSTPATH not set.")

env_vars_desc = {
    "CARGO_CMD": {
        "description": "The cargo command to execute.",
        "default": "build",
    },
    "HOSTPATH": {
        "description": 'Directory for the final executable in "wslpath -au" style.',
        "default": temp or "",
    },
    "SYNC_HOST": {
        "description": "Destination for the final executable. Must be in Unix format.",
        "default": temp or "",
    },
    "SYNC_PING_ADDRESS": {
        "description": "Websocket address to ping with list of transferred files.",
        "default": None,
    },
    "CARGO_BUILD_TARGET": {
        "description": "Build target for Cargo.",
        "default": "debug",
    },
    "SCP_FLAGS": {
        "description": "If SSH is detected, these flags will be added to the SCP command when syncing build files.",
        "default": None,
    },
}


# PARSE #######################################################################
class CustomFormatter(
    argparse.ArgumentDefaultsHelpFormatter, argparse.RawDescriptionHelpFormatter
):
    pass


epilog = "Accepts the following environment variables:\n"
for name, values in env_vars_desc.items():
    epilog += f"  {name}:\t\t{values.get('description')}\n\t\t\t(default: {values.get('default')})\n"

env = os.environ
parser = argparse.ArgumentParser(
    prog="wsl",
    description="Builds the project assuming you're on WSL. By default will compile with `-Fdebug -Fdev -Finspector`. Packaged files are checksummed and only changed files are synced. If SSH is detected, there will be some extra steps involved, and dynamic linking will be disabled.",
    epilog=epilog,
    formatter_class=CustomFormatter,
)
parser.add_argument(
    "-b", "--no-build", action="store_true", help="Do not build the project."
)
parser.add_argument(
    "-s", "--no-sync", action="store_true", help="Do not sync the project."
)
parser.add_argument(
    "-H", "--no-hash", action="store_true", help="Do not check for changes."
)
parser.add_argument(
    "-r", "--no-run", action="store_true", help="Do not run the project."
)
parser.add_argument(
    "-t",
    "--trace",
    action="store_true",
    help="Runs with `-Fbevy/trace_tracy --release` and removes `-Fdev`. On Windows this requires running the built application as an administrator.",
)
parser.add_argument("wsl", help="(dummy argument)")
parser.add_argument(
    "-f",
    "--no-default-features",
    action="store_true",
    help="By default, cargo wsl will run with `-Fdebug -Fdev -Finspector`. Pass this flag if you want to disable that behavior.",
)
parser.add_argument(
    "-v", "--verbose", action="count", help="Run verbosely.", default=0
)

ssh_connection = os.environ.get("SSH_CONNECTION")
if ssh_connection:
    print("SSH detected.")

args = common.parse_with_forward(parser, "cargo")
if not args.no_default_features:
    args.forward = args.forward + [
        "-Fdebug",
        "-Fdev",
        "-Finspector",
    ]
if args.trace:
    args.forward = args.forward + ["-Fbevy/trace_tracy", "--release"]

if ssh_connection or args.trace and "-Fdev" in args.forward:
    args.forward.remove("-Fdev")


def print_and_run(cmd: str | list[str], **shargs: object):  # type: ignore
    return common.print_and_run(cmd, args.verbose, **shargs)  # type: ignore


env_vars: dict[str, str] = {}
for key in env_vars_desc:
    env_vars[key] = (
        os.environ.get(key) or env_vars_desc[key].get("default") or ""
    )

pkg_name = "bevy_game"
bin_name = "bevy_game.exe"
pdb_name = "bevy_game.pdb"
with open("Cargo.toml", "rb") as f:
    toml = tomllib.load(f)
    pkg_name = toml.get("package").get("name")  # type:ignore
    bin_name = pkg_name + ".exe"
    pdb_name = pkg_name + ".pdb"

profile = (
    "release"
    if "-r" in args.forward or "--release" in args.forward
    else "debug"
)
target_dir = os.path.abspath(  # type: ignore
    os.path.join("target", env_vars["CARGO_BUILD_TARGET"], profile)  # type: ignore
)


if args.verbose > 2:
    print(f"ARGS:\n{args}")
    print(f"ENV_VARS:\n{env_vars}")


# BUILD #######################################################################

start_path = None
if not args.no_build:
    print_and_run(
        [
            "cargo",
            env_vars["CARGO_CMD"],
            *args.forward,
        ]
    ).check_returncode()  # type:ignore

    bin_path = os.path.join(env_vars["HOSTPATH"], pkg_name, pkg_name + ".exe")
    bin_path = (
        subprocess.check_output(f"wslpath -aw {bin_path}", shell=True)
        .decode()
        .replace("\n", "")
    )

    winpath = "$env:PATH"
    if "-Fdev" in args.forward:
        try:
            rustlibs = (
                subprocess.check_output(
                    [
                        "rustc",
                        f"--target={env_vars['CARGO_BUILD_TARGET']}",
                        "--print",
                        "target-libdir",
                    ]
                )
                .decode()
                .strip()
            )
            rustlibs = (
                subprocess.check_output(["wslpath", "-aw", rustlibs])
                .decode()
                .strip()
            )
            deps = os.path.abspath(os.path.join(target_dir, "deps"))
            deps = (
                subprocess.check_output(["wslpath", "-aw", deps])
                .decode()
                .strip()
            )
            winpath = f"$env:PATH;{rustlibs};{deps};"

        except:
            print("WARN: Could not generate path to dynamic libraries!")

    runas = "-Verb RunAs" if args.trace else "-NoNewWindow"
    start = """\
    Start-Process {runas} -Wait -Environment @{{
    RUST_LOG="{RUST_LOG}"
    DEBUG_LEVEL="{DEBUG_LEVEL}"
    RUST_BACKTRACE="{RUST_BACKTRACE}"
    PATH="{winpath}"
    }} -FilePath "{bin_path}" \
    2>&1 | Tee-Object -FilePath run.log\
    """.format(
        RUST_LOG=os.environ.get("RUST_LOG"),
        DEBUG_LEVEL=os.environ.get("DEBUG_LEVEL"),
        RUST_BACKTRACE=os.environ.get("RUST_BACKTRACE"),
        bin_path=bin_path,
        runas=runas,
        winpath=winpath,
    )
    if args.verbose > 1:
        print(f"start.ps1:\n{start}")
    start_path = os.path.join(target_dir, "start.ps1")
    with open(start_path, "w") as f:
        f.write(start)
else:
    print("Not building.")


# HASH #########################################################################
hashfile_path = os.path.join(target_dir, ".hash")


def do_hash(build_files: list[str]):
    # generate checksum
    mismatched = ()
    hashfile = ""
    for file in build_files:
        with open(file, "rb") as f:
            hash = xxhash.xxh64_hexdigest(f.read())
            hashfile += f"{file} {hash}\n"

    for root, _subdirs, files in os.walk("assets"):
        for file in files:
            path = os.path.join(root, file)
            with open(path, "rb") as f:
                hash = xxhash.xxh64_hexdigest(f.read())
                hashfile += f"{path} {hash}\n"

    # check the checksum
    if os.path.exists(hashfile_path):
        with open(hashfile_path) as f:
            zipped = zip(hashfile.splitlines(), f.readlines())
            if args.verbose > 1:
                print(
                    "Comparing hashes:\n",
                    "\n".join(
                        list(
                            map(
                                lambda t: f"{t[0].split()[0]} {t[0].split()[1]} {t[1].split()[1]}",
                                zipped,
                            )
                        )
                    ),
                )
            filtered = filter(lambda tuple: tuple[0] != tuple[1], zipped)
            mismatched = list(map(lambda tuple: tuple[0].split()[0], filtered))
    else:
        mismatched = list(map(lambda l: l.split()[0], hashfile.splitlines()))

    # overwrite it
    with open(hashfile_path, "w") as f:
        f.write(hashfile)

    return mismatched


# SYNC #########################################################################
if not args.no_sync:
    exec_path = os.path.join(target_dir, bin_name)
    pdb_path = os.path.join(target_dir, pdb_name)
    build_files = [start_path or "", exec_path, pdb_path]

    if args.no_hash:
        print("Not hashing.")
        files_to_sync = build_files
        for root, subdirs, files in os.walk("assets"):
            for file in files:
                path = os.path.join(root, file)
                files_to_sync.append(path)
    else:
        files_to_sync = do_hash(build_files)
    if args.verbose > 1:
        print("Mismatched files:", files_to_sync)

    # sync
    files_to_sync.append(hashfile_path)
    destination = os.path.join(env_vars["HOSTPATH"], pkg_name)
    sync_script = ""

    for local_file in files_to_sync:
        remote_file = local_file.replace(target_dir + "/", "")
        filepath = os.path.join(destination, remote_file)
        if ssh_connection:
            sync_script += f"mkdir {os.path.dirname(filepath)}\n"
            sync_script += f"put -R {local_file} {filepath}\n"
        else:
            sync_script += f"mkdir -p {os.path.dirname(filepath)}\n"
            sync_script += f"cp {local_file} {filepath}\n"

    if ssh_connection:
        host = env_vars["SYNC_HOST"]
        if not host:
            user = os.environ.get("USER")
            ssh_connection = ssh_connection.split(" ")
            ip = ssh_connection[0]
            port = ssh_connection[1]
            host = f"{user}@{ip}:{port}"

        sync_script = f"sftp sftp://{host}/{env_vars["HOSTPATH"]} <<EOF\n{sync_script}\nEOF"
        print_and_run(["bash", "-c", sync_script])
    else:
        print_and_run(["bash", "-c", sync_script])

    if env_vars["SYNC_PING_ADDRESS"]:
        remote_files = map(
            lambda f: f.replace(target_dir + "/", ""), files_to_sync
        )
        print_and_run(
            f'echo qproj sent {" ".join(remote_files)} | websocat {env_vars["SYNC_PING_ADDRESS"]}',
            shell=True,
        )
else:
    print("Not syncing.")

# RUN #########################################################################
if not args.no_run and not ssh_connection:
    start_path = os.path.join(env_vars["HOSTPATH"], pkg_name, "start.ps1")
    start_path = (
        subprocess.check_output(f"wslpath -aw {start_path}", shell=True)
        .decode()
        .replace("\n", "")
    )
    tty = subprocess.check_output("wslpath -aw $(tty)", shell=True).decode()
    subprocess.call(["pwsh.exe", start_path])
else:
    print("Not running.")
