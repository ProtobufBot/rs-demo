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

pub fn image(url: &str) -> Message {
    return Message {
        r#type: "image".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
        ])),
    };
}

pub fn record(url: &str) -> Message {
    return Message {
        r#type: "record".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
        ])),
    };
}

pub fn flash(url: &str) -> Message {
    return Message {
        r#type: "image".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
            ("type".to_string(), "flash".to_string()),
        ])),
    };
}

pub fn show(url: &str, effect_id: i32) -> Message {
    return Message {
        r#type: "image".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
            ("type".to_string(), "show".to_string()),
            ("effect_id".to_string(), effect_id.to_string()),
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

pub fn at_all() -> Message {
    return Message {
        r#type: "at".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("qq".to_string(), "all".to_string()),
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

pub fn poke(qq: i64) -> Message {
    return Message {
        r#type: "poke".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("qq".to_string(), qq.to_string()),
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

pub fn light_app(content: &str) -> Message {
    return Message {
        r#type: "light_app".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("content".to_string(), content.to_string()),
        ])),
    };
}

pub fn xml(id: i32, content: &str) -> Message {
    return Message {
        r#type: "service".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("sub_type".to_string(), "xml".to_string()),
            ("id".to_string(), id.to_string()),
            ("content".to_string(), content.to_string()),
        ])),
    };
}

pub fn json(id: i32, content: &str) -> Message {
    return Message {
        r#type: "service".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("sub_type".to_string(), "json".to_string()),
            ("id".to_string(), id.to_string()),
            ("content".to_string(), content.to_string()),
        ])),
    };
}

pub fn reply(message_id: i32) -> Message {
    return Message {
        r#type: "reply".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("message_id".to_string(), message_id.to_string()),
        ])),
    };
}

pub fn sleep(time: i64) -> Message {
    return Message {
        r#type: "sleep".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("time".to_string(), time.to_string()),
        ])),
    };
}

pub fn tts(text: &str) -> Message {
    return Message {
        r#type: "tts".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("text".to_string(), text.to_string()),
        ])),
    };
}

pub fn video(url: &str, cover: &str, cache: bool) -> Message {
    return Message {
        r#type: "video".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("url".to_string(), url.to_string()),
            ("cover".to_string(), cover.to_string()),
            ("cache".to_string(), if cache { "1".to_string() } else { "0".to_string() }),
        ])),
    };
}

pub fn gift(qq: i64, id: i32) -> Message {
    return Message {
        r#type: "gift".to_string(),
        data: HashMap::<_, _>::from_iter(IntoIter::new([
            ("qq".to_string(), qq.to_string()),
            ("id".to_string(), id.to_string()),
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
