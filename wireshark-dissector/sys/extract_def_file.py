import re

RE_FUNCTION_NAME=re.compile(r'^    pub fn (?P<name>[A-Za-z0-9_]+).*$')
def extract_exported_functions(lines):
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

def do(filename, output_def):
    with open(filename) as f:
        input = list(f)
    function_names = extract_exported_functions(input)
    result = ["EXPORTS\n"] + ["    {}\n".format(name) for name in function_names]
    with open(output_def, "w") as f:
        f.write("".join(result))

def main():
    import argparse
    p = argparse.ArgumentParser(description="Generate a .def file from a bindgen-generated Wireshark binding")
    p.add_argument("filename", metavar="FILENAME", help="bindgen-generated Wireshark binding .rs file")
    p.add_argument("output_def", metavar="OUTPUT_DEF", help="Path to .def file to produce")
    args = p.parse_args()
    do(args.filename, args.output_def)

if __name__ == '__main__':
    main()
