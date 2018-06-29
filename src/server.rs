use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::{self, Write};
use std::mem;
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use rand::{self, Rng};

use error::AppResult;
use message::{Member, Request, RequestBody, Notify, UnicastNotify, BroadcastNotify};
use client;

struct Client {
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

fn get_random_name<R: Rng> (rng: &mut R) -> String {
    const CHARSET: &'static [u8] = b"0123456789";

    let mut name = String::from("Guest");

    for _ in 0..3 {
        name.push(*rng.choose(CHARSET).unwrap() as char);
    }

    name
}

pub fn run(addr: &str, port: u16) -> AppResult<()> {
    let listener = try!(TcpListener::bind((addr, port)));
    let local_addr = try!(listener.local_addr());

    println!("Listening {}", local_addr);
    let (req_tx, req_rx) = mpsc::channel();

    thread::spawn(move || server_loop(req_rx));

    for stream in listener.incoming() {
        match stream {
            Err(err) => {
                writeln!(io::stderr(), "{}", err);
                continue
            }
            Ok(stream) => {
                let cli_addr = try!(stream.peer_addr());
                println!("Connected from {}", cli_addr);

                let req_tx = req_tx.clone();
                thread::spawn(move || {
                    if let Err(err) = client::run(stream, req_tx) {
                        writeln!(io::stderr(), "Client {} aborted: {}", cli_addr, err);
                    }
                });
            }
        }
    }

    Ok(())
}

fn server_loop(req_rx: Receiver<Request>) {
    let mut rng = rand::thread_rng();
    let mut map = HashMap::new();
    let mut names = HashMap::new();

    loop {
        let mut broadcast = None;

        match req_rx.recv() {
            Err(err) => {
                writeln!(io::stderr(), "Request recv error: {}", err);
            }
            Ok(req) => {
                let addr = req.addr;
                if let Err(err) = handle_request(req, &mut map, &mut names, &mut broadcast, &mut rng) {
                    writeln!(io::stderr(), "Response send Rrror: {}", err);
                    if let Some(cli) = map.remove(&addr) {
                        cli.leave();
                    }
                }
            }
        };

        if let Some(ntf) = broadcast {
            for cli in map.values() {
                cli.tx.send(Notify::Broadcast(ntf.clone()));
            }
        }
    }
}

fn handle_request<R: Rng>(
    Request {addr, body}: Request,
    map: &mut HashMap<SocketAddr, Client>,
    names: &mut HashMap<String, String>,
    broadcast: &mut Option<BroadcastNotify>,
    rng: &mut R)
    -> AppResult<()>
{
    match body {
        RequestBody::Join { tx } => {
            let mut name;
            loop {
                name = get_random_name(rng);
                if !names.contains_key(&name) {
                    break;
                }
            }

            let cli = Client { name: name.clone(), addr: addr, tx: tx };
            let list = map.values().map(|cli| {
                Member {
                    name: cli.name.clone(),
                    addr: cli.addr
                }
            }).collect();
            
            try!(cli.join(list));

            names.insert(cli.name.clone(), "".to_string());
            let _ = map.insert(addr, cli);

            
            *broadcast = Some(BroadcastNotify::Join { name: name, addr: addr });
        }
        RequestBody::Leave => {
            if let Some(cli) = map.remove(&addr) {
                let name = cli.name.clone();
                let addr = cli.addr;
                if let Err(err) = cli.leave() {
                    let _ = writeln!(io::stderr(), "Response send Rrror: {}", err);
                
                }

                *broadcast = Some(BroadcastNotify::Leave { name: name, addr: addr });
            }
        }
        RequestBody::List => {
            if let Some(cli) = map.get(&addr) {
                let list = map.values().map(|cli| {
                    Member {
                        name: cli.name.clone(),
                        addr: cli.addr
                    }
                }).collect();
                try!(cli.list(list));
            }
        }
        RequestBody::Rename { name, password } => {
            for val in map.values() {
                if &val.name == &name {
                    try!(val.rename_failure());
                    ()
                }
            }

            if let Some(cli) = map.get_mut(&addr) {
                match names.entry(name.clone()) {
                    Occupied(o) => {
                        let current_password = o.get();
                        if current_password.is_empty() {
                            let old_name = try!(cli.rename_success(name.clone()));
                            *broadcast = Some(BroadcastNotify::Rename {
                                old_name: old_name,
                                new_name: name,
                                addr: cli.addr
                            });
                        } else {
                            if current_password == &password {
                                let old_name = try!(cli.rename_success(name.clone()));
                                *broadcast = Some(BroadcastNotify::Rename {
                                    old_name: old_name,
                                    new_name: name,
                                    addr: cli.addr
                                });
                            } else {
                                try!(cli.rename_failure());
                            }
                        }
                    }
                    Vacant(v) => {
                        let old_name = try!(cli.rename_success(name.clone()));
                        v.insert(password);
                        *broadcast = Some(BroadcastNotify::Rename {
                            old_name: old_name,
                            new_name: name,
                            addr: cli.addr
                        });
                    }
                }
            }
        }
        RequestBody::Register { name, password } => {
            if let Some(cli) = map.get(&addr) {
                let can_set = !names.contains_key(&name);
                // let can_set = names[&name].is_empty();
                if can_set {
                    names.insert(name.clone(), password.clone());   
                    *broadcast = Some(BroadcastNotify::Register {
                        name: name.clone(),
                        addr: cli.addr
                    });
                } else {
                    try!(cli.register_failure());
                }
            }
        }
        RequestBody::Submit { message } => {
            if let Some(cli) = map.get(&addr) {
                try!(cli.submit_success());

                *broadcast = Some(BroadcastNotify::Submit {
                    name: cli.name.clone(),
                    addr: cli.addr,
                    message: message
                });
            }
        }
        RequestBody::UnicastMessage { message } => {
            if let Some(cli) = map.get(&addr) {
                try!(cli.message(message));
            }
        }
    }
    Ok(())
}