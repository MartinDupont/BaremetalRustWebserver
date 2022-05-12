// FIXME: Make me pass! Diff budget: 30 lines.

#[derive(Default)]
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn string<T: AsRef<str> + ToString + std::fmt::Display>(mut self, s: T) -> Self {
        self.string = Some(s.to_string());
        self
    }

    fn number(mut self, num: usize) -> Self {
        self.number = Some(num);
        self
    }
}

impl ToString for Builder {
    fn to_string(&self) -> String {
        match (&self.string, self.number) {
            (Some(s), Some(n)) => format!("{} {}", s, n),
            (Some(s), None) => format!("{}", s),
            (None, Some(n)) => format!("{}", n),
            _ => String::from("")
        }
    }
    // Implement the trait
}

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default().string("heap!".to_owned()).to_string();

    assert_eq!(c, "heap!");
}
