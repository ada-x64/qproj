import subprocess
import sys


def print_and_run(cmd: list[str], verbose: int):
    if verbose > 0:
        print("> " + " ".join(cmd))
    subprocess.run(cmd)


def parse_with_forward(parser, subprocess: str):
    parser.add_argument(
        "--",
        help=f"Arguments passed after the '--' will be forwarded directly to {subprocess}",
        dest="forward",
    )
    argv = sys.argv[1:]
    idx = argv.index("--") if "--" in argv else -1
    unparsed = argv[idx + 1 :] if idx >= 0 else None
    argv = argv[:idx] if idx >= 0 else argv
    args = parser.parse_args(argv)
    args.forward = unparsed or []
    return args
