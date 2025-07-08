#  𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
#  SPDX-License-Identifier: MIT OR Apache-2.0

#  𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
#  SPDX-License-Identifier: MIT OR Apache-2.0


import argparse
import glob

parser = argparse.ArgumentParser()
parser.add_argument("--fix", action="store_true", help="Write to disk.")
args = parser.parse_args()


def get_header(c: str):
    return f"""\
{c} 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
{c} SPDX-License-Identifier: MIT OR Apache-2.0
"""


c_dict = {"toml": "# ", "py": "# ", "rs": "//"}

paths = ["libs/**/*", "scripts/**/*", "bins/**/*", "./*"]

ok = True
for key in c_dict.keys():
    for path in paths:
        for file in glob.iglob(f"{path}.{key}", recursive=True):
            with open(file, "r+") as f:
                # Read the first 4 lines to check header
                head_lines: list[str] = []
                for _ in range(4):
                    try:
                        head_lines.append(next(f))
                    except StopIteration:
                        break

                head = "".join(head_lines)
                header = get_header(c_dict[key])
                if head != header:
                    ok = False
                    if args.fix:
                        f.seek(0)
                        content = f.read()
                        f.seek(0)
                        f.write(f"{header}\n{content}")
                        print(f"🟡 FIX {file}")
                    else:
                        print(f"🔴 ERR {file}")

if ok:
    print("✅ Headers ok!")

exit_code = 0 if ok or args.fix else 1
exit(exit_code)
