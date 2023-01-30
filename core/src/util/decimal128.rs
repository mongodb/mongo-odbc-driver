use likely_stable::unlikely;

use bson::Decimal128;

fn set_bytes(x: &mut u32, bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        *x |= (*b as u32) << (i * 8);
    }
}

pub trait Decimal128Plus {
    fn to_formatted_string(&self) -> String;

    fn not_zero(&self) -> bool;
}

const BSON_DECIMAL128_STRING: usize = 43;
const BSON_DECIMAL128_INF: &str = "Infinity";
const BSON_DECIMAL128_NAN: &str = "NaN";
const COMBINATION_MASK: u32 = 0x1f; // Extract least significant 5 bits
const EXPONENT_MASK: u32 = 0x3fff; // Extract least significant 14 bits
const COMBINATION_INFINITY: u32 = 30; // Value of combination field for Inf
const COMBINATION_NAN: u32 = 31; // Value of combination field for NaN
const EXPONENT_BIAS: u32 = 6176; // decimal128 exponent bias

impl Decimal128Plus for Decimal128 {
    fn to_formatted_string(&self) -> String {
        self.bytes().to_formatted_string()
    }

    fn not_zero(&self) -> bool {
        self.bytes().not_zero()
    }
}

impl Decimal128Plus for [u8; 16] {
    // algorithm from https://github.com/mongodb/mongo-c-driver/blob/master/src/libbson/src/bson/bson-decimal128.c
    fn to_formatted_string(&self) -> String {
        let mut str_out = String::new();

        // Note: bits in this routine are referred to starting at 0,
        // from the sign bit, towards the coefficient.
        // This is big-endian. The rust storage format is little-endian.
        let mut high = 0u32; // bits 0 - 31
        let mut midh = 0u32; // bits 32 - 63
        let mut midl = 0u32; // bits 64 - 95
        let mut low = 0u32; // bits 96 - 127
        set_bytes(&mut low, self.get(0..4).unwrap());
        set_bytes(&mut midl, self.get(4..8).unwrap());
        set_bytes(&mut midh, self.get(8..12).unwrap());
        set_bytes(&mut high, self.get(12..16).unwrap());
        let mut significand = [0u8; 36]; // the base-10 digits in the significand
        let mut significand_index = 0usize; // read index into significand
        let mut is_zero = false; // true if the number is zero

        if (high as i32) < 0 {
            //negative
            str_out.push('-');
        }

        let combination = (high >> 26) & COMBINATION_MASK; // bits 1 - 5

        let (biased_exponent, significand_msb) = if unlikely((combination >> 3) == 3) {
            match combination {
                COMBINATION_INFINITY => return str_out + BSON_DECIMAL128_INF,
                COMBINATION_NAN => return str_out + BSON_DECIMAL128_NAN,
                _ => (
                    (high >> 15) & EXPONENT_MASK,
                    (0x8 + ((high >> 14) & 0x1)) as u8,
                ),
            }
        } else {
            ((high >> 17) & EXPONENT_MASK, ((high >> 14) & 0x7) as u8)
        };
        // unbiased exponent
        let exponent = biased_exponent as i32 - EXPONENT_BIAS as i32;
        // Create string of significand digits

        // Convert the 114-bit binary number represented by
        // (high, midh, midl, low) to at most 34 decimal
        // digits through modulo and division.
        let significand128_part0 =
            (high & 0x3fff) as u64 + (((significand_msb & 0xf) as u64) << 14);
        let mut significand128 = (significand128_part0 as u128) << 96;
        significand128 |= (midh as u128) << 64;
        significand128 |= (midl as u128) << 32;
        significand128 |= low as u128;
        if significand128 == 0
            // The significand is non-canonical or zero.
            // In order to preserve compatibility with the densely packed decimal
            // format, the maximum value for the significand of decimal128 is
            // 1e34 - 1.  If the value is greater than 1e34 - 1, the IEEE 754
            // standard dictates that the significand is interpreted as zero.
            || significand128_part0 >= (1 << 17)
        {
            is_zero = true;
        } else {
            let mut k = 3isize;
            while k >= 0 {
                const DIVISOR: u128 = 1000 * 1000 * 1000;
                let mut least_digits = significand128 % DIVISOR;
                significand128 /= DIVISOR;

                if least_digits == 0 {
                    k -= 1;
                    continue;
                }

                let mut j = 8isize;
                while j >= 0 {
                    significand[(k * 9 + j) as usize] = (least_digits % 10) as u8;
                    least_digits /= 10;
                    j -= 1;
                }
                k -= 1;
            }
        }

        // Output format options:
        // Scientific - [-]d.dddE(+/-)dd or [-]dE(+/-)dd
        // Regular    - ddd.ddd

        let mut significand_digits = if is_zero {
            significand[significand_index] = 0;
            1
        } else {
            let mut significand_digits = 36;
            while significand[significand_index] == 0 {
                significand_digits -= 1;
                significand_index += 1;
            }
            significand_digits
        };

        // the exponent if scientific notation is used
        let scientific_exponent = significand_digits - 1 + exponent;

        // The scientific exponent checks are dictated by the string conversion
        // specification and are somewhat arbitrary cutoffs.
        //
        // We must check exponent > 0, because if this is the case, the number
        // has trailing zeros.  However, we *cannot* output these trailing zeros,
        // because doing so would change the precision of the value, and would
        // change stored data if the string converted number is round tripped.
        //
        if scientific_exponent < -6 || exponent > 0 {
            // Scientific format
            str_out.push(char::from_digit(significand[significand_index] as u32, 10).unwrap());
            significand_index += 1;
            significand_digits -= 1;

            if significand_digits != 0 {
                str_out.push('.');
            }

            let mut i = 0;
            while i < significand_digits && str_out.len() < 36 {
                str_out.push(char::from_digit(significand[significand_index] as u32, 10).unwrap());
                significand_index += 1;
                i += 1;
            }

            // Exponent
            str_out.push('E');
            if scientific_exponent > 0 {
                str_out.push_str(format!("+{scientific_exponent}").as_str());
            } else {
                str_out.push_str(format!("{scientific_exponent}").as_str());
            }
            return str_out;
        }
        if exponent >= 0 {
            let mut i = 0;
            while i < significand_digits && str_out.len() < 36 {
                str_out.push(char::from_digit(significand[significand_index] as u32, 10).unwrap());
                significand_index += 1;
                i += 1;
            }
            return str_out;
        }
        let mut radix_position = significand_digits + exponent;
        if radix_position > 0 {
            // non-zero digits before radix
            let mut i = 0;
            while i < radix_position && str_out.len() < BSON_DECIMAL128_STRING {
                str_out.push(char::from_digit(significand[significand_index] as u32, 10).unwrap());
                significand_index += 1;
                i += 1;
            }
        } else {
            // leading zero before radix point
            str_out.push('0');
        }

        str_out.push('.');
        while radix_position < 0 {
            str_out.push('0');
            radix_position += 1;
        }

        let mut i = 0;
        // Note we do not have radix_position - 1 in max here unlike the C code.
        // This is because the C code has radix_position++ in the while condition,
        // so `C radix postion` == `rust radix position` + 1 at this point.
        while (i < (significand_digits - std::cmp::max(radix_position, 0)))
            && str_out.len() < BSON_DECIMAL128_STRING
        {
            str_out.push(char::from_digit(significand[significand_index] as u32, 10).unwrap());
            significand_index += 1;
            i += 1;
        }
        str_out
    }

