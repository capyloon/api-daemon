use crossbeam_channel as crossbeam;
use http::request::Request;
use std::rc::Rc;

pub struct Receiver {
    pub rx: Rc<crossbeam::Receiver<Request<Vec<u8>>>>,
}

impl Receiver {
    pub fn is_empty(&self) -> bool {
        self.rx.len() == 0
    }

    pub fn len(&self) -> usize {
        self.rx.len()
    }

    pub fn next(&self) -> Option<Request<Vec<u8>>> {
        self.rx.try_recv().ok()
    }
}

pub(crate) struct Sender {
    pub tx: crossbeam::Sender<Request<Vec<u8>>>,
}

impl Sender {
    pub fn new(tx: crossbeam::Sender<Request<Vec<u8>>>) -> Self {
        Self { tx }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crossbeam_channel as channel;

    #[test]
    fn request_receiver_is_empty() {
        let (tx, rx) = channel::unbounded();
        let rr = Receiver { rx: Rc::new(rx) };

        assert!(rr.is_empty());

        add_request(tx);

        assert!(!rr.is_empty());
    }

    #[test]
    fn request_reciever_len() {
        let (tx, rx) = channel::unbounded();
        let rr = Receiver { rx: Rc::new(rx) };

        assert_eq!(rr.len(), 0);

        add_request(tx);

        assert_eq!(rr.len(), 1);
    }

    #[test]
    fn request_reciever_next() {
        let (tx, rx) = channel::unbounded();
        let rr = Receiver { rx: Rc::new(rx) };

        assert!(rr.next().is_none());

        add_request(tx);

        assert!(rr.next().is_some());
    }

    fn add_request(tx: channel::Sender<Request<Vec<u8>>>) {
        let _ = tx.send(Request::default());
    }
}
