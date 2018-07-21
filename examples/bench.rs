#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate stateright;

mod state_machines;

use clap::{Arg, App, AppSettings, SubCommand};
use state_machines::two_phase_commit;
use state_machines::write_once_register;
use stateright::*;
use std::collections::BTreeSet;
use std::iter::FromIterator;

fn main() {
    let args = App::new("bench")
        .about("benchmarks the stateright library")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("2pc")
            .about("two phase commit")
            .arg(Arg::with_name("rm_count")
                 .help("number of resource managers")
                 .default_value("7")))
        .subcommand(SubCommand::with_name("wor")
            .about("write-once register")
            .arg(Arg::with_name("client_count")
                 .help("number of clients proposing values")
                 .default_value("5")))
        .get_matches();
    match args.subcommand() {
        ("2pc", Some(args)) => {
            let rm_count = value_t!(args, "rm_count", u32).expect("rm_count");
            println!("Benchmarking two phase commit with {} resource managers.", rm_count);

            let mut sys = two_phase_commit::TwoPhaseSys {
                rms: BTreeSet::from_iter(0..rm_count)
            };
            sys.checker(KeepPaths::Yes, two_phase_commit::is_consistent).check_and_report();
        }
        ("wor", Some(args)) => {
            let client_count = std::cmp::min(
                26, value_t!(args, "client_count", u8).expect("client_count"));
            println!("Benchmarking a write-once register with {} clients.", client_count);

            let mut actors = vec![write_once_register::Cfg::Server];
            for i in 0..client_count {
                actors.push(write_once_register::Cfg::Client {
                    server_id: 0, desired_value: ('A' as u8 + i) as char
                });
            }

            let sys = stateright::actor::model::ActorSystem { actors, init_network: Vec::new() };
            let mut checker = sys.checker(KeepPaths::Yes, |_sys, state| {
                let values = write_once_register::response_values(&state);
                match values.as_slice() {
                    [] => true,
                    // Should have a tighter bound. Could recompute count but probably cleaner to
                    // update the checker to accept a `Fn` instead of a `fn`.
                    [v] => 'A' <= *v && *v <= 'Z',
                    _ => false
                }
            });
            checker.check_and_report();
        }
        _ => panic!("expected subcommand")
    }
}

