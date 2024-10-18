/// Network information either mainet of testnet with a chain id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet(u32),
}
