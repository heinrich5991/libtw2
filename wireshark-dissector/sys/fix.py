import re

def rindex(l, element):
    return len(l) - 1 - l[::-1].index(element)

DERIVE_RE=re.compile(r'^#\[derive\((?P<derives>[A-Za-z ,]*)\)\]')

def do(filename):
    with open(filename) as f:
        input = list(f)
    struct_def_index = input.index("pub struct wtap_packet_header {\n")
    for i, line in reversed(list(enumerate(input[:struct_def_index]))):
        m = DERIVE_RE.match(line)
        if m:
            derive_index = i
            break
        # Don't match too far apart.
        if struct_def_index - i > 10:
            break
    if not m:
        raise ValueError("fuzzy matching too far apart, could not find derive for wtap_packet_header")
    new_derives = [x.strip() for x in m.group("derives").split(", ") if x.strip() != "Debug"]
    input[derive_index] = "#[derive({})]\n".format(", ".join(new_derives))
    with open(filename, "w") as f:
        f.write("".join(input))

def main():
    import argparse
    p = argparse.ArgumentParser(description="Post-process a bindgen-generated Wireshark binding")
    p.add_argument("filename", metavar="FILENAME", help="File to post-process")
    args = p.parse_args()
    do(args.filename)

if __name__ == '__main__':
    main()