    fn not_zero(&self) -> bool {
        let mut high = 0u32; // bits 0 - 31
        let mut midh = 0u32; // bits 32 - 63
        let mut midl = 0u32; // bits 64 - 95
        let mut low = 0u32; // bits 96 - 127
        set_bytes(&mut low, self.get(0..4).unwrap());
        set_bytes(&mut midl, self.get(4..8).unwrap());
        set_bytes(&mut midh, self.get(8..12).unwrap());
        set_bytes(&mut high, self.get(12..16).unwrap());
        let combination = (high >> 26) & COMBINATION_MASK; // bits 1 - 5
        let significand_msb = if unlikely((combination >> 3) == 3) {
            match combination {
                COMBINATION_INFINITY => return true,
                COMBINATION_NAN => return true,
                _ => 0x8 + ((high >> 14) & 0x1) as u8,
            }
        } else {
            ((high >> 14) & 0x7) as u8
        };
        // Convert the 114-bit binary number represented by
        // (high, midh, midl, low) to at most 34 decimal
        // digits through modulo and division.
        let significand128_part0 =
            (high & 0x3fff) as u64 + (((significand_msb & 0xf) as u64) << 14);
        let mut significand128 = (significand128_part0 as u128) << 96;
        significand128 |= (midh as u128) << 64;
        significand128 |= (midl as u128) << 32;
        significand128 |= low as u128;
        !(significand128 == 0
            // The significand is non-canonical or zero.
            // In order to preserve compatibility with the densely packed decimal
            // format, the maximum value for the significand of decimal128 is
            // 1e34 - 1.  If the value is greater than 1e34 - 1, the IEEE 754
            // standard dictates that the significand is interpreted as zero.
            || significand128_part0 >= (1 << 17))
    }
}

