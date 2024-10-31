use std::{collections::VecDeque, env, ffi::{c_int, c_long, CStr}, io::{self, stdin, Read, Write}, mem::take, os::{fd::AsRawFd, unix::process::CommandExt}, process::{exit, Child, Command, Stdio}, sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::{self, Duration}};

use crate::lib::TextStyler;
use termios::*;

//////////////////////////
const ESCAPE_CHAR: u8 = '/' as u8;
const TAB_CHAR: u8 = '\t' as u8;
const NEW_LINE: u8 = '\n' as u8;
const CARRIAGE_RETURN: u8 = '\r' as u8;
const BACKSPACE: u8 = '\u{7f}' as u8;
const INTERRUPT_EXIT: u8 = '\u{3}' as u8;
const SIGKILL: i32 = 9;
type Sighandler = extern "C" fn(c_int);
extern "C" {
    fn signal (sig: c_int, handler: Sighandler) -> Sighandler;
}
extern "C" fn signal_handler(sig: c_int) {
    unsafe {
        signal(sig, signal_handler);
    }
}
/*
int tcgetattr(int fd, struct termios *termios_p);
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p);
*/
fn raw_mode(fd: i32) ->(Termios,Termios) {

    let mut termios: Termios; 
    unsafe {
        termios = std::mem::zeroed();
    }
    tcgetattr(fd,&mut termios).unwrap();
    let original_termios = termios.clone();
    // Disable ICANON and ECHO
    termios.c_lflag &= !(ICANON | ECHO);
    return (termios,original_termios);

}

fn unraw_mode(fd:i32, original_termios: &mut Termios) ->() {
    tcsetattr(fd, TCSANOW, &original_termios).unwrap();
}

pub fn command_input()  {

    let fd = io::stdin().as_raw_fd();
    let (mut termios, mut original_termios) = raw_mode(fd);
    //return;
    unsafe {
        signal(2, signal_handler);
    }

    /// declaration of required variables
    let mut buf = String::new();
    let mut c: [u8; 1] = [0];
    let mut stdout = io::stdout();
    let mut child: Option<Child> = None;
    

    /// loop to take input
    loop {

        /// taking character by character input
        stdin().read_exact(&mut c).expect("unable to read".red_front().as_str());

        /// processing character by character
        match c {
            /// handelling ctrl+c
            [INTERRUPT_EXIT] => {
                buf.clear();
                write!(stdout,"\n");
                stdout.flush();
                print_shell_description(get_username(), get_current_location());
            },
            /// handelling backspace
            [BACKSPACE] => {
                if buf.len() == 0 {
                    continue;
                } else {
                    buf.pop();
                    write!(stdout,"{}", ('\u{8}' as char));
                    write!(stdout," ");
                    write!(stdout,"{}", ('\u{8}' as char));
                    stdout.flush();
                }
            },
            /// when pressed enter check for child process
            [CARRIAGE_RETURN]|[NEW_LINE] => {
                write!(stdout,"\n");
                stdout.flush();
                if(buf.trim() == String::from("exit")){
                    break;
                }
                unraw_mode(fd, &mut original_termios);
                run_command(separate_pipes(buf));
                raw_mode(fd);
                print_shell_description(get_username(), get_current_location());
                buf = String::new();
            },
            /// TODO tab for autocomplete and suggest
            [TAB_CHAR] => {},
            /// pushing other characters into buffer string
            [c] => {
                write!(stdout,"{}",c as char);
                stdout.flush();
                buf.push(c as char);
            },
        }
    }
    unraw_mode(fd,&mut original_termios);


}



fn kill_child( mutex_child: &mut Option<Child>) {

    match mutex_child {
        Some(child_process) => {
            child_process.kill();
            return;
        }
        None => {
            return;
        }
    }
}


fn get_username() -> String { //everytime calculating username since program might change username
    match Command::new("whoami").output() { 
        Ok(output) => {
            match String::from_utf8(output.stdout) {
                Ok(username) => String::from(username.trim()),
                Err(_) => String::from("username"),
            }
        }
        Err(_) => String::from("username"),
    } 
}

fn get_current_location() -> Vec<String> { //everytime calculating current location since program might change current working directory
    match env::current_dir() { 
        Ok(output) => {
            match output.to_str() {
                Some(current_location) => {
                    return String::from (current_location.trim()).split("/").map(|x| String::from(x)).collect();
                },
                None => {
                    return vec![String::from("unknown")];
                }
            }
        }
        Err(_) => {return vec![String::from("unknown")];},
    }
}

fn print_shell_description(username: String,current_location: Vec<String> ) {
    let mut username_directory = String::from("[");
    username_directory += username.as_str();
    username_directory += " ";
    username_directory += match current_location.last() {
        Some(last_value) => {
            last_value
        },
        None => "unknown",
    };
    username_directory += "/]$ ";
    
    
    print!("{}",username_directory.green_front().bold());
    std::io::stdout().flush().unwrap();
}


/***********EXECUTERS**************/

fn execute_command(mut command_list:VecDeque<String>) {


    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];

    if command_list.len() >= 2 {

        let mut result = String::new();
        let first_command = command_list.pop_front().unwrap();
        let last_command = command_list.pop_back().unwrap();

        ///executing first command in piped_commands
        match execute_first_command(first_command) {
            Some(output) => {
                result = output;
            },
            None => {
                return;
            }
        }
        ///executing middle command in piped_commands
        for command in command_list {

            let output = execute_middle_command(command, result);
            match output {
                Some(output) => {
                    result = output;
                }
                None => {
                    return;
                }
            }
        }
        
        ////executing last command in piped_commands
        execute_last_command(last_command, result);

    }
    else 
    {
        execute_direct_command(command_list[0].clone());
    }
    //exit(0);
}


