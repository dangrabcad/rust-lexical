//! Prototype for version 2 of atoi.

use util::*;

// Calculate the offset of a reference inside an array.
macro_rules! offset {
    ($arr:ident, $v:ident) => (
        distance($arr.as_ptr(), $v)
    );
}

macro_rules! to_digit {
    ($c:expr, $radix:ident) => (($c as char).to_digit($radix));
}

// STANDALONE

/// Iterate over the digits and iteratively process them.
macro_rules! standalone {
    ($value:ident, $radix:ident, $digits:ident, $offset:expr, $op:ident, $code:ident) => (
        let index = move | c: &u8 | -> usize { offset!($digits, c) + $offset };
        for c in $digits.iter() {
            let digit = match to_digit!(*c, $radix) {
                Some(v) => v,
                None    => return Err((ErrorCode::InvalidDigit, index(c)).into()),
            };
            $value = match $value.checked_mul(as_cast($radix)) {
                Some(v) => v,
                None    => return Err((ErrorCode::$code, index(c)).into()),
            };
            $value = match $value.$op(as_cast(digit)) {
                Some(v) => v,
                None    => return Err((ErrorCode::$code, index(c)).into()),
            };
        }
    );
}

// Standalone atoi processor.
#[inline]
pub(crate) fn standalone<T>(radix: u32, bytes: &[u8], is_signed: bool)
    -> Result<T>
    where T: Integer
{
    // Filter out empty inputs.
    if bytes.is_empty() {
        return Err(ErrorCode::Empty.into());
    }

    let (sign, offset, digits) = match &index!(bytes[0]) {
        b'+'              => (Sign::Positive, 1, &index!(bytes[1..])),
        b'-' if is_signed => (Sign::Negative, 1, &index!(bytes[1..])),
        _                 => (Sign::Positive, 0, bytes),
    };

    // Filter out empty inputs.
    if digits.is_empty() {
        return Err(ErrorCode::Empty.into());
    }

    // Parse the integer.
    let mut value = T::ZERO;
    if sign == Sign::Positive {
        standalone!(value, radix, digits, offset, checked_add, Overflow);
    } else {
        standalone!(value, radix, digits, offset, checked_sub, Underflow);
    }
    Ok(value)
}

/// Calculate the mantissa and the number of truncated digits from a digits iterator.
/// Preconditions: All the characters in digits are <= radix.
#[cfg(feature = "correct")]
pub(crate) fn standalone_mantissa<'a, T, Iter>(radix: u32, mut digits: Iter)
    -> (T, usize)
    where T: UnsignedInteger,
          Iter: Iterator<Item=&'a u8>
{
    let mut value: T = T::ZERO;
    while let Some(&c) = digits.next() {
        let digit = to_digit!(c, radix).unwrap();
        value = match value.checked_mul(as_cast(radix)).and_then(|v| v.checked_add(as_cast(digit))) {
            Some(v) => v,
            None    => return (value, 1 + digits.count()),
        };
    }
    (value, 0)
}

/// Handle unsigned +/- numbers and forward to implied implementation.
///  Can just use local namespace
#[inline]
pub(crate) fn standalone_unsigned<'a, T>(radix: u32, bytes: &'a [u8])
    -> Result<T>
    where T: UnsignedInteger
{
    standalone(radix, bytes, false)
}

/// Handle signed +/- numbers and forward to implied implementation.
///  Can just use local namespace
#[inline]
pub(crate) fn standalone_signed<'a, T>(radix: u32, bytes: &'a [u8])
    -> Result<T>
    where T: SignedInteger
{
    standalone(radix, bytes, true)
}

// API
// ---

// RANGE API (FFI)
generate_from_range_api!(atou8_range, atou8_radix_range, u8, standalone_unsigned);
generate_from_range_api!(atou16_range, atou16_radix_range, u16, standalone_unsigned);
generate_from_range_api!(atou32_range, atou32_radix_range, u32, standalone_unsigned);
generate_from_range_api!(atou64_range, atou64_radix_range, u64, standalone_unsigned);
generate_from_range_api!(atousize_range, atousize_radix_range, usize, standalone_unsigned);
generate_from_range_api!(atoi8_range, atoi8_radix_range, i8, standalone_signed);
generate_from_range_api!(atoi16_range, atoi16_radix_range, i16, standalone_signed);
generate_from_range_api!(atoi32_range, atoi32_radix_range, i32, standalone_signed);
generate_from_range_api!(atoi64_range, atoi64_radix_range, i64, standalone_signed);
generate_from_range_api!(atoisize_range, atoisize_radix_range, isize, standalone_signed);

#[cfg(has_i128)] generate_from_range_api!(atou128_range, atou128_radix_range, u128, standalone_unsigned);
#[cfg(has_i128)] generate_from_range_api!(atoi128_range, atoi128_radix_range, i128, standalone_signed);

