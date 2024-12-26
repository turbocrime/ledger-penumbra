/// Network information either mainet of testnet with a chain id.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub enum Network {
    Mainnet,
    Testnet(u32),
}
