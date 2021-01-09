import argparse
import json
import os

import datatypes

def write_file(filename, data):
    with open(filename, "w") as f:
        f.write(data)

def generate(spec, out_dir, name):
    protocol = datatypes.load_protocol_spec(spec)

    os.makedirs(os.path.join(out_dir, "src/msg"), exist_ok=True)

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_header_enums()
        datatypes.emit_enum_module(protocol.constants, protocol.game_enumerations)
    write_file(os.path.join(out_dir, "src/enums.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_header_msg_connless(protocol.connless_messages)
        datatypes.emit_enum_connless_module("Connless", protocol.connless_messages)
    write_file(os.path.join(out_dir, "src/msg/connless.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_header_msg_game()
        datatypes.emit_enum_msg_module("Game", protocol.game_messages)
    write_file(os.path.join(out_dir, "src/msg/game.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_header_msg_system()
        datatypes.emit_enum_msg_module("System", protocol.system_messages)
    write_file(os.path.join(out_dir, "src/msg/system.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_header_snap_obj()
        datatypes.emit_enum_obj_module("SnapObj", protocol.snapshot_objects, protocol.game_flags)
    write_file(os.path.join(out_dir, "src/snap_obj.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_main_lib()
    write_file(os.path.join(out_dir, "src/lib.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_msg_module()
    write_file(os.path.join(out_dir, "src/msg/mod.rs"), emit.get())

    emit = datatypes.Emit()
    with emit:
        datatypes.emit_cargo_toml(name)
    write_file(os.path.join(out_dir, "Cargo.toml"), emit.get())

def main():
    p = argparse.ArgumentParser(description="Generate Rust protocol files for a protocol described in a JSON file")
    p.add_argument("spec", metavar="SPEC", help="JSON spec file")
    p.add_argument("out", metavar="OUT", help="Output directory")
    p.add_argument("name", metavar="NAME", help="Crate name")
    args = p.parse_args()
    with open(args.spec) as f:
        spec = json.load(f)
    generate(spec, args.out, args.name)

if __name__ == "__main__":
    main()
