use crate::onebot::Message;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::iter::FromIterator;
use std::array::IntoIter;
use std::ops::Add;
use std::str::FromStr;
use std::ops::Deref;

impl Add<Message> for Message {
    type Output = Vec<Message>;

    fn add(self, msg: Message) -> Vec<Message> {
        vec![self, msg.clone()]
    }
}

impl Add<&Message> for Message {
    type Output = Vec<Message>;

    fn add(self, msg: &Message) -> Vec<Message> {
        vec![self, msg.clone()]
    }
}

impl Add<&Vec<Message>> for Message {
    type Output = Vec<Message>;

    fn add(self, msgs: &Vec<Message>) -> Vec<Message> {
        let mut ret = vec![self];
        for msg in msgs {
            ret.push(msg.clone())
        }
        return ret;
    }
}

impl Add<&Message> for Vec<Message> {
    type Output = Vec<Message>;

    fn add(self, msg: &Message) -> Vec<Message> {
        let mut ret = vec![];
        for msg in self {
            ret.push(msg.clone())
        }
        ret.push(msg.clone());
        return ret;
    }
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}


pub fn text(text: &str) -> Message {
    return Message {
        r#type: "text".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([("text".to_string(), text.to_string())])),
    };
}

pub fn at(qq: i64) -> Message {
    return Message {
        r#type: "at".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([("at".to_string(), qq.to_string())])),
    };
}


pub fn face(id: i32) -> Message {
    return Message {
        r#type: "face".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([("id".to_string(), id.to_string())])),
    };
}