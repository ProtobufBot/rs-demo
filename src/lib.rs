pub mod bot;

pub mod onebot {
    include!(concat!(env!("OUT_DIR"), "/onebot.rs"));
}
