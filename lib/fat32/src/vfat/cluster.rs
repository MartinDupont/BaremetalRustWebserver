#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

// raw num is the cluster number. I.e. the nth cluster. NOT the cluster address.
impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    pub fn raw(&self) -> u32 { self.0 }
}

// TODO: Implement any useful helper methods on `Cluster`.
