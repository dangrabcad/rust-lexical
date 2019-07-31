//! Stores the current state of the parsed float.

use atoi;
use util::*;

cfg_if! {
if #[cfg(feature = "correct")] {
use super::alias::*;
use super::exponent::*;
}}  // cfg_if

// PARSE
// -----

// Left-trim leading 0s.
macro_rules! ltrim_0 {
    ($bytes:expr) => { ltrim_char_slice($bytes, b'0') };
}

// Right-trim leading 0s.
macro_rules! rtrim_0 {
    ($bytes:expr) => { rtrim_char_slice($bytes, b'0') };
}

// Export a character to digit.
macro_rules! to_digit {
    ($c:expr, $radix:ident) => (($c as char).to_digit($radix));
}

/// Consume digits from the input byte array.
#[allow(unused_variables)]
#[inline]
fn consume_digits(radix: u32, bytes: &[u8]) -> usize
{
    #[cfg(feature = "radix")]
    return bytes.iter()
        .take_while(|&&c| to_digit!(c, radix).is_some())
        .count();

    #[cfg(not(feature = "radix"))]
    return bytes.iter()
        .take_while(|&&c| b'0' <= c && c <= b'9')
        .count();
}

/// Parse the integer component.
macro_rules! parse_integer {
    ($radix:ident, $bytes:ident, $state:ident) => {
        let integer_count = consume_digits($radix, $bytes);
        $state.integer = &index!($bytes[..integer_count]);
        $bytes = &index!($bytes[integer_count..]);
    };
}

/// Parse the fraction component.
macro_rules! parse_fraction {
    ($radix:ident, $bytes:ident, $state:ident) => {
        $bytes = &index!($bytes[1..]);
        let fraction_count = consume_digits($radix, $bytes);
        $state.fraction = &index!($bytes[..fraction_count]);
        $bytes = &index!($bytes[fraction_count..]);
    };
}

/// Parse the exponent component.
macro_rules! parse_exponent {
    ($radix:ident, $bytes:ident, $state:ident) => {
        if let Some(&c) = $bytes.get(0) {
            if case_insensitive_equal(c, exponent_notation_char($radix)) {
                // Have an exponent, need to parse.
                let digits = &index!($bytes[1..]);

                // Filter out empty inputs.
                if digits.is_empty() {
                    return Err(ErrorCode::EmptyExponent.into());
                }

                // Check if there's a sign and the raw digits.
                let (sign_bytes, raw_digits) = match &index!(digits[0]) {
                    b'+' | b'-' => (1, &index!(digits[1..])),
                    _    => (0, digits),
                };

                // Filter out empty inputs.
                if raw_digits.is_empty() {
                    return Err(ErrorCode::EmptyExponent.into());
                }

                // Parse the exponent digits.
                let exponent_count = sign_bytes + consume_digits($radix, raw_digits);
                $state.exponent = &index!(digits[..exponent_count]);
                $bytes = &index!(digits[exponent_count..]);
            }
        }
    };
}

// RAW FLOAT STATE
// ---------------

/// Raw substring and information from parsing the float.
#[derive(Debug)]
pub(super) struct RawFloatState<'a> {
    /// Substring for the integer component of the mantissa.
    pub(super) integer: &'a [u8],
    /// Substring for the fraction component of the mantissa.
    pub(super) fraction: &'a [u8],
    /// Substring for the exponent component.
    pub(super) exponent: &'a [u8],
}

