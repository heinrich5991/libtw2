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
        ["channels", "start_points", "num_points", "name[8s]"],
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

header = """\
extern crate datafile;

use datafile::OnlyI32;

use std::mem;

pub trait MapItem: OnlyI32 {
    fn version() -> i32;
    fn offset() -> usize;
}

pub trait MapItemExt: MapItem {
    fn len() -> usize {
        mem::size_of::<Self>() / mem::size_of::<i32>()
    }
    fn sum_len() -> usize {
        Self::offset() + Self::len()
    }
    fn from_slice(slice: &[i32]) -> Option<&Self> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if slice[0] < Self::version() {
            return None;
        }
        let result: &[i32] = &slice[Self::offset()..Self::sum_len()];
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some(unsafe { &*(result.as_ptr() as *const Self) })
    }
    fn from_slice_mut(slice: &mut [i32]) -> Option<&mut Self> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if slice[0] < Self::version() {
            return None;
        }
        let result: &mut [i32] = &mut slice[Self::offset()..Self::sum_len()];
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some(unsafe { &mut *(result.as_ptr() as *mut Self) })
    }
}

impl<T:MapItem> MapItemExt for T { }

pub fn i32s_to_bytes(result: &mut [u8], input: &[i32]) {
    assert!(result.len() == input.len() * mem::size_of::<i32>());
    for (output, input) in result.chunks_mut(mem::size_of::<i32>()).zip(input) {
        output[0] = (((input >> 24) & 0xff) - 0x80) as u8;
        output[1] = (((input >> 16) & 0xff) - 0x80) as u8;
        output[2] = (((input >>  8) & 0xff) - 0x80) as u8;
        output[3] = (((input >>  0) & 0xff) - 0x80) as u8;
    }
}

pub fn bytes_to_string(bytes: &[u8]) -> &[u8] {
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0 {
            return &bytes[..i]
        }
    }
    bytes
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MapItemCommonV0 {
    pub version: i32,
}

unsafe impl OnlyI32 for MapItemCommonV0 { }
impl MapItem for MapItemCommonV0 { fn version() -> i32 { 0 } fn offset() -> usize { 0 } }
"""

def make_items(items):
    MEMBER_NORMAL=re.compile(r'^(?P<name>[a-z_]+)$')
    MEMBER_ARRAY=re.compile(r'^(?P<name>[a-z_]+)\[(?P<size>[1-9][0-9]*)\]$')
    MEMBER_STRING=re.compile(r'^(?P<name>[a-z_]+)\[(?P<size>[1-9][0-9]*)s\]$')

    result = []
    for (type_id, name, versions) in items:
        result_versions = []
        for version in versions:
            result_version = []
            for member in version:
                m = MEMBER_NORMAL.match(member)
                if m is not None:
                    result_version.append((m.group('name'), None, None))
                else:
                    m = MEMBER_ARRAY.match(member)
                    if m is not None:
                        result_version.append((m.group('name'), int(m.group('size')), None))
                    else:
                        m = MEMBER_STRING.match(member)
                        if m is not None:
                            result_version.append((m.group('name'), int(m.group('size')), 's'))
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
            result.append("pub const MAP_ITEMTYPE_{}: u16 = {};".format(name.upper(), type_id))
    result.append("")
    return "\n".join(result)

def generate_structs(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("#[derive(Clone, Copy, Debug)]")
            result.append("#[repr(C)]")
            if version:
                result.append("pub struct {s} {{".format(s=struct_name(name, i)))
                for (member, size, _) in version:
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
            result.append("unsafe impl OnlyI32 for {s} {{ }}".format(s=struct_name(name, i)))
    result.append("")
    return "\n".join(result)

def generate_impl_map_item(items):
    result = []
    for (_, name, versions) in items:
        offset = 1
        for (i, version) in enumerate(versions):
            result.append("impl MapItem for {s} {{ fn version() -> i32 {{ {v} }} fn offset() -> usize {{ {o} }} }}".format(s=struct_name(name, i), v=i+1, o=offset))
            for (_, size, _) in version:
                if size is None:
                    offset += 1
                else:
                    offset += size
    result.append("")
    return "\n".join(result)

def generate_impl_string(items):
    result = []
    for (_, name, versions) in items:
        offset = 1
        for (i, version) in enumerate(versions):
            for (member, size, type) in version:
                if size is None or type is None:
                    continue
                if type != 's':
                    raise ValueError("Invalid type: {t}".format(type))
                result.append("""\
impl {s} {{
    pub fn {m}_get(&self) -> [u8; {num_bytes}] {{
        let mut result: [u8; {num_bytes}] = unsafe {{ mem::uninitialized() }};
        i32s_to_bytes(&mut result, &self.{m});
        result[{num_bytes}-1] = 0;
        result
    }}
}}""".format(s=struct_name(name, i), m=member, num_bytes=size*4))

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
        generate_impl_string,
    ]:
        print(g(items))

if __name__ == '__main__':
    import sys
    sys.exit(main())
