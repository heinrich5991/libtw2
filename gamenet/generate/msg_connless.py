import network
import fix_network
import datatypes

datatypes.emit_header_msg_connless()

for m in network.Connless:
    m.emit_consts()
print()

datatypes.emit_enum_connless("Connless", network.Connless)

for m in network.Connless:
    m.emit_definition()
    print()

for m in network.Connless:
    m.emit_impl_encode_decode()
    m.emit_impl_debug()
    print()
