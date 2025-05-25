#          •
#  ┏┓┏┓┏┓┏┓┓
#  ┗┫┣┛┛ ┗┛┃
# --┗┛-----┛------------------------------------------ (c) 2025 contributors ---
int("> " + " ".join(cmd))
        else:
            print(f"> {cmd}")
    return subprocess.run(cmd, **args)


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
