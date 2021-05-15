use crate::message::{get_dummy_append_entry_req, get_dummy_request_vote};
use crate::leader::{append_entry_request, send_healthcheck_message};
use crate::server::{setup_tcp_listener, request_vote};

//local modules
mod kv_store;
mod connection_handler;
mod cmd;
mod log;
mod message;
mod leader;
mod server;

fn main() {

    //create the CLI app
    let matches = cmd::get_cli_app();

    //Run TCP Server Listener
    if let ("server", Some(_server_matches)) = matches.subcommand() {
        let is_leader = _server_matches.is_present("is_leader");
        setup_tcp_listener(is_leader);
    }

    //client CLI wrapper for easy testing of message sending
    if let ("client", Some(client_matches)) = matches.subcommand() {

        //HEALTHCHECK Message
        if let ("healthcheck", Some(healthcheck_matches)) = client_matches.subcommand() {
            cmd::print_address(healthcheck_matches);
            let args = cmd::get_address(healthcheck_matches);
            send_healthcheck_message(args.address);
        }

        //APPEND_ENTRY message
        if let ("append", Some(append_matches)) = client_matches.subcommand() {
            cmd::print_address(append_matches);
            let args = cmd::get_address(append_matches);
            append_entry_request(args.address, get_dummy_append_entry_req());
        }

        //REQUEST_VOTE message
        if let ("request", Some(request_matches)) = client_matches.subcommand() {
            cmd::print_address(request_matches);
            let args = cmd::get_address(request_matches);
            request_vote(args.address, get_dummy_request_vote());
        }
    }
}