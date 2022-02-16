pub trait IdEncoder {
    type Output;
    fn from(self, data: &[u8]) -> Self;
}

pub struct Abc {
    pub name: String,
}

impl AsRef<[u8]> for Abc {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

fn hello<T: AsRef<[u8]>>(data: T) {
    println!("{:?}", data.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        hello(Abc {
            name: "Abc".to_owned(),
        });
    }
}