fn execute_first_command(first_command: String) -> Option<String>{
    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];
    let mut buf = single_command_vector(first_command);
    if(buf.len() == 0) {
        println!("cannont start with pipe");
        std::io::stdout().flush().unwrap();
        return None;
    }
    if(builtin_command_list.contains(&buf[0])){
        let output = execute_env_commands(buf);
        return output;
    }

    let mut result = String::new();

    let mut output = Command::new(&buf[0]);
    for i in (1..buf.len()) {
        output.arg(&buf[i]);
    }
    output.stdout(Stdio::piped());

    let mut child = match output.spawn() {
        Ok(child) => {
            child
        }
        Err(_) => {
            println!("failed to spawn child");
            return None;
        }
    };
    if let Some(mut child_out) = child.stdout.take() {
        child_out.read_to_string(&mut result);
    } else {
        println!("failed to write the output in pipe");
        return None;
    }
    child.wait();
    return Some(result);
}

fn execute_last_command(last_command: String,mut result: String) {

    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];
    let mut buf = single_command_vector(last_command);
    if(buf.len() == 0) {
        println!("cannont end with pipe");
        std::io::stdout().flush().unwrap();
        return;
    }
    if(builtin_command_list.contains(&buf[0])){
        match execute_env_commands(buf) {
            Some(output) => {
                println!("{}",output);
            }
            None => {
            }
        }
        return;
    }

    let mut output = Command::new(&buf[0]);
    for i in (1..buf.len()) {
        output.arg(&buf[i]);
    }
    output.stdin(Stdio::piped());
    let mut child = match output.spawn() {
        Ok(child) => {
            child
        },
        Err(_) => {
            println!("failed to spawn child");
            return;
        }
    };
    child.stdin = if let Some(mut child_in) = child.stdin.take() {
        child_in.write(result.as_bytes());
        Some(child_in)
    } else {
        None
    };
    child.wait();

}

fn execute_middle_command(command: String, mut result: String) -> Option<String> {

    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];
    let mut buf = single_command_vector(command);
    if(buf.len() == 0) {
        println!("empty command between pipes");
        std::io::stdout().flush().unwrap();
        return None;
    }
    if(builtin_command_list.contains(&buf[0])){
        let output = execute_env_commands(buf);
        return output;
    }

    let mut output = Command::new(&buf[0]);
    for i in (1..buf.len()) {
        output.arg(&buf[i]);
    }
    output.stdout(Stdio::piped());
    output.stdin(Stdio::piped());
    let mut child = output.spawn().unwrap();
    
    if let Some(mut child_in) = child.stdin.take() {
        child_in.write(result.as_bytes());
    } else {
        println!("not able to read result from previous pipe");
        return None;
    }
    result.clear();
    if let Some(mut child_out) = child.stdout.take() {
        child_out.read_to_string(&mut result);
    } else {
        println!("failed to write the output in pipe");
        return None;
    }
    child.wait();

    return Some(result);
}






fn execute_direct_command(command: String) {

    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];
    let buf = single_command_vector(command);

    if buf.len() == 0 {
        println!("");
        return;
    }
    if(builtin_command_list.contains(&buf[0])){
        let output = execute_env_commands(buf);
        match output {
            Some(output) => {
                println!("{}",output);
            },
            None => {return;}
        }
        return;
    }
        
    let mut output = Command::new(&buf[0]);
    for i in (1..buf.len()) {
        output.arg(&buf[i]);
    }
    let child = output.spawn();
    match child {
        Ok(mut child) => {
            child.wait();
        },
        Err(_) => {
            println!("No such file or directory");
        }
    }
}

fn single_command_vector(buf:String) -> Vec<String> {
    let buf = String::from(buf.trim());
    let buf: Vec<String> = buf.split_whitespace().map(|args|  String::from(args.trim())).collect();
    return buf;
}

fn execute_env_commands(buf:Vec<String>) ->Option<String> {
    let  builtin_command_list = [String::from("exit"),String::from("pwd"),String::from("cd"),String::from("export"),String::from("unset")];
    if &buf[0] == &builtin_command_list[1] {

        match env::current_dir() {
            Ok(path) => {
                match path.to_str() {
                    Some(path) => {
                        return Some(String::from(path));
                    }
                    None => {
                        return Some(String::from("unknown"));
                    }
                }
            },
            Err(_) => {
                return Some(String::from("unknown"));
            }
        }
        
    } else if &buf[0] == &builtin_command_list[2] {
        handle_cd(buf);
        return None;

    } else {
        println!("{}","to implement".red_front());
        exit(0);
        return None;
    }
}

fn handle_cd(buf:Vec<String>) {
    if(buf.len() == 1) {
        return;
    }
    match env::set_current_dir(&buf[1]) {
        Ok(_) => {}
        Err(_) => {
            println!("{}","No such directory/file exist".red_front());
        }
    }
}

fn run_command(command_list: VecDeque<String>) {
    execute_command(command_list);
}


fn separate_pipes(buf: String) -> VecDeque<String> {
    let buf: VecDeque<String> = buf.split("|").map(|args| String::from(args.trim())).collect();
    return buf;
}

