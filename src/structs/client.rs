use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpStream};
use std::mem;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use error::AppResult;
use message::{Member, Request, RequestBody, Notify, UnicastNotify, BroadcastNotify};

pub struct Client {
    addr: SocketAddr,
    name: String,
    tx: Sender<Notify>
}

impl Client {
    fn join(&self, list: Vec<Member>) -> AppResult<()> {
        let ntf = UnicastNotify::Join { name: self.name.clone() };
        try!(self.send_message(ntf));
        self.list(list)
    }

    fn leave(&self) -> AppResult<()> {
        let rsp = UnicastNotify::Leave;
        self.send_message(rsp)
    }

    fn list(&self, list: Vec<Member>) -> AppResult<()> {
        let rsp = UnicastNotify::List(list);
        self.send_message(rsp)
    }
    
    fn message(&self, message: String) -> AppResult<()> {
        let rsp = UnicastNotify::Message(message);
        self.send_message(rsp)
    }

    fn rename_success(&mut self, name: String) -> AppResult<String> {
        try!(self.send_rename(true));
        Ok(mem::replace(&mut self.name, name))
    }

    fn rename_failure(&self) -> AppResult<()> {
        self.send_rename(false)
    }

    fn send_rename(&self, succeed: bool) -> AppResult<()> {
        let rsp = UnicastNotify::Rename(succeed);
        self.send_message(rsp)
    }

    fn register_failure(&self) -> AppResult<()> {
        self.send_register(false)
    }

    fn send_register(&self, succeed: bool) -> AppResult<()> {
        let rsp = UnicastNotify::Register(succeed);
        self.send_message(rsp)
    }

    fn submit_success(&self) -> AppResult<()> {
        self.send_submit(true)
    }

    fn send_submit(&self, succeed: bool) -> AppResult<()> {
        let rsp = UnicastNotify::Submit(succeed);
        self.send_message(rsp)
    }

    fn send_message(&self, message: UnicastNotify) -> AppResult<()> {
        try!(self.tx.send(Notify::Unicast(message)));
        Ok(())
    }
}
