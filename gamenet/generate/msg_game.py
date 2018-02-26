import network
import fix_network
import datatypes

emit = datatypes.Emit()

with emit:
    datatypes.emit_header_msg_game()
    datatypes.emit_enum_msg_module("Game", network.Messages)

emit.dump()