// SLICE API
generate_from_slice_api!(atou8_slice, atou8_radix_slice, u8, standalone_unsigned);
generate_from_slice_api!(atou16_slice, atou16_radix_slice, u16, standalone_unsigned);
generate_from_slice_api!(atou32_slice, atou32_radix_slice, u32, standalone_unsigned);
generate_from_slice_api!(atou64_slice, atou64_radix_slice, u64, standalone_unsigned);
generate_from_slice_api!(atousize_slice, atousize_radix_slice, usize, standalone_unsigned);
generate_from_slice_api!(atoi8_slice, atoi8_radix_slice, i8, standalone_signed);
generate_from_slice_api!(atoi16_slice, atoi16_radix_slice, i16, standalone_signed);
generate_from_slice_api!(atoi32_slice, atoi32_radix_slice, i32, standalone_signed);
generate_from_slice_api!(atoi64_slice, atoi64_radix_slice, i64, standalone_signed);
generate_from_slice_api!(atoisize_slice, atoisize_radix_slice, isize, standalone_signed);

#[cfg(has_i128)] generate_from_slice_api!(atou128_slice, atou128_radix_slice, u128, standalone_unsigned);
#[cfg(has_i128)] generate_from_slice_api!(atoi128_slice, atoi128_radix_slice, i128, standalone_signed);

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "radix")]
    const DATA: [(u8, &'static str); 35] = [
        (2, "100101"),
        (3, "1101"),
        (4, "211"),
        (5, "122"),
        (6, "101"),
        (7, "52"),
        (8, "45"),
        (9, "41"),
        (10, "37"),
        (11, "34"),
        (12, "31"),
        (13, "2B"),
        (14, "29"),
        (15, "27"),
        (16, "25"),
        (17, "23"),
        (18, "21"),
        (19, "1I"),
        (20, "1H"),
        (21, "1G"),
        (22, "1F"),
        (23, "1E"),
        (24, "1D"),
        (25, "1C"),
        (26, "1B"),
        (27, "1A"),
        (28, "19"),
        (29, "18"),
        (30, "17"),
        (31, "16"),
        (32, "15"),
        (33, "14"),
        (34, "13"),
        (35, "12"),
        (36, "11"),
    ];

    #[test]
    fn atou8_base10_test() {
        assert_eq!(Ok(0), atou8_slice(b"0"));
        assert_eq!(Ok(127), atou8_slice(b"127"));
        assert_eq!(Ok(128), atou8_slice(b"128"));
        assert_eq!(Ok(255), atou8_slice(b"255"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 0).into()), atou8_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atou8_slice(b"1a"));
    }

    #[cfg(feature = "radix")]
    #[test]
    fn atou8_basen_test() {
        for (b, s) in DATA.iter() {
            assert_eq!(atou8_radix_slice(*b, s.as_bytes()), Ok(37));
        }
    }

    #[test]
    fn atoi8_base10_test() {
        assert_eq!(Ok(0), atoi8_slice(b"0"));
        assert_eq!(Ok(127), atoi8_slice(b"127"));
        assert_eq!(Err((ErrorCode::Overflow, 2).into()), atoi8_slice(b"128"));
        assert_eq!(Err((ErrorCode::Overflow, 2).into()), atoi8_slice(b"255"));
        assert_eq!(Ok(-1), atoi8_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atoi8_slice(b"1a"));
    }

    #[cfg(feature = "radix")]
    #[test]
    fn atoi8_basen_test() {
        for (b, s) in DATA.iter() {
            assert_eq!(atoi8_radix_slice(*b, s.as_bytes()), Ok(37));
        }
    }

    #[test]
    fn atou16_base10_test() {
        assert_eq!(Ok(0), atou16_slice(b"0"));
        assert_eq!(Ok(32767), atou16_slice(b"32767"));
        assert_eq!(Ok(32768), atou16_slice(b"32768"));
        assert_eq!(Ok(65535), atou16_slice(b"65535"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 0).into()), atou16_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atou16_slice(b"1a"));
    }

    #[test]
    fn atoi16_base10_test() {
        assert_eq!(Ok(0), atoi16_slice(b"0"));
        assert_eq!(Ok(32767), atoi16_slice(b"32767"));
        assert_eq!(Err((ErrorCode::Overflow, 4).into()), atoi16_slice(b"32768"));
        assert_eq!(Err((ErrorCode::Overflow, 4).into()), atoi16_slice(b"65535"));
        assert_eq!(Ok(-1), atoi16_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atoi16_slice(b"1a"));
    }

    #[cfg(feature = "radix")]
    #[test]
    fn atoi16_basen_test() {
        for (b, s) in DATA.iter() {
            assert_eq!(atoi16_radix_slice(*b, s.as_bytes()), Ok(37));
        }
        assert_eq!(atoi16_radix_slice(36, b"YA"), Ok(1234));
    }

    #[test]
    fn atou32_base10_test() {
        assert_eq!(Ok(0), atou32_slice(b"0"));
        assert_eq!(Ok(2147483647), atou32_slice(b"2147483647"));
        assert_eq!(Ok(2147483648), atou32_slice(b"2147483648"));
        assert_eq!(Ok(4294967295), atou32_slice(b"4294967295"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 0).into()), atou32_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atou32_slice(b"1a"));
    }

    #[test]
    fn atoi32_base10_test() {
        assert_eq!(Ok(0), atoi32_slice(b"0"));
        assert_eq!(Ok(2147483647), atoi32_slice(b"2147483647"));
        assert_eq!(Err((ErrorCode::Overflow, 9).into()), atoi32_slice(b"2147483648"));
        assert_eq!(Err((ErrorCode::Overflow, 9).into()), atoi32_slice(b"4294967295"));
        assert_eq!(Ok(-1), atoi32_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atoi32_slice(b"1a"));
    }

    #[test]
    fn atou64_base10_test() {
        assert_eq!(Ok(0), atou64_slice(b"0"));
        assert_eq!(Ok(9223372036854775807), atou64_slice(b"9223372036854775807"));
        assert_eq!(Ok(9223372036854775808), atou64_slice(b"9223372036854775808"));
        assert_eq!(Ok(18446744073709551615), atou64_slice(b"18446744073709551615"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 0).into()), atou64_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atou64_slice(b"1a"));
    }

    #[test]
    fn atoi64_base10_test() {
        assert_eq!(Ok(0), atoi64_slice(b"0"));
        assert_eq!(Ok(9223372036854775807), atoi64_slice(b"9223372036854775807"));
        assert_eq!(Err((ErrorCode::Overflow, 18).into()), atoi64_slice(b"9223372036854775808"));
        assert_eq!(Err((ErrorCode::Overflow, 19).into()), atoi64_slice(b"18446744073709551615"));
        assert_eq!(Ok(-1), atoi64_slice(b"-1"));
        assert_eq!(Err((ErrorCode::InvalidDigit, 1).into()), atoi64_slice(b"1a"));

        // Add tests discovered via fuzzing.
        assert_eq!(Err((ErrorCode::Overflow, 19).into()), atoi64_slice(b"406260572150672006000066000000060060007667760000000000000000000+00000006766767766666767665670000000000000000000000666"));
    }

    proptest! {
        #[test]
        fn u8_invalid_proptest(i in r"[+]?[0-9]{2}\D") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let index = result.err().unwrap().index;
            assert!(index == 2 || index == 3);
        }

        #[test]
        fn u8_overflow_proptest(i in r"[+]?[1-9][0-9]{3}") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn u8_negative_proptest(i in r"[-][1-9][0-9]{2}") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::InvalidDigit);
        }

        #[test]
        fn u8_double_sign_proptest(i in r"[+]{2}[0-9]{2}") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn u8_sign_only_proptest(i in r"[+]") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn u8_trailing_digits_proptest(i in r"[+]?[0-9]{2}\D[0-9]{2}") {
            let result = atou8_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 2 || error.index == 3);
        }

        #[test]
        fn i8_invalid_proptest(i in r"[+-]?[0-9]{2}\D") {
            let result = atoi8_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 2 || error.index == 3);
        }

        #[test]
        fn i8_overflow_proptest(i in r"[+]?[1-9][0-9]{3}\D") {
            let result = atoi8_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn i8_underflow_proptest(i in r"[-][1-9][0-9]{3}\D") {
            let result = atoi8_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Underflow);
        }

        #[test]
        fn i8_double_sign_proptest(i in r"[+-]{2}[0-9]{2}") {
            let result = atoi8_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn i8_sign_only_proptest(i in r"[+-]") {
            let result = atoi8_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::Empty);
        }

        #[test]
        fn i8_trailing_digits_proptest(i in r"[+-]?[0-9]{2}\D[0-9]{2}") {
            let result = atoi8_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 2 || error.index == 3);
        }

        #[test]
        fn u16_invalid_proptest(i in r"[+]?[0-9]{4}\D") {
            let result = atou16_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 4 || error.index == 5);
        }

        #[test]
        fn u16_overflow_proptest(i in r"[+]?[1-9][0-9]{5}\D") {
            let result = atou16_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn u16_negative_proptest(i in r"[-][1-9][0-9]{4}") {
            let result = atou16_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::InvalidDigit);
        }

        #[test]
        fn u16_double_sign_proptest(i in r"[+]{2}[0-9]{4}") {
            let result = atou16_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn u16_sign_only_proptest(i in r"[+]") {
            let result = atou16_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn u16_trailing_digits_proptest(i in r"[+]?[0-9]{4}\D[0-9]{2}") {
            let result = atou16_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 4 || error.index == 5);
        }

        #[test]
        fn i16_invalid_proptest(i in r"[+-]?[0-9]{4}\D") {
            let result = atoi16_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 4 || error.index == 5);
        }

        #[test]
        fn i16_overflow_proptest(i in r"[+]?[1-9][0-9]{5}\D") {
            let result = atoi16_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn i16_underflow_proptest(i in r"[-][1-9][0-9]{5}\DD") {
            let result = atoi16_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Underflow);
        }

        #[test]
        fn i16_double_sign_proptest(i in r"[+-]{2}[0-9]{4}") {
            let result = atoi16_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn i16_sign_only_proptest(i in r"[+-]") {
            let result = atoi16_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn i16_trailing_digits_proptest(i in r"[+-]?[0-9]{4}\D[0-9]{2}") {
            let result = atoi16_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 4 || error.index == 5);
        }

        #[test]
        fn u32_invalid_proptest(i in r"[+]?[0-9]{9}\D") {
            let result = atou32_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 9 || error.index == 10);
        }

        #[test]
        fn u32_overflow_proptest(i in r"[+]?[1-9][0-9]{10}\D") {
            let result = atou32_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn u32_negative_proptest(i in r"[-][1-9][0-9]{9}") {
            let result = atou32_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::InvalidDigit);
        }

        #[test]
        fn u32_double_sign_proptest(i in r"[+]{2}[0-9]{9}") {
            let result = atou32_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn u32_sign_only_proptest(i in r"[+]") {
            let result = atou32_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn u32_trailing_digits_proptest(i in r"[+]?[0-9]{9}\D[0-9]{2}") {
            let result = atou32_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 9 || error.index == 10);
        }

        #[test]
        fn i32_invalid_proptest(i in r"[+-]?[0-9]{9}\D") {
            let result = atoi32_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 9 || error.index == 10);
        }

        #[test]
        fn i32_overflow_proptest(i in r"[+]?[1-9][0-9]{10}\D") {
            let result = atoi32_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn i32_underflow_proptest(i in r"-[1-9][0-9]{10}\D") {
            let result = atoi32_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Underflow);
        }

        #[test]
        fn i32_double_sign_proptest(i in r"[+-]{2}[0-9]{9}") {
            let result = atoi32_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn i32_sign_only_proptest(i in r"[+-]") {
            let result = atoi32_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn i32_trailing_digits_proptest(i in r"[+-]?[0-9]{9}\D[0-9]{2}") {
            let result = atoi32_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 9 || error.index == 10);
        }

        #[test]
        fn u64_invalid_proptest(i in r"[+]?[0-9]{19}\D") {
            let result = atou64_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 19 || error.index == 20);
        }

        #[test]
        fn u64_overflow_proptest(i in r"[+]?[1-9][0-9]{21}\D") {
            let result = atou64_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn u64_negative_proptest(i in r"[-][1-9][0-9]{21}") {
            let result = atou64_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::InvalidDigit);
        }

        #[test]
        fn u64_double_sign_proptest(i in r"[+]{2}[0-9]{19}") {
            let result = atou64_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn u64_sign_only_proptest(i in r"[+]") {
            let result = atou64_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn u64_trailing_digits_proptest(i in r"[+]?[0-9]{19}\D[0-9]{2}") {
            let result = atou64_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 19 || error.index == 20);
        }

        #[test]
        fn i64_invalid_proptest(i in r"[+-]?[0-9]{18}\D") {
            let result = atoi64_slice(i.as_bytes());
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 18 || error.index == 19);
        }

        #[test]
        fn i64_overflow_proptest(i in r"[+]?[1-9][0-9]{19}\D") {
            let result = atoi64_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Overflow);
        }

        #[test]
        fn i64_underflow_proptest(i in r"-[1-9][0-9]{19}\D") {
            let result = atoi64_slice(i.as_bytes());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Underflow);
        }

        #[test]
        fn i64_double_sign_proptest(i in r"[+-]{2}[0-9]{18}") {
            let result = atoi64_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 1);
        }

        #[test]
        fn i64_sign_only_proptest(i in r"[+-]") {
            let result = atoi32_slice(i.as_bytes());
            assert!(result.is_err());
            let code = result.err().unwrap().code;
            assert_eq!(code, ErrorCode::Empty);
        }

        #[test]
        fn i64_trailing_digits_proptest(i in r"[+-]?[0-9]{18}\D[0-9]{2}") {
            let result = atoi64_slice(i.as_bytes());
            let error = result.err().unwrap();
            assert_eq!(error.code, ErrorCode::InvalidDigit);
            assert!(error.index == 18 || error.index == 19);
        }
    }
}
