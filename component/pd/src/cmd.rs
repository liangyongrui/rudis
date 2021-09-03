use common::pd_message::{ServerInit, ServerStatus};
use connection::parse::{frame::Frame, Parse};

use crate::status;

pub fn server_init_apply(parse: &mut Parse) -> common::Result<Frame> {
    let payload: ServerInit = bincode::deserialize(&*parse.next_bulk()?)?;
    Ok(Frame::Bulk(
        bincode::serialize(&status::server_init(&payload)?)?.into(),
    ))
}

pub fn server_heartbeat_apply(parse: &mut Parse) -> common::Result<Frame> {
    let payload: ServerStatus = bincode::deserialize(&*parse.next_bulk()?)?;
    Ok(Frame::Bulk(
        bincode::serialize(&status::server_heartbeat(&payload))?.into(),
    ))
}

pub fn crate_group_apply() -> Frame {
    status::create_group();
    Frame::ok()
}
