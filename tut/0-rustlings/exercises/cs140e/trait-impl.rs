// FIXME: Make me pass! Diff budget: 25 lines.

#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16),
}

impl PartialEq for Duration {

    fn eq(&self, other: &Self) -> bool {
        let thousand: u64 = 1000;
        let sixty: u64 = 60;

        let my_milliseconds = match self {
            Duration::MilliSeconds(val) => *val,
            Duration::Seconds(val) => *val as u64 * thousand ,
            Duration::Minutes(val) => *val as u64 * thousand * sixty,
        };

        let other_milliseconds = match other {
            Duration::MilliSeconds(val) => *val,
            Duration::Seconds(val) => *val as u64 * thousand ,
            Duration::Minutes(val) => *val as u64 * thousand * sixty,
        };

        my_milliseconds == other_milliseconds
    }
}

// What traits does `Duration` need to implement?

#[test]
fn traits() {
    assert_eq!(Duration::Seconds(120), Duration::Minutes(2));
    assert_eq!(Duration::Seconds(420), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(420000), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(43000), Duration::Seconds(43));
}
