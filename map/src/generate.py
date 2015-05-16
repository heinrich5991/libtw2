import re

ITEMS = [
    (0, "version", [
        [],
    ]),
    (1, "info", [
        ["author", "map_version", "credits", "license"],
    ]),
    (2, "image", [
        ["width", "height", "external", "name", "data"],
        ["format"],
    ]),
    (3, "envelope", [
        ["channels", "start_points", "num_points", "name[8]"],
        ["synchronized"],
    ]),
    (4, "group", [
        ["offset_x", "offset_y", "parallax_x", "parallax_y", "start_layer", "num_layers"],
        ["use_clipping", "clip_x", "clip_y", "clip_w", "clip_h"],
    ]),
    (5, "layer", [
        ["type_", "flags"],
    ]),
    (6, "envpoints", [
        ["time", "curvetype", "values[4]"],
    ]),
]

header = """
extern crate datafile;

use datafile::UnsafeOnlyI32;

use std::mem;

pub trait MapItem: UnsafeOnlyI32 {
    fn version(unused_self: Option<Self>) -> i32;
    fn offset(unused_self: Option<Self>) -> usize;
}

pub trait MapItemExt {
    fn len(unused_self: Option<Self>) -> usize;
    fn sum_len(unused_self: Option<Self>) -> usize;

    fn from_slice(slice: &[i32]) -> Option<&Self>;
    //fn from_slice_mut(slice: &mut [i32]) -> Option<&mut Self>;
}

impl<T:MapItem> MapItemExt for T {
    fn len(_: Option<T>) -> usize {
        mem::size_of::<T>() / mem::size_of::<i32>()
    }
    fn sum_len(_: Option<T>) -> usize {
        MapItem::offset(None::<T>) + MapItemExt::len(None::<T>)
    }

    fn from_slice(slice: &[i32]) -> Option<&T> {
        if slice.len() < MapItemExt::sum_len(None::<T>) {
            return None;
        }
        if slice[0] < MapItem::version(None::<T>) {
            return None;
        }
        let result: &[i32] = &slice[MapItem::offset(None::<T>)..MapItemExt::sum_len(None::<T>)];
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<T>());
        Some(unsafe { &*(result.as_ptr() as *const T) })
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct MapItemCommonV0 {
    pub version: i32,
}

impl UnsafeOnlyI32 for MapItemCommonV0 { }
impl MapItem for MapItemCommonV0 { fn version(_: Option<MapItemCommonV0>) -> i32 { 0 } fn offset(_: Option<MapItemCommonV0>) -> usize { 0 } }
"""

def make_items(items):
    MEMBER_NORMAL=re.compile(r'^(?P<name>[a-z_]+)$')
    MEMBER_ARRAY=re.compile(r'^(?P<name>[a-z_]+)\[(?P<size>[1-9][0-9]*)\]$')

    result = []
    for (type_id, name, versions) in items:
        result_versions = []
        for version in versions:
            result_version = []
            for member in version:
                m = MEMBER_NORMAL.match(member)
                if m is not None:
                    result_version.append((m.group('name'), None))
                else:
                    m = MEMBER_ARRAY.match(member)
                    if m is not None:
                        result_version.append((m.group('name'), int(m.group('size'))))
                    else:
                        raise ValueError("Invalid member '{}'.".format(member))
            result_versions.append(result_version)
        result.append((type_id, name, result_versions))

    return result

def struct_name(name, i):
    return "MapItem{}V{}".format(name.title().replace('_', ''), i + 1)

def generate_header(items):
    return header

def generate_constants(items):
    result = []
    for (type_id, name, _) in items:
        if type_id is not None:
            result.append("pub static MAP_ITEMTYPE_{}: u16 = {};".format(name.upper(), type_id))
    result.append("")
    return "\n".join(result)

def generate_structs(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("#[derive(Clone, Copy, Debug)]")
            result.append("#[repr(packed, C)]")
            if version:
                result.append("pub struct {s} {{".format(s=struct_name(name, i)))
                for (member, size) in version:
                    if size is None:
                        result.append("    pub {}: i32,".format(member))
                    else:
                        result.append("    pub {}: [i32; {}],".format(member, size))
                result.append("}")
            else:
                result.append("pub struct {s};".format(s=struct_name(name, i)))
            result.append("")
    return "\n".join(result)

def generate_impl_unsafe_i32_only(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("impl UnsafeOnlyI32 for {s} {{ }}".format(s=struct_name(name, i)))
    result.append("")
    return "\n".join(result)

def generate_impl_map_item(items):
    result = []
    for (_, name, versions) in items:
        offset = 1
        for (i, version) in enumerate(versions):
            result.append("impl MapItem for {s} {{ fn version(_: Option<{s}>) -> i32 {{ {v} }} fn offset(_: Option<{s}>) -> usize {{ {o} }} }}".format(s=struct_name(name, i), v=i+1, o=offset))
            for (member, size) in version:
                if size is None:
                    offset += 1
                else:
                    offset += size
    result.append("")
    return "\n".join(result)

def main():
    items = make_items(ITEMS)

    for g in [
        generate_header,
        generate_constants,
        generate_structs,
        generate_impl_unsafe_i32_only,
        generate_impl_map_item,
    ]:
        print(g(items))

if __name__ == '__main__':
    import sys
    sys.exit(main())
