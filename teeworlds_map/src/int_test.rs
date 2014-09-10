#![feature(macro_rules)]

macro_rules! map_item(
	($name:ident, $($members:ident)*) => (
		#[deriving(Clone, Show)]
		#[repr(packed)]
		pub struct $name {
			$(pub $members: i32,)*
		}
	);
)

map_item!(MapItemVersionV1, version foo)

