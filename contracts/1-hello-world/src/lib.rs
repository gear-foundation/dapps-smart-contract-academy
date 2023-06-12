#![no_std]
use gstd::{debug, msg, prelude::*};
use hello_world_io::InputMessages;

static mut GREETING: Option<String> = None;

#[no_mangle]
extern "C" fn handle() {
    let input_message: InputMessages = msg::load().expect("Error in loading InputMessages");
    let greeting = unsafe { GREETING.as_ref().expect("Program hasn't been initialized") };
    match input_message {
        InputMessages::SendHelloTo(account) => {
            debug!("Message: SendHelloTo {:?}", account);
            msg::send(account, greeting, 0).expect("Error in sending Hello message to account");
        }
        InputMessages::SendHelloReply => {
            debug!("Message: SendHelloReply");
            msg::reply(greeting, 0).expect("Error in sending reply");
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    //let greeting: String = msg::load().expect("Can't decode init message");
    let greeting = String::from_utf8(msg::load_bytes().expect("Can't load an init message"))
        .expect("Can't decode to String");
    debug!("Program was initialized with message {:?}", greeting);
    unsafe { GREETING = Some(greeting) };
}

#[no_mangle]
extern "C" fn state() {
    let greeting = unsafe { GREETING.get_or_insert(Default::default()) };
    msg::reply_bytes(greeting, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}
