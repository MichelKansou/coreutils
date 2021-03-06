use std::{
    os::{raw::c_int, unix::process::CommandExt},
    process::{self, Command},
};

use coreutils_core::{
    libc::ENOENT,
    os::{
        group::Error as GrError,
        process::{change_root, set_group, set_groups, set_user},
    },
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("chroot.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    // Ok to unwrap cause it's required argument
    let root = matches.value_of("NEWROOT").unwrap();
    let cmd = matches.value_of("COMMAND").unwrap_or("/bin/sh");
    let args: Vec<&str> = match matches.values_of("ARGS") {
        Some(args) => args.collect(),
        None => vec!["-i"],
    };

    if let Err(err) = change_root(root) {
        eprintln!("chroot: Cannot change root directory to {}: {}", root, err);
        process::exit(125);
    }

    if let Some(groups_list) = matches.value_of("groups") {
        if let Err(err) = set_groups_from_list(groups_list) {
            eprintln!("chroot: Unable to set a group from supplementary list: {}", err);
            process::exit(1);
        }
    }

    if let Some(userspec_str) = matches.value_of("userspec") {
        if let Err(err) = set_user_from_userspec(userspec_str) {
            eprintln!("chroot: Unable to set user and/or group from userspec: {}", err);
            process::exit(1);
        }
    }

    if let Some(group) = matches.value_of("group") {
        if let Err(err) = set_group(group) {
            eprintln!("chroot: Unable to set group for process: {}", err);
            process::exit(1);
        }
    }

    if let Some(user) = matches.value_of("user") {
        if let Err(err) = set_user(user) {
            eprintln!("chroot: Unable to set user for process: {}", err);
            process::exit(1);
        }
    }

    let err = Command::new(cmd).args(args).exec();

    if err.raw_os_error().unwrap() as c_int == ENOENT {
        eprintln!("chroot: '{}': {}", cmd, err);
        process::exit(127);
    } else {
        eprintln!("chroot: {}", err);
        process::exit(126);
    }
}

fn set_groups_from_list(groups_list: &str) -> Result<(), GrError> {
    let groups: Vec<&str> = groups_list.split(',').collect();
    Ok(set_groups(&groups)?)
}

fn set_user_from_userspec(userspec: &str) -> Result<(), GrError> {
    let parts: Vec<&str> = userspec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("chroot: Userspec is in an incorrect format");
        std::process::exit(1);
    }

    let (user, group) = (parts[0], parts[1]);

    set_group(group)?;
    set_user(user)?;

    Ok(())
}
