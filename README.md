# telnet_chat
## Usage
start the server
```bash
$ cd to/project/folder/telnet_chat && cargo run
```
connect to server
```bash
$ telnet localhost 10001
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
Welcome to telnet chat!
Hello! Your name is Guest287! Enjoy!
Members online:
Join: Guest287 from 127.0.0.1:56676
/help
Commands:
    /help                       Show this message.
    /exit                       Exit from chat.
    /list                       Show participants list.
    /rename NAME [PASSWORD]     Change your name.
    /register NAME PASSWORD     Protect your name
```
