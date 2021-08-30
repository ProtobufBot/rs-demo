use crate::onebot::Message;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::iter::FromIterator;
use std::array::IntoIter;
use std::ops::Add;
use std::str::FromStr;
use std::ops::Deref;

pub fn text(text: &str) -> Message {
    return Message {
        r#type: "text".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("text".to_string(), text.to_string()),
        ])),
    };
}

pub fn at(qq: i64) -> Message {
    return Message {
        r#type: "at".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("at".to_string(), qq.to_string()),
        ])),
    };
}

pub fn face(id: i32) -> Message {
    return Message {
        r#type: "face".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("id".to_string(), id.to_string()),
        ])),
    };
}

pub fn share(url: &str, title: &str, content: &str, image: &str) -> Message {
    return Message {
        r#type: "share".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
            ("title".to_string(), title.to_string()),
            ("content".to_string(), content.to_string()),
            ("image".to_string(), image.to_string()),
        ])),
    };
}


impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

impl Add<Message> for Message {
    type Output = Vec<Message>;

    fn add(self, rhs: Message) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&Message> for Message {
    type Output = Vec<Message>;

    fn add(self, rhs: &Message) -> Self::Output {
        &self + rhs.clone()
    }
}

impl Add<Message> for &Message {
    type Output = Vec<Message>;

    fn add(self, rhs: Message) -> Self::Output {
        self + &rhs
    }
}

impl Add<&Message> for &Message {
    type Output = Vec<Message>;

    fn add(self, rhs: &Message) -> Self::Output {
        vec![self.clone(), rhs.clone()]
    }
}

impl Add<Vec<Message>> for Message {
    type Output = Vec<Message>;

    fn add(self, rhs: Vec<Message>) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&Vec<Message>> for Message {
    type Output = Vec<Message>;

    fn add(self, rhs: &Vec<Message>) -> Self::Output {
        &self + rhs
    }
}

impl Add<Vec<Message>> for &Message {
    type Output = Vec<Message>;

    fn add(self, rhs: Vec<Message>) -> Self::Output {
        self + &rhs
    }
}

impl Add<&Vec<Message>> for &Message {
    type Output = Vec<Message>;

    fn add(self, rhs: &Vec<Message>) -> Self::Output {
        let mut ret = vec![self.clone()];
        for msg in rhs {
            ret.push(msg.clone())
        }
        return ret;
    }
}

impl Add<Message> for Vec<Message> {
    type Output = Vec<Message>;

    fn add(self, rhs: Message) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&Message> for Vec<Message> {
    type Output = Vec<Message>;

    fn add(self, rhs: &Message) -> Self::Output {
        &self + rhs
    }
}

impl Add<Message> for &Vec<Message> {
    type Output = Vec<Message>;

    fn add(self, rhs: Message) -> Self::Output {
        self + &rhs
    }
}

impl Add<&Message> for &Vec<Message> {
    type Output = Vec<Message>;

    fn add(self, rhs: &Message) -> Self::Output {
        let mut ret: Vec<Message> = self.clone();
        ret.push(rhs.clone());
        return ret;
    }
}
