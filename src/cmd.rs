use clap::{ArgMatches, App, AppSettings, Arg};

#[derive(Debug)]
pub struct Arguments {
    pub key: String,
    pub value: String,
    pub address: String,
}

pub fn get_cli_app() -> ArgMatches<'static> {
    let matches = App::new("poncho-raft")
        .about("Raft-Based CLI Key-Value TCP Server and Key-Value Store with JSON Messaging")
        .version("1.0")
        .author("Jon V <rigzba21>")
        .subcommand(
            App::new("server")
                .about("Starts a Raft Server on Localhost")
                .arg(
                    Arg::with_name("is_leader")
                        .help("boolean whether or not this server starts off as a Raft Leader")
                )
        )
        .subcommand(
            App::new("client")
                .about("CLI for testing Leader Message Requests")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("healthcheck")
                    .about("Sends a Healthcheck Message")
                    .arg(
                    Arg::with_name("address")
                        .required(true)
                        .takes_value(true)
                        .help("Remote Server Address; Format: 127.0.0.1:8001")
                    ),
                )
                .subcommand(App::new("append")
                    .about("Sends an arbitrary APPEND_ENTRY message")
                    .arg(
                        Arg::with_name("address")
                            .required(true)
                            .takes_value(true)
                            .help("Remote Server Address; Format: 127.0.0.1:8001")
                    ),
                )
                .subcommand(App::new("request")
                    .about("Sends an arbitrary REQUEST_VOTE message")
                    .arg(
                        Arg::with_name("address")
                            .required(true)
                            .takes_value(true)
                            .help("Remote Server Address; Format: 127.0.0.1:8001")
                    ),
                )
        )

        .get_matches();
    matches
}

//get args for raft specific communications
pub fn get_address(arg_matchers: &ArgMatches) -> Arguments {
    Arguments{
        key: "".to_string(),
        value: "".to_string(),
        address: arg_matchers.value_of("address").unwrap().parse().unwrap(),
    }
}

pub fn print_address(arg_matchers: &ArgMatches) {
    let fmt_msg = format!("Remote-Address: {}", arg_matchers.value_of("address").unwrap());
    println!("{}", fmt_msg);
}