impl<'a> RawFloatState<'a> {
    /// Create new raw float state.
    pub(super) fn new() -> RawFloatState<'a> {
        RawFloatState {
            integer: &[],
            fraction: &[],
            exponent: &[],
        }
    }

    /// Parse the float state from raw bytes.
    pub(super) fn parse(radix: u32, bytes: &'a [u8], offset: usize)
        -> Result<RawFloatState<'a>>
    {
        // Initialize variables.
        let mut state: RawFloatState = RawFloatState::new();
        let mut digits = bytes;

        // Cannot be empty due to a precondition in the API dispatcher.
        // We need to process the float and the parse the subcomponents.
        debug_assert!(!digits.is_empty());
        let digit = digits[0];
        if digit == b'.' {
            // Leading period, parse fraction without integer component
            parse_fraction!(radix, digits, state);
            if state.fraction.len() == 0 {
                // Invalid floating-point number, no integer or fraction components.
                return Err(ErrorCode::EmptyFraction.into());
            }
            parse_exponent!(radix, digits, state);
        } else {
            if to_digit!(digit, radix).is_some() {
                // Leading digit, parse fraction with integer component.
                parse_integer!(radix, digits, state);
                if let Some(b'.') = digits.get(0) {
                    parse_fraction!(radix, digits, state);
                }
                parse_exponent!(radix, digits, state);
            } else {
                // Invalid floating point number.
                return Err((ErrorCode::InvalidDigit, offset).into());
            }
        };

        match digits.is_empty() {
            true  => {
                // Do our post-processing on the digits the create a pretty float.
                // This is required for accurate results in the slow-path algorithm,
                // otherwise, we may incorrect guess the mantissa or scientific
                // exponent.
                state.integer = ltrim_0!(state.integer).0;
                state.fraction = rtrim_0!(state.fraction).0;
                Ok(state)
            },
            false => {
                let index = bytes.len() - digits.len();
                Err((ErrorCode::InvalidDigit, offset + index).into())
            },
        }
    }

    /// Parse the raw float state into an exponent.
    pub(super) fn raw_exponent(&self, radix: u32) -> i32 {
        match self.exponent.len() {
            // No exponent, we good here.
            0 => 0,
            // Have an exponent, parse it.
            _ => match atoi::standalone_signed::<i32>(radix, self.exponent) {
                Ok(v)                     => v,
                Err(e)                    => {
                    match e.code {
                        ErrorCode::Overflow  => i32::max_value(),
                        ErrorCode::Underflow => i32::min_value(),
                        _                    => unreachable!(),
                    }
                },
            },
        }
    }

    /// Process the float state for the moderate or slow atof processor.
    #[cfg(feature = "correct")]
    pub(super) fn process(self, truncated: usize, raw_exponent: i32) -> FloatState<'a> {
        let integer = self.integer;
        let fraction = self.fraction;
        let digits_start = match integer.len() {
            0 => ltrim_char_slice(fraction, b'0').1,
            _ => 0,
        };
        FloatState { integer, fraction, digits_start, truncated, raw_exponent }
    }
}

// FLOAT STATE
// -----------

/// Substrings and information from parsing the float.
#[cfg(feature = "correct")]
#[derive(Debug)]
pub(super) struct FloatState<'a> {
    /// Substring for the integer component of the mantissa.
    pub(super) integer: &'a [u8],
    /// Substring for the fraction component of the mantissa.
    pub(super) fraction: &'a [u8],
    /// Offset to where the digits start in either integer or fraction.
    pub(super) digits_start: usize,
    /// Number of truncated digits from the mantissa.
    pub(super) truncated: usize,
    /// Raw exponent for the float.
    pub(super) raw_exponent: i32,
}

#[cfg(feature = "correct")]
impl<'a> FloatState<'a> {
    /// Get the length of the integer substring.
    #[inline]
    pub(super) fn integer_len(&self) -> usize {
        self.integer.len()
    }

    /// Get number of parsed integer digits.
    #[inline]
    pub(super) fn integer_digits(&self) -> usize {
        self.integer_len()
    }

    /// Iterate over the integer digits.
    #[inline]
    pub(super) fn integer_iter(&self) -> SliceIter<u8> {
        self.integer.iter()
    }

    /// Get the length of the fraction substring.
    #[inline]
    pub(super) fn fraction_len(&self) -> usize {
        self.fraction.len()
    }

    /// Iterate over the fraction digits.
    #[inline]
    pub(super) fn fraction_digits(&self) -> usize {
        self.fraction_len() - self.digits_start
    }

    /// Iterate over the digits, by chaining two slices.
    #[inline]
    pub(super) fn fraction_iter(&self) -> SliceIter<u8> {
        // We need to rtrim the zeros in the slice fraction.
        // These are useless and just add computational complexity later,
        // just like leading zeros in the integer.
        // We need them to calculate the number of truncated bytes,
        // but we should remove them before doing anything costly.
        // In practice, we only call `mantissa_iter()` once per parse,
        // so this is effectively free.
        self.fraction[self.digits_start..].iter()
    }

    /// Get the number of digits in the mantissa.
    /// Cannot overflow, since this is based off a single usize input string.
    #[inline]
    pub(super) fn mantissa_digits(&self) -> usize {
        self.integer_digits() + self.fraction_digits()
    }

    /// Iterate over the mantissa digits, by chaining two slices.
    #[inline]
    pub(super) fn mantissa_iter(&self) -> ChainedSliceIter<u8> {
        self.integer_iter().chain(self.fraction_iter())
    }

    /// Get number of truncated digits.
    #[inline]
    pub(super) fn truncated_digits(&self) -> usize {
        self.truncated
    }

    /// Get the mantissa exponent from the raw exponent.
    #[inline]
    pub(super) fn mantissa_exponent(&self) -> i32 {
        mantissa_exponent(self.raw_exponent, self.fraction_len(), self.truncated_digits())
    }