#[cfg(test)]
mod unit {
    use super::Decimal128Plus;

    mod to_formatted_string {
        use super::Decimal128Plus;

        #[test]
        fn nan() {
            assert_eq!(
                "NaN".to_string(),
                [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124].to_formatted_string()
            );
        }

        #[test]
        fn inf() {
            assert_eq!(
                "Infinity".to_string(),
                [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120].to_formatted_string()
            );
        }

        #[test]
        fn neg_inf() {
            assert_eq!(
                "-Infinity".to_string(),
                [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248].to_formatted_string()
            );
        }

        #[test]
        fn zero() {
            assert_eq!(
                "0.0".to_string(),
                [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 48].to_formatted_string()
            );
        }

        #[test]
        fn neg_zero() {
            assert_eq!(
                "-0.0".to_string(),
                [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 176].to_formatted_string()
            );
        }

        #[test]
        fn one() {
            // not sure why it drops .0 from 1 and not 0, but this is what the server does,
            // the algorithm is correct.
            assert_eq!(
                "1".to_string(),
                [1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 48].to_formatted_string()
            );
        }

        #[test]
        fn neg_one() {
            // not sure why it drops .0 from -1 and not -0, but this is what the server does,
            // the algorithm is correct.
            assert_eq!(
                "-1".to_string(),
                [1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 176].to_formatted_string()
            );
        }

        #[test]
        fn big() {
            assert_eq!(
                "412345123451234512345".to_string(),
                [217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 48]
                    .to_formatted_string()
            );
        }

        #[test]
        fn neg_big() {
            assert_eq!(
                "-412345123451234512345".to_string(),
                [217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 176]
                    .to_formatted_string()
            );
        }

        #[test]
        fn really_big() {
            assert_eq!(
                "1.8E+309".to_string(),
                [18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 168, 50].to_formatted_string()
            );
        }

        #[test]
        fn neg_really_big() {
            assert_eq!(
                "-1.8E+309".to_string(),
                [18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 168, 178].to_formatted_string()
            );
        }

        #[test]
        fn really_small() {
            assert_eq!(
                "1.8E-309".to_string(),
                [18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 212, 45].to_formatted_string()
            );
        }

        #[test]
        fn neg_really_small() {
            assert_eq!(
                "-1.8E-309".to_string(),
                [18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 212, 173].to_formatted_string()
            );
        }

        #[test]
        fn pi() {
            assert_eq!(
                "3.1415926535897932384".to_string(),
                [96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 48]
                    .to_formatted_string()
            );
        }

        #[test]
        fn neg_pi() {
            assert_eq!(
                "-3.1415926535897932384".to_string(),
                [96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 176]
                    .to_formatted_string()
            );
        }
    }

    mod not_zero {
        use super::Decimal128Plus;

        fn nan() {
            assert!([0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124].not_zero());
        }

        #[test]
        fn inf() {
            assert!([0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120].not_zero());
        }

        #[test]
        fn neg_inf() {
            assert!([0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248].not_zero());
        }

        #[test]
        fn zero() {
            assert!(![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 48].not_zero());
        }

        #[test]
        fn neg_zero() {
            assert!(![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 176].not_zero());
        }

        #[test]
        fn one() {
            assert!([1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 48].not_zero());
        }

        #[test]
        fn neg_one() {
            assert!([1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 176].not_zero());
        }

        #[test]
        fn big() {
            assert!([217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 48].not_zero());
        }

        #[test]
        fn neg_big() {
            assert!([217u8, 109, 109, 175, 20, 41, 112, 90, 22, 0, 0, 0, 0, 0, 64, 176].not_zero());
        }

        #[test]
        fn really_big() {
            assert!([18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 168, 50].not_zero());
        }

        #[test]
        fn neg_really_big() {
            assert!([18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 168, 178].not_zero());
        }

        #[test]
        fn really_small() {
            assert!([18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 212, 45].not_zero());
        }

        #[test]
        fn neg_really_small() {
            assert!([18u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 212, 173].not_zero());
        }

        #[test]
        fn pi() {
            assert!([96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 48].not_zero());
        }

        #[test]
        fn neg_pi() {
            assert!([96, 226, 246, 85, 188, 202, 251, 179, 1, 0, 0, 0, 0, 0, 26, 176].not_zero());
        }
    }
}
