import network
import fix_network
import datatypes

emit = datatypes.Emit()

with emit:
    datatypes.emit_header_msg_connless()
    datatypes.emit_enum_connless_module("Connless", network.Connless)

emit.dump()
