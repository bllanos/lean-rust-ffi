use lean_sys::{lean_dec, lean_obj_res};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct U128AsU64Pair {
    high: u64,
    low: u64,
}

impl From<u128> for U128AsU64Pair {
    fn from(n: u128) -> Self {
        let high: u128 = n >> U128AsU64Pair::ELEMENT_BIT_SIZE;
        let low: u128 = n & (<u64 as Into<u128>>::into(u64::MAX));
        Self {
            high: high.try_into().unwrap(),
            low: low.try_into().unwrap(),
        }
    }
}

impl From<U128AsU64Pair> for u128 {
    fn from(n: U128AsU64Pair) -> Self {
        let U128AsU64Pair { high, low } = n;
        (<u64 as Into<u128>>::into(high) << U128AsU64Pair::ELEMENT_BIT_SIZE)
            + <u64 as Into<u128>>::into(low)
    }
}

impl U128AsU64Pair {
    const ELEMENT_BIT_SIZE: u32 = u64::BITS;

    pub fn to_lean_nat(self) -> lean_obj_res {
        let Self { high, low } = self;
        unsafe {
            let high_nat = lean_sys::lean_uint64_to_nat(high);
            let shift = lean_sys::lean_uint32_to_nat(Self::ELEMENT_BIT_SIZE);

            let high_nat_shifted = lean_sys::lean_nat_shiftl(high_nat, shift);
            lean_dec(high_nat);
            lean_dec(shift);

            let low_nat = lean_sys::lean_uint64_to_nat(low);
            let result = lean_sys::lean_nat_add(high_nat_shifted, low_nat);
            lean_dec(high_nat_shifted);
            lean_dec(low_nat);

            result
        }
    }
}

pub fn u128_to_lean_nat(n: u128) -> lean_obj_res {
    U128AsU64Pair::from(n).to_lean_nat()
}

#[cfg(test)]
mod tests {
    use super::*;

    use lean_sys::lean_obj_arg;

    const HIGH: u64 = (1u64 << (u64::BITS - 1)) + 1u64;
    const LOW: u64 = HIGH + (1u64 << (u64::BITS / 2));

    fn make_u128() -> u128 {
        u128::from_le_bytes(
            [LOW.to_le_bytes(), HIGH.to_le_bytes()]
                .concat()
                .try_into()
                .unwrap(),
        )
    }

    fn make_u128asu64pair() -> U128AsU64Pair {
        U128AsU64Pair {
            high: HIGH,
            low: LOW,
        }
    }

    unsafe fn lean_nat_to_u128(nat: lean_obj_arg) -> u128 {
        let high_u64;
        let low_u64;
        unsafe {
            let shift = lean_sys::lean_uint32_to_nat(u64::BITS);
            let high = lean_sys::lean_nat_big_shiftr(nat, shift);
            let high_shifted = lean_sys::lean_nat_shiftl(high, shift);
            lean_dec(shift);
            let low = lean_sys::lean_nat_big_sub(nat, high_shifted);
            lean_dec(high_shifted);
            high_u64 = lean_sys::lean_uint64_of_big_nat(high);
            lean_dec(high);
            low_u64 = lean_sys::lean_uint64_of_big_nat(low);
            lean_dec(low);
        }

        U128AsU64Pair {
            high: high_u64,
            low: low_u64,
        }
        .into()
    }

    #[test]
    fn from_u128() {
        let pair = make_u128asu64pair();
        let n = make_u128();
        assert_eq!(U128AsU64Pair::from(n), pair);
    }

    #[test]
    fn into_u128() {
        let pair = make_u128asu64pair();
        let n = make_u128();
        assert_eq!(<U128AsU64Pair as Into<u128>>::into(pair), n);
    }

    #[test]
    fn test_u128_to_lean_nat() {
        let n = make_u128();
        let nat = u128_to_lean_nat(n);
        assert_eq!(unsafe { lean_nat_to_u128(nat) }, n);
    }
}
