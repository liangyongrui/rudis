use common::{
    connection::parse::{frame::Frame, Parse},
    pd_message::{ServerInit, ServerStatus},
};

use crate::status;

pub fn server_init_apply<'a>(parse: &'a mut Parse<'a>) -> common::Result<Frame<'a>> {
    let payload: ServerInit = bincode::deserialize(&*parse.next_bulk()?)?;
    Ok(Frame::OwnedBulk(bincode::serialize(&status::server_init(
        &payload,
    )?)?))
}

pub fn server_heartbeat_apply<'a>(parse: &mut Parse<'a>) -> common::Result<Frame<'a>> {
    let payload: ServerStatus = bincode::deserialize(&*parse.next_bulk()?)?;
    Ok(Frame::OwnedBulk(bincode::serialize(
        &status::server_heartbeat(&payload),
    )?))
}

pub fn crate_group_apply() -> Frame<'static> {
    status::create_group();
    Frame::ok()
}
