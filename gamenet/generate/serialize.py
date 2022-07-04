import loader

import argparse
import json

class JsonEncodeError(ValueError):
    pass

def serialize_json(obj, compact_patterns, **kwargs):
    return "".join(list(_serialize_json_iter(obj, loc=(), compact_patterns=compact_patterns, **kwargs)))

def loc_matches(loc, pattern):
    if len(pattern) > len(loc):
        return False
    for l, p in zip(loc, pattern):
        if p is not None and l != p:
            return False
    return True

def loc_matches_any(loc, patterns):
    return any(loc_matches(loc, pattern) for pattern in patterns)

def _format_loc(loc):
    return ".".join(loc)

def _serialize_json_iter(obj, loc, compact_patterns, **kwargs):
    indent = len(loc)
    compact = loc_matches_any(loc, compact_patterns)
    if isinstance(obj, dict):
        yield "{"
        empty = True
        for k, v in obj.items():
            if not empty:
                if compact:
                    yield ", "
                else:
                    yield ","
            if not compact:
                yield "\n" + "\t" * (indent + 1)
            empty = False
            if not isinstance(k, str):
                raise JsonEncodeError("JSON disallows non-string keys: {!r} at {}".format(k, _format_loc(loc)))
            yield from _serialize_json_iter(k, loc, compact_patterns, **kwargs)
            yield ": "
            yield from _serialize_json_iter(v, loc + (k,), compact_patterns, **kwargs)
        if compact or empty:
            yield "}"
        else:
            yield "\n" + "\t" * indent + "}"
    elif isinstance(obj, (list, tuple)):
        yield "["
        empty = True
        for i, el in enumerate(obj):
            if not empty:
                if compact:
                    yield ", "
                else:
                    yield ","
            if not compact:
                yield "\n" + "\t" * (indent + 1)
            empty = False
            yield from _serialize_json_iter(el, loc + (str(i),), compact_patterns, **kwargs)
        if compact or empty:
            yield "]"
        else:
            yield "\n" + "\t" * indent + "]"
    elif obj is None or isinstance(obj, (bool, float, int, str)):
        yield json.dumps(obj, **kwargs)
    else:
        raise JsonEncodeError("Unserializable object {!r} at {}".format(obj, _format_loc(loc)))


def main():
    p = argparse.ArgumentParser(description="Generate protocol specs from Teeworlds-style network.py")
    p.add_argument("--version", choices="0.5 0.6 0.7 ddnet-15.2.5 ddnet-16.2 none".split(), help="Force version for fixup instead of heuristically guessing a version")
    p.add_argument("network_py", help="Path to network.py")
    args = p.parse_args()
    version = args.version
    if version is None:
        version = "auto"
    elif version == "none":
        version = None
    network = loader.load_network(args.network_py, version)
    serialized = {
        "constants": [e.serialize() for e in network.Constants],
        "game_enumerations": [e.serialize() for e in network.Enums],
        "game_flags": [e.serialize() for e in network.Flags],
        "game_messages": [e.serialize() for e in network.Messages],
        "snapshot_objects": [e.serialize() for e in network.Objects],
        "system_messages": [e.serialize() for e in network.System],
        "connless_messages": [e.serialize() for e in network.Connless],
    }
    serialized_json = serialize_json(serialized, compact_patterns=[
        ("constants", None),
        (None, None, "id"),
        (None, None, "name"),
        (None, None, "super"),
        (None, None, "values", None),
        (None, None, "members", None),
    ])
    print(serialized_json)

if __name__ == "__main__":
    main()
