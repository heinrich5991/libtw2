#![crate_name="map_macros"]
#![crate_type="dylib"]

#![feature(plugin_registrar)]
#![feature(quote)]

extern crate syntax;
extern crate rustc;

use rustc::plugin::Registry;
use syntax::ast::AttrOuter;
use syntax::ast::Attribute;
use syntax::ast::Attribute_;
use syntax::ast::DUMMY_NODE_ID;
use syntax::ast::Ident;
use syntax::ast::Item;
use syntax::ast::ItemStruct;
use syntax::ast::Item_;
use syntax::ast::MetaItem;
use syntax::ast::MetaList;
use syntax::ast::MetaWord;
use syntax::ast::NamedField;
use syntax::ast::Public;
use syntax::ast::StructDef;
use syntax::ast::StructField_;
use syntax::ast::TokenTree;
use syntax::ast_util;
use syntax::attr;
use syntax::codemap::Span;
use syntax::codemap::Spanned;
use syntax::ext::base::ExtCtxt;
use syntax::ext::base::MacItems;
use syntax::ext::base::MacResult;
use syntax::ext::build::AstBuilder;
use syntax::parse::token::COLON;
use syntax::parse::token::COMMA;
use syntax::parse::token::IDENT;
use syntax::parse::token::InternedString;
use syntax::parse::token::LBRACKET;
use syntax::parse::token::LIT_INTEGER;
use syntax::parse::token::RBRACKET;
use syntax::parse::token;
use syntax::ptr::P;

use std::from_str::FromStr;
use std::iter::Repeat;

