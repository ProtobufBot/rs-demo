pub mod bot;
pub mod msg;

pub mod onebot {
    include!(concat!(env!("OUT_DIR"), "/onebot.rs"));
}
