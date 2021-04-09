import re

def rindex(l, element):
    return len(l) - 1 - l[::-1].index(element)

DERIVE_RE=re.compile(r'^#\[derive\((?P<derives>[A-Za-z ,]*)\)\]')

TYPE_REPLACEMENTS = {line[:17]: line for line in """\
pub type size_t = usize;
pub type guint8 = u8;
pub type guint16 = u16;
pub type gint32 = i32;
pub type guint32 = u32;
pub type gint64 = i64;
pub type guint64 = u64;
pub type gsize = usize;
pub type __time_t = i64;
pub type time_t = __time_t;
""".splitlines()}
#2345678901234567

def replace_types(line):
    if line[:17] in TYPE_REPLACEMENTS:
        return TYPE_REPLACEMENTS[line[:17]]
    return line

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
    input = [replace_types(line) for line in input]
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