// Parsing this:
// map_item!(<type_id>, <name>, (<version>: (<field>([<count>]))*)+)
fn expand_map_item(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult+'static> {
    let mut p = cx.new_parser_from_tts(tts);
    let mut iter = Repeat::new(()).map(|()| p.bump_and_get()).take_while(|x| *x != token::EOF).peekable();

    // <type_id>
    let type_id: u16 = match iter.next() {
        Some(LIT_INTEGER(name)) => FromStr::from_str(token::get_name(name).get()).expect("Expected integer type id."),
        Some(_) | None => fail!("Expected integer type id."),
    };

    // ,
    match iter.next() {
        Some(COMMA) => {},
        Some(_) | None => fail!("Expected comma."),
    }

    // <name>
    let name = match iter.next() {
        Some(IDENT(ident, _)) => token::str_to_ident(format!("MapItem{}", token::get_ident(ident).get()).as_slice()),
        Some(_) | None => fail!("Expected identifier for name."),
    };

    // ,
    match iter.next() {
        Some(COMMA) => {},
        Some(_) | None => fail!("Expected comma."),
    }

    enum Member {
        Simple(Ident),
        Array(Ident, uint),
    }

    impl Member {
        fn size(&self) -> uint {
            match self {
                &Simple(_) => 1,
                &Array(_, size) => size,
            }
        }
    }

    let mut versions: Vec<Vec<Member>> = vec![];
    let mut version = 0i32;

    versions.push(vec![Simple(token::str_to_ident("version"))]);

    loop {
        let mut members = vec![];

        version += 1;
        // <version>
        if version != match iter.next() {
            Some(LIT_INTEGER(name)) => FromStr::from_str(token::get_name(name).get()).expect("Expected integer version."),
            Some(_) => fail!("Expected integer version."),
            None => if version != 1 { break } else { fail!("Expected version 1."); },
        } {
            fail!("Expected version {}.", version);
        }

        // :
        match iter.next() {
            Some(COLON) => {},
            Some(_) | None => fail!("Expected colon."),
        }

        loop {
            // <field>
            let field_ident = match iter.next() {
                Some(IDENT(ident, _)) => ident,
                Some(COMMA) | None => { versions.push(members); break; }
                Some(x) => fail!("Unexpected token {}", x),
            };

            match iter.peek() {
                Some(&LBRACKET) => {
                    // [
                    iter.next();

                    // <count>
                    let count: uint = match iter.next() {
                        Some(LIT_INTEGER(name)) => FromStr::from_str(token::get_name(name).get()).expect("Expected integer array size."),
                        Some(_) | None => fail!("Expected integer array size."),
                    };

                    // ]
                    match iter.next() {
                        Some(RBRACKET) => {},
                        Some(_) | None => fail!("Expected closing bracket."),
                    }

                    members.push(Array(field_ident, count));
                }
                _ => members.push(Simple(field_ident)),
            }
        }
    }

    let mut items: Vec<P<Item>> = vec![];
    let mut total_size: uint = 0;

    for (i, v) in versions.iter().enumerate() {
        let struct_size: uint = v.iter().fold(0, |size, member| size + member.size());
        let offset: uint = total_size;
        let version: i32 = i.to_i32().expect("Version must fit into an i32.");
        total_size += struct_size;

        let struct_name: Ident = token::str_to_ident(format!("{}V{}", token::get_ident(name).get(), i).as_slice());
        let mut struct_def = StructDef {
            fields: Vec::with_capacity(v.len()),
            ctor_id: None,
            super_struct: None,
            is_virtual: false,
        };

        for m in v.iter() {
            match m {
                &Simple(ident) => {
                    struct_def.fields.push(Spanned {
                        node: StructField_ {
                            kind: NamedField(ident, Public),
                            id: DUMMY_NODE_ID,
                            ty: quote_ty!(cx, i32),
                            attrs: vec![],
                        },
                        span: sp,
                    });
                },
                &Array(ident, count) => {
                    struct_def.fields.push(Spanned {
                        node: StructField_ {
                            kind: NamedField(ident, Public),
                            id: DUMMY_NODE_ID,
                            ty: quote_ty!(cx, [i32, ..$count]),
                            attrs: vec![],
                        },
                        span: sp,
                    });
                },
            }
        }

        let attrs: Vec<Attribute> = {
            let intern_repr      = InternedString::new("repr");
            let intern_c         = InternedString::new("C");
            let intern_packed    = InternedString::new("packed");
            //let intern_deriving  = InternedString::new("deriving");
            //let intern_allow     = InternedString::new("allow");
            //let intern_deadcode  = InternedString::new("dead_code");
            
            vec![
                // #[repr(C, packed)]
                Attribute { span: sp, node: Attribute_ {
                    id: attr::mk_attr_id(),
                    style: AttrOuter,
                    value: P(MetaItem {
                        span: sp,
                        node: MetaList(intern_repr, vec![
                            P(MetaItem { span: sp, node: MetaWord(intern_c) }),
                            P(MetaItem { span: sp, node: MetaWord(intern_packed) }),
                        ]),
                    }),
                    is_sugared_doc: false,
                }},
                // #[deriving()]
                /*Attribute { span: sp, node: Attribute_ {
                    id: attr::mk_attr_id(),
                    style: AttrOuter,
                    value: P(MetaItem {
                        span: sp,
                        node: MetaList(intern_deriving, vec![
                        ]),
                    }),
                    is_sugared_doc: false,
                }},*/
                // #[allow(dead_code)]
                /*Attribute { span: sp, node: Attribute_ {
                    id: attr::mk_attr_id(),
                    style: AttrOuter,
                    value: P(MetaItem {
                        span: sp,
                        node: MetaList(intern_allow, vec![
                            P(MetaItem { span: sp, node: MetaWord(intern_deadcode) }),
                        ]),
                    }),
                    is_sugared_doc: false,
                }},*/
            ]
        };

        let item_struct: Item_ = ItemStruct(P(struct_def), ast_util::empty_generics());
        let item_struct: P<Item> = cx.item(sp, struct_name, attrs, item_struct);
        let item_struct: P<Item> = item_struct.map(|mut x| { x.vis = Public; x });
        items.push(item_struct);

        items.push(quote_item!(cx,
            impl ::std::clone::Clone for $struct_name {
                fn clone(&self) -> $struct_name {
                    *self
                }
            }
        ).unwrap());

        items.push(quote_item!(cx,
            impl $struct_name {
                #[allow(dead_code)]
                pub fn from_slice(slice: &[i32]) -> Option<&$struct_name> {
                    if slice.len() < $total_size {
                        return None;
                    }
                    if slice[0] < $version {
                        return None;
                    }
                    let result: &[i32] = slice.slice($offset, $total_size);
                    assert!(result.len() * ::std::mem::size_of::<i32>() == ::std::mem::size_of::<$struct_name>());
                    Some(unsafe { &*(result.as_ptr() as *const $struct_name) })
                }
                #[allow(dead_code)]
                pub fn from_slice_mut(slice: &mut [i32]) -> Option<&mut $struct_name> {
                    if slice.len() < $total_size {
                        return None;
                    }
                    if slice[0] < $version {
                        return None;
                    }
                    let result: &mut [i32] = slice.slice_mut($offset, $total_size);
                    assert!(result.len() * ::std::mem::size_of::<i32>() == ::std::mem::size_of::<$struct_name>());
                    Some(unsafe { &mut *(result.as_mut_ptr() as *mut $struct_name) })
                }
            }
        ).unwrap());
    }

    MacItems::new(items.into_iter())
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("map_item", expand_map_item);
}
