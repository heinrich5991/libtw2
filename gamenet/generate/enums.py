import network
import fix_network
import datatypes

datatypes.emit_header_enums()

for e in network.Enums:
    e.emit_definition()
    print()

for e in network.Enums:
    e.emit_impl()
    print()
