import os.path
import re
import subprocess
import sys

RE_FUNCTION_NAME=re.compile(r'^    pub fn (?P<name>[A-Za-z0-9_]+).*$')
def extract_imported_functions(lines):
    result = []
    in_extern_c = False
    for line in lines:
        m = RE_FUNCTION_NAME.match(line)
        if line == 'extern "C" {\n':
            in_extern_c = True
        elif line == "}\n":
            in_extern_c = False
        elif in_extern_c and m:
            result.append(m.group("name"))
    return sorted(result)

def extract_exported_functions(library):
    """
    Returns the list of all exported functions from the shared object specified
    by the file path `library`.
    """
    output = subprocess.check_output(["nm", "-D", library], encoding="utf8")
    symbols = [line.split() for line in output.splitlines()]
    return [name for (*_, type_, name) in symbols if type_ == "T"]

def write_def_file(filename, symbols):
    result = ["EXPORTS\n"] + ["    {}\n".format(name) for name in symbols]
    with open(filename, "w") as f:
        f.write("".join(result))

def do(filename, libs_path, def_path, libraries):
    with open(filename) as f:
        input = list(f)
    wanted_function_names = extract_imported_functions(input)
    have_function_names = {}
    for library in libraries:
        library_path = os.path.join(libs_path, "lib{}.so".format(library))
        for symbol in extract_exported_functions(library_path):
            if symbol in have_function_names:
                print("Found symbol {} in two different libraries: {} and {}".format(symbol, have_function_names[symbol], library), file=sys.stderr)
                return 1
            have_function_names[symbol] = library

    missing_function_names = set(wanted_function_names) - set(have_function_names)
    if missing_function_names:
        print("Can't find the following symbol(s) in any of the given libraries: {}".format(", ".join(sorted(missing_function_names))))
        return 1

    imports = {library: [] for library in libraries}
    for symbol in wanted_function_names:
        imports[have_function_names[symbol]].append(symbol)

    for library in libraries:
        write_def_file(os.path.join(def_path, "lib{}.def".format(library)), sorted(imports[library]))

def main():
    import argparse
    p = argparse.ArgumentParser(description="Generates .def files from a bindgen-generated Wireshark binding and Wireshark libraries. Only works on Linux and requires the `nm` tool.")
    p.add_argument("filename", metavar="FILENAME", help="bindgen-generated Wireshark binding .rs file")
    p.add_argument("libs_path", metavar="LIBS_PATH", help="Path to the libraries to take the symbols from")
    p.add_argument("def_path", metavar="DEF_PATH", help="Output path for the .def files")
    p.add_argument("libraries", metavar="LIBRARY", nargs="+", help="Library names (without lib prefix and without .so suffix)")
    args = p.parse_args()
    sys.exit(do(args.filename, args.libs_path, args.def_path, args.libraries))

if __name__ == '__main__':
    main()
