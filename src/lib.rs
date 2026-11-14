mod pb;

use pb::{Transfer, Transfers};
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// A Substreams map emits one protobuf Transfers envelope per block.
/// scripts/export_jsonl.py flattens this envelope to one JSON object per transfer.
#[substreams::handlers::map]
pub fn map_transfers(block: Block) -> Result<Transfers, Error> {
    let mut transfers = Vec::new();

    // The library's transactions() iterator excludes failed transactions.
    for tx in block.transactions() {
        // walk_instructions includes both top-level instructions and CPI instructions,
        // preserving instruction order and resolving address lookup table accounts.
        for ix in tx.walk_instructions() {
            let program_id = ix.program_id().to_string();
            let accounts: Vec<String> = ix.accounts().iter().map(|a| a.to_string()).collect();
            if let Some(transfer) = decode_transfer(&program_id, &accounts, ix.data()) {
                transfers.push(transfer);
            }
        }
    }
    Ok(Transfers { transfers })
}

/// Decode only recognized instructions. Never panic on untrusted blockchain data.
/// The addresses are *instruction account addresses*: for SPL these are token
/// accounts, not necessarily the wallet owner addresses.
fn decode_transfer(program: &str, accounts: &[String], data: &[u8]) -> Option<Transfer> {
    let (from_index, to_index, amount) = if program == SYSTEM_PROGRAM {
        if data.len() < 12 { return None; }
        let discriminator = u32::from_le_bytes(data.get(0..4)?.try_into().ok()?);
        let amount = u64::from_le_bytes(data.get(4..12)?.try_into().ok()?);
        match discriminator {
            2 if accounts.len() >= 2 => (0, 1, amount), // Transfer
            11 if accounts.len() >= 3 => (0, 2, amount), // TransferWithSeed
            _ => return None,
        }
    } else if program == TOKEN_PROGRAM || program == TOKEN_2022_PROGRAM {
        let discriminator = *data.first()?;
        let amount = u64::from_le_bytes(data.get(1..9)?.try_into().ok()?);
        match discriminator {
            3 if accounts.len() >= 2 => (0, 1, amount), // Transfer
            12 if accounts.len() >= 3 && data.len() >= 10 => (0, 2, amount), // TransferChecked
            _ => return None,
        }
    } else {
        return None;
    };
    Some(Transfer {
        from: accounts.get(from_index)?.clone(),
        to: accounts.get(to_index)?.clone(),
        amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("account-{i}")).collect()
    }
    fn system_data(tag: u32, amount: u64) -> Vec<u8> {
        [tag.to_le_bytes().as_slice(), amount.to_le_bytes().as_slice()].concat()
    }
    fn token_data(tag: u8, amount: u64) -> Vec<u8> {
        [vec![tag], amount.to_le_bytes().to_vec()].concat()
    }
    #[test]
    fn native_transfer() {
        let t = decode_transfer(SYSTEM_PROGRAM, &acc(2), &system_data(2, 325253)).unwrap();
        assert_eq!((t.from.as_str(), t.to.as_str(), t.amount), ("account-0", "account-1", 325253));
    }
    #[test]
    fn native_seed_transfer() {
        let t = decode_transfer(SYSTEM_PROGRAM, &acc(3), &system_data(11, 9)).unwrap();
        assert_eq!((t.to.as_str(), t.amount), ("account-2", 9));
    }
    #[test]
    fn spl_transfer_and_checked() {
        for program in [TOKEN_PROGRAM, TOKEN_2022_PROGRAM] {
            assert_eq!(decode_transfer(program, &acc(2), &token_data(3, 42)).unwrap().amount, 42);
            let mut checked = token_data(12, 99);
            checked.push(6);
            let t = decode_transfer(program, &acc(4), &checked).unwrap();
            assert_eq!((t.to.as_str(), t.amount), ("account-2", 99));
        }
    }
    #[test]
    fn boundaries_and_zero() {
        for amount in [0, 1, u64::MAX] {
            assert_eq!(decode_transfer(SYSTEM_PROGRAM, &acc(2), &system_data(2, amount)).unwrap().amount, amount);
        }
    }
    #[test]
    fn rejects_invalid_data_without_panics() {
        for program in [SYSTEM_PROGRAM, TOKEN_PROGRAM, TOKEN_2022_PROGRAM] {
            for length in 0..12 {
                let data = vec![0u8; length];
                assert!(decode_transfer(program, &acc(4), &data).is_none());
            }
        }
        assert!(decode_transfer("not-a-transfer-program", &acc(4), &system_data(2, 5)).is_none());
        assert!(decode_transfer(SYSTEM_PROGRAM, &acc(1), &system_data(2, 5)).is_none());
        assert!(decode_transfer(SYSTEM_PROGRAM, &acc(4), &system_data(3, 5)).is_none());
        assert!(decode_transfer(TOKEN_PROGRAM, &acc(4), &token_data(9, 5)).is_none());
        assert!(decode_transfer(TOKEN_PROGRAM, &acc(1), &token_data(3, 5)).is_none());
    }
    #[test]
    fn checked_transfer_requires_decimals_byte() {
        assert!(decode_transfer(TOKEN_PROGRAM, &acc(4), &token_data(12, 10)).is_none());
    }
}
