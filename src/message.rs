use std::net::SocketAddr;
use std::sync::mpsc::Sender;

type Address = SocketAddr;

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub addr: Address,
}

pub struct Request {
    pub addr: Address,
    pub body: RequestBody
}

pub enum RequestBody {
    Join { tx: Sender<Notify> },
    Leave,
    List,
    Rename { name: String, password: String },
    Register { name: String, password: String },
    Submit { message: String },
    UnicastMessage { message: String }
}

#[derive(Debug, Clone)]
pub enum Notify {
    Unicast(UnicastNotify),
    Broadcast(BroadcastNotify)
}

#[derive(Debug, Clone)]
pub enum UnicastNotify {
    Join { name: String },
    Leave,
    List(Vec<(Member)>),
    Rename(bool),
    Register(bool),
    Submit(bool),
    Message(String)
}

#[derive(Debug, Clone)]
pub enum BroadcastNotify {
    Join { name: String, addr: Address },
    Leave { name: String, addr: Address },
    Rename { old_name: String, new_name: String, addr: Address },
    Register { name: String, addr: Address },
    Submit { name: String, addr: Address, message: String },
}
