use std::fmt;

#[derive(PartialEq, Eq, Clone)]
pub struct Text {
    pub kind: TextKind,
    pub value: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum TextKind {
    Raw,
    Bold,
    Italic,
    Underline,
    Keyboard,
    Code,
    Blockquote,
    Emphasis,
    Abbreviation,
    Inserted,
    Deleted,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Heading {
    Header3,
    Header4,
    Header5,
    Header6,
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            TextKind::Raw => write!(f, "{}", self.value),
            TextKind::Bold => write!(f, "<b>{}</b>", self.value),
            TextKind::Italic => write!(f, "<i>{}</i>", self.value),
            TextKind::Underline => write!(f, "<u>{}</u>", self.value),
            TextKind::Keyboard => todo!(),
            TextKind::Blockquote => todo!(),
            TextKind::Emphasis => todo!(),
            TextKind::Abbreviation => todo!(),
            TextKind::Inserted => todo!(),
            TextKind::Deleted => todo!(),
            TextKind::Code => todo!(),
        }
    }
}
