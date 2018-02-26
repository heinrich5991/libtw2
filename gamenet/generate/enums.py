import network
import fix_network
import datatypes

emit = datatypes.Emit()

with emit:
    datatypes.emit_header_enums()
    datatypes.emit_enum_module(network.Enums)

emit.dump()
