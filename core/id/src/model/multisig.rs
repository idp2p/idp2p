#[derive(Debug, Serialize, Deserialize)]
pub struct IdMultiSig {
    m: u8,
    n: u8,
}

impl IdMultiSig {
    /// Creates a new `IdMultiSig` instance after validating that `m <= n`.
    pub fn new(m: u8, n: u8) -> Result<Self> {
        if m == 0 {
            bail!("The number of required signers `m` must be greater than 0.");
        }
        if m > n {
            bail!("The number of required signers `m` should be less than or equal to the total number of signers `n`.");
        }
        Ok(Self { m, n })
    }
}
