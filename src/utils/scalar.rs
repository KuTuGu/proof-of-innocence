use anyhow::{anyhow, Result};
use dusk_plonk::prelude::*;

const ADDRESS_LENGTH: usize = 40;

pub fn from_addr(s: &str) -> Result<BlsScalar> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let len = s.len();
    if len != ADDRESS_LENGTH {
        return Err(anyhow!(
            "Wrong address length: {len}, should be: {ADDRESS_LENGTH}"
        ));
    }

    let u2 = u64::from_str_radix(&s[0..8], 16)?;
    let u1 = u64::from_str_radix(&s[8..24], 16)?;
    let u0 = u64::from_str_radix(&s[24..40], 16)?;
    Ok(BlsScalar([u0, u1, u2, 0]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_addr() {
        let scalar = from_addr("0x0000000000000000000000010000000000000001").unwrap();
        assert_eq!(scalar.0, [1, 1, 0, 0]);
        let scalar = from_addr("0000000000000000000000010000000000000001").unwrap();
        assert_eq!(scalar.0, [1, 1, 0, 0]);
    }

    #[test]
    #[should_panic(expected = "Wrong address length: 0, should be: 40")]
    fn test_from_addr_with_less_length() {
        from_addr("").unwrap();
    }

    #[test]
    #[should_panic(expected = "Wrong address length: 41, should be: 40")]
    fn test_from_addr_with_more_length() {
        from_addr("0xf0000000000000000000000010000000000000001").unwrap();
    }
}
