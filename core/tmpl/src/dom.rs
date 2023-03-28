pub mod text;
use std::fmt;

use self::text::{Heading, Text, TextKind};

#[derive(PartialEq, Eq, Clone)]
pub struct Consent {
    pub header: Header,
    pub body: Body,
    pub footer: Footer,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Header {
    pub kind: Heading,
    pub text: String,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Body {
    pub sections: Vec<Section>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Footer {
    pub kind: TextKind,
    pub text: String,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Section {
    pub header: Heading,
    pub children: Vec<SectionChildKind>,
}

#[derive(PartialEq, Eq, Clone)]
pub enum SectionChildKind {
    Paragraph(Paragraph),
    List(List),
}

#[derive(PartialEq, Eq, Clone)]
pub struct Paragraph {
    pub texts: Vec<Text>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct List {
    pub children: Vec<String>,
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut content = String::new();
        for text in &self.texts {
            content = format!("{}{:?}", content, text);
        }

        write!(f, "<p>{}</p>", content)
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
        /**/
    }
}