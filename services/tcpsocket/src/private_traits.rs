use crate::generated::common::TcpSocketMethods;
use crate::tcpsocket::EventType;
use mio::Poll;
use std::io::Result;

pub trait PrivateTrait: TcpSocketMethods {
    fn register(&self, poll: &Poll);
    fn close_internal(&mut self);
    fn post_close(&mut self, poll: &Poll);
    fn is_ready(&self) -> bool;
    fn set_ready(&self, status: bool) -> bool;
    fn on_event(&self, etype: EventType, data: Vec<u8>);
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn drain_queue(&mut self);
    fn send_queue(&mut self, request_id: u64, data: Vec<u8>);
}