    /// Get the scientific exponent from the raw exponent.
    #[inline]
    pub(super) fn scientific_exponent(&self) -> i32 {
        scientific_exponent(self.raw_exponent, self.integer_digits(), self.digits_start)
    }
}

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    fn new_state<'a>(integer: &'a [u8], fraction: &'a [u8], exponent: &'a [u8])
        -> RawFloatState<'a>
    {
        RawFloatState { integer, fraction, exponent }
    }

    fn check_parse(radix: u32, digits: &str, expected: Result<RawFloatState>)
    {
        match RawFloatState::parse(radix, digits.as_bytes(), 0) {
            Ok(v)  => {
                let expected = expected.unwrap();
                assert_eq!(v.integer, expected.integer);
                assert_eq!(v.fraction, expected.fraction);
                assert_eq!(v.exponent, expected.exponent);
            },
            Err(e) => assert_eq!(e, expected.err().unwrap()),
        }
    }

    #[test]
    fn parse_test() {
        // Valid
        check_parse(10, "1.2345", Ok(new_state(b"1", b"2345", b"")));
        check_parse(10, "12.345", Ok(new_state(b"12", b"345", b"")));
        check_parse(10, "12345.6789", Ok(new_state(b"12345", b"6789", b"")));
        check_parse(10, "1.2345e10", Ok(new_state(b"1", b"2345", b"10")));
        check_parse(10, "1.2345e+10", Ok(new_state(b"1", b"2345", b"+10")));
        check_parse(10, "1.2345e-10", Ok(new_state(b"1", b"2345", b"-10")));
        check_parse(10, "100000000000000000000", Ok(new_state(b"100000000000000000000", b"", b"")));
        check_parse(10, "100000000000000000001", Ok(new_state(b"100000000000000000001", b"", b"")));
        check_parse(10, "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791.9999999999999999999999999999999999999999999999999999999999999999999999", Ok(new_state(b"179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791", b"9999999999999999999999999999999999999999999999999999999999999999999999", b"")));
        check_parse(10, "1009e-31", Ok(new_state(b"1009", b"", b"-31")));
        check_parse(10, "001.0", Ok(new_state(b"1", b"", b"")));

        // Invalid
        check_parse(10, "1.2345/", Err((ErrorCode::InvalidDigit, 6).into()));
        check_parse(10, "1.2345e", Err(ErrorCode::EmptyExponent.into()));
        check_parse(10, ".", Err(ErrorCode::EmptyFraction.into()));

        // Invalid exponent character
        check_parse(10, "1^45", Err((ErrorCode::InvalidDigit, 1).into()));

        // Trailing characters
        check_parse(10, "1e45 ", Err((ErrorCode::InvalidDigit, 4).into()));
        check_parse(10, "1e45-", Err((ErrorCode::InvalidDigit, 4).into()));
        check_parse(10, "1e45+", Err((ErrorCode::InvalidDigit, 4).into()));
        check_parse(10, "1e45a", Err((ErrorCode::InvalidDigit, 4).into()));
    }

    #[test]
    fn raw_exponent_test() {
        assert_eq!(0, new_state(b"1", b"2345", b"").raw_exponent(10));
        assert_eq!(0, new_state(b"1", b"2345", b"0").raw_exponent(10));
        assert_eq!(0, new_state(b"1", b"2345", b"+0").raw_exponent(10));
        assert_eq!(0, new_state(b"1", b"2345", b"-0").raw_exponent(10));
        assert_eq!(5, new_state(b"1", b"2345", b"5").raw_exponent(10));
        assert_eq!(123, new_state(b"1", b"2345", b"+123").raw_exponent(10));
        assert_eq!(i32::max_value(), new_state(b"1", b"2345", b"4294967296").raw_exponent(10));
        assert_eq!(i32::max_value(), new_state(b"1", b"2345", b"+4294967296").raw_exponent(10));
        assert_eq!(i32::min_value(), new_state(b"1", b"2345", b"-4294967296").raw_exponent(10));
    }

    #[cfg(feature = "correct")]
    #[test]
    fn scientific_exponent_test() {
        // Check "1.2345", simple.
        let state = FloatState {
            integer: "1".as_bytes(),
            fraction: "2345".as_bytes(),
            digits_start: 0,
            truncated: 0,
            raw_exponent: 0,
        };
        assert_eq!(state.scientific_exponent(), 0);

        // Check "0.12345", simple.
        let state = FloatState {
            integer: "".as_bytes(),
            fraction: "12345".as_bytes(),
            digits_start: 0,
            truncated: 0,
            raw_exponent: 0,
        };
        assert_eq!(state.scientific_exponent(), -1);
    }
}
