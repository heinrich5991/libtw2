use crate::msg;
use crate::snap_obj;
use buffer::CapacityError;
use libtw2_gamenet_common::error::Error;
use libtw2_gamenet_common::msg::MessageId;
use libtw2_gamenet_common::msg::SystemOrGame;
use libtw2_gamenet_common::traits;
use libtw2_packer::ExcessData;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Packer;
use libtw2_packer::Unpacker;
use libtw2_packer::Warning;
use warn::Warn;

pub struct Protocol(());

impl traits::ProtocolStatic for Protocol {
    type SnapObj = snap_obj::SnapObj;
    fn obj_size(type_id: u16) -> Option<u32> {
        snap_obj::obj_size(type_id)
    }
}

impl<'a> traits::Protocol<'a> for Protocol {
    type Game = msg::Game<'a>;
    type System = msg::System<'a>;
}

impl traits::SnapObj for crate::SnapObj {
    fn decode_obj<W: Warn<ExcessData>>(
        warn: &mut W,
        obj_type_id: snap_obj::TypeId,
        p: &mut IntUnpacker,
    ) -> Result<Self, Error> {
        crate::SnapObj::decode_obj(warn, obj_type_id, p)
    }
    fn obj_type_id(&self) -> snap_obj::TypeId {
        self.obj_type_id()
    }
    fn encode(&self) -> &[i32] {
        self.encode()
    }
}

impl<'a> traits::Message<'a> for msg::Game<'a> {
    fn decode_msg<W: Warn<Warning>>(
        warn: &mut W,
        id: SystemOrGame<MessageId, MessageId>,
        p: &mut Unpacker<'a>,
    ) -> Result<msg::Game<'a>, Error> {
        if let SystemOrGame::Game(id) = id {
            msg::Game::decode_msg(warn, id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    fn msg_id(&self) -> SystemOrGame<MessageId, MessageId> {
        SystemOrGame::Game(self.msg_id())
    }
    fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        self.encode_msg(p)
    }
}

impl<'a> traits::Message<'a> for msg::System<'a> {
    fn decode_msg<W: Warn<Warning>>(
        warn: &mut W,
        id: SystemOrGame<MessageId, MessageId>,
        p: &mut Unpacker<'a>,
    ) -> Result<msg::System<'a>, Error> {
        if let SystemOrGame::System(id) = id {
            msg::System::decode_msg(warn, id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    fn msg_id(&self) -> SystemOrGame<MessageId, MessageId> {
        SystemOrGame::System(self.msg_id())
    }
    fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        self.encode_msg(p)
    }
}
