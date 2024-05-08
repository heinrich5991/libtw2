import re

def rindex(l, element):
    return len(l) - 1 - l[::-1].index(element)

# 17 is the unique prefix :)
TYPE_REPLACEMENTS = {line[:17]: line for line in """\
pub type size_t = usize;
pub type gint8 = i8;
pub type guint8 = u8;
pub type gint16 = i16;
pub type guint16 = u16;
pub type gint32 = i32;
pub type guint32 = u32;
pub type gint64 = i64;
pub type guint64 = u64;
pub type gsize = usize;
pub type __time_t = i64;
pub type time_t = __time_t;
pub type __uint64_t = u64;
""".splitlines()}
#2345678901234567

def replace_types(line):
    if line[:17] in TYPE_REPLACEMENTS:
        return TYPE_REPLACEMENTS[line[:17]]
    return line

def do(filename):
    with open(filename) as f:
        input = list(f)
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
