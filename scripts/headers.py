import argparse
import glob

parser = argparse.ArgumentParser()
parser.add_argument("--fix", action="store_true", help="Write to disk.")
args = parser.parse_args();

def get_header(c: str):
    return f"""{c}         •
{c} ┏┓┏┓┏┓┏┓┓
{c} ┗┫┣┛┛ ┗┛┃
{c}--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
"""

c_dict = {
    'toml': '# ',
    'py': '# ',
    'rs': "//"
}

paths = ["crates/**", "scripts", "src", "./"]

ok = True
for key in c_dict.keys():
    for path in paths:
        for file in glob.iglob(f"{path}/*.{key}", recursive=True):
            with open(file, "r+") as f:
                head = ''.join([next(f) for _ in range(4)])
                header = get_header(c_dict[key])
                if head != header:
                    ok = False
                    if args.fix:
                        f.write(f"{header}\n{f.read()}")
                        print(f"FIX {file}")
                    else:
                        print(f"ERR {file}")

                else:
                    print(f"OK {file}")

exit(0 if ok else 1)
