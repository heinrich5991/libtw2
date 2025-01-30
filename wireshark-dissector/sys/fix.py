import os.path
import re
import subprocess
import sys

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

def extract_exported_functions(library):
    """
    Returns the list of all exported functions from the shared object specified
    by the file path `library`.
    """
    output = subprocess.check_output(["nm", "-D", library], encoding="utf8")
    symbols = [line.split() for line in output.splitlines()]
    return [name for (*_, type_, name) in symbols if type_ == "T"]

RE_FUNCTION_NAME=re.compile(r'^    pub fn (?P<name>[A-Za-z0-9_]+).*$')
def annotate_imported_functions(lines, exported_functions):
    result = []
    extern_c = []
    library = None
    for line in lines:
        if not extern_c:
            if line == 'extern "C" {\n':
                extern_c.append(line)
            else:
                result.append(line)
        else:
            m = RE_FUNCTION_NAME.match(line)
            extern_c.append(line)
            if line == "}\n":
                result.append("#[cfg_attr(windows, link(name = \"lib{}\", kind = \"raw-dylib\"))]\n".format(library))
                result += extern_c
                extern_c = []
                library = None
            elif m:
                function = m.group("name")
                if library is None:
                    library = exported_functions[function]
                elif library != exported_functions[function]:
                    raise RuntimeError("extern \"C\" block has functions from two different libraries, can't annotate")
    return result

def do(filename, libs_path, libraries):
    with open(filename) as f:
        input = list(f)
    input = [replace_types(line) for line in input]

    exported_functions_by_library = [(library, extract_exported_functions(os.path.join(libs_path, "lib{}.so".format(library)))) for library in libraries]
    exported_functions = {}
    for library, functions in exported_functions_by_library:
        for function in functions:
            if function in exported_functions:
                print("Found symbol {} in two different libraries: {} and {}".format(function, exported_functions[function], library), file=sys.stderr)
                return 1
            exported_functions[function] = library

    input = annotate_imported_functions(input, exported_functions)
    with open(filename, "w") as f:
        f.write("".join(input))

def main():
    import argparse
    p = argparse.ArgumentParser(description="Post-process a bindgen-generated Wireshark binding. Only works on Linux and requires the `nm` tool.")
    p.add_argument("filename", metavar="FILENAME", help="File to post-process")
    p.add_argument("libs_path", metavar="LIBS_PATH", help="Path to the libraries to take the symbols from")
    p.add_argument("libraries", metavar="LIBRARY", nargs="+", help="Library names (without lib prefix and without .so suffix)")
    args = p.parse_args()
    sys.exit(do(args.filename, args.libs_path, args.libraries))

if __name__ == '__main__':
    main()
