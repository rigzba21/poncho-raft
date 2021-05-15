# `poncho-raft`
`poncho-raft` is a CLI utility TCP server and client for sending and receiving JSON serialized messages with a Key/Value
Store that implements the Raft algorithm.

This project is the continuation of the one I started during the [Rafting Trip Course](https://dabeaz.com/raft.html).

#### Prereqs
- [Rust](https://www.rust-lang.org/tools/install)

## Quickstart

In one terminal, run:
```bash
cargo build
./target/debug/poncho-raft server is_leader
```
This runs the TCP server, and listens for incoming messages.

Make note of the terminal output for the _listener address_ and _port_:
example:
```shell
TCP Listener on address: 127.0.0.1, port: 8002
```

Then in another terminal, you can use a `client` CLI to issue `append`, `healthcheck`, and `request` messages to the running server:
```bash
#healthcheck message for testing
./target/debug/poncho-raft client healthcheck 127.0.0.1:8002

#send an AppendEntry message
./target/debug/poncho-raft client append 127.0.0.1:8002

#send a RequestVote message
./target/debug/poncho-raft client request 127.0.0.1:8002
```

Now monitor the output in both the `server` and `client` CLI:
example `server` output for receiving an `append` message:
```shell
Message Queue is Empty...Continuing
New connection: 127.0.0.1:33458
"{\"src_id\":12231374011174487598,\"src_addr\":\"127.0.0.1:33458\",\"msg_type\":\"APPEND_ENTRY\",\"payload\":{\"log_entry\":{\"leader_term\":1,\"leader_id\":\"1234\",\"prev_index\":0,\"prev_term\":0,\"leader_commit_index\":1},\"entries\":{\"log_entries\":[{\"leader_term\":1,\"leader_id\":\"1234\",\"prev_index\":0,\"prev_term\":0,\"leader_commit_index\":1}]}}}"
Response Message: "{\"msg_type\":\"HEALTHCHECK\",\"payload\":\"ok\"}"
doing leader stuff
Message Popped off Queue, new Size: 0
Log Appended: LogEntry {
    leader_term: 1,
    leader_id: "1234",
    prev_index: 0,
    prev_term: 0,
    leader_commit_index: 1,
}
```

example `client` CLI output with response from server:
```shell
Remote-Address: 127.0.0.1:8001
"{\"msg_type\":\"HEALTHCHECK\",\"payload\":\"ok\"}"
```


#### Example Help Output
```bash
cargo build

./target/debug/poncho-raft --help
```
outputs:
```bash
raftkv 1.0
Jon Velando <rigzba21>
Raft-Based CLI Key-Value TCP Server and Key-Value Store with JSON Messaging

USAGE:
    raftkv [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    client    Testing Client to Interact with Raft Servers
    help      Prints this message or the help of the given subcommand(s)
    server    Starts a Raft Server on Localhost
```

#### Example GET/SET/DELETE
In another terminal, run:
```bash
#HEALTHCHECK
./target/debug/poncho-raft client healthcheck 127.0.0.1:8002
```

## Running Unit Tests
```shell
cargo test -- --nocapture
```

## Raft Network/Cluster Setup

To setup a cluster of 5x nodes, run: `make all`. This will create directories for each node. Then run the following:
```bash
# start node1 as the initial leader
./node1/poncho-raft server is_leader

# start the rest of the nodes as followers

# in a separate terminal window:
./node2/poncho-raft server

# in a separate terminal window:
./node3/poncho-raft server

# in a separate terminal window:
./node4/poncho-raft server

# in a separate terminal window:
./node5/poncho-raft server
```

#### Troubleshooting:
Running all of the nodes in a single `tmux` session was not working for me, but separate individual terminal windows did.

#### Cleanup

Run `make clean`.
