import network
import fix_network
import datatypes

emit = datatypes.Emit()
with emit:
    datatypes.emit_header_snap_obj()
    datatypes.emit_enum_obj_module("SnapObj", network.Objects, network.Flags)
emit.dump()
