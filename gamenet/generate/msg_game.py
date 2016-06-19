import network
import fix_network
import datatypes

datatypes.emit_header_msg_game()

for m in network.Messages:
    m.emit_consts()
print()

datatypes.emit_enum("Game", network.Messages)

for m in network.Messages:
    m.emit_definition()
    print()

for m in network.Messages:
    m.emit_impl_encode_decode()
    m.emit_impl_debug()
    print()
