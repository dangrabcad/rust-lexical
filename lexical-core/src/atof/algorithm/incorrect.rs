//! Incorrect, fast algorithms for string-to-float conversions.

use atoi;
use util::*;
use super::state::RawFloatState;

// FRACTION

type Wrapped<F> = WrappedFloat<F>;

/// Process the integer component of the raw float.
#[inline]
fn process_integer<F: StablePower>(radix: u32, state: &RawFloatState)
    -> F
{
    match state.integer.len() {
        0 => F::ZERO,
        _ => atoi::standalone::<Wrapped<F>>(radix, state.integer, true)
                .expect("Should not overflow nor error.")
                .into_inner()
    }
}

/// Process the fraction component of the raw float.
#[inline]
fn process_fraction<F: StablePower>(radix: u32, state: &RawFloatState)
    -> F
{
    match state.fraction.len() {
        0 => F::ZERO,
        _ => {
            // We don't really care about numerical precision, so just break
            // the fraction into 12-digit pieces.
            // 12 is the maximum number of digits we can use without
            // potentially overflowing  a 36-radix float string.
            let mut fraction = F::ZERO;
            let mut digits: i32 = 0;
            for chunk in state.fraction.chunks(12) {
                digits = digits.saturating_add(chunk.len().as_i32());
                let value: u64 = atoi::standalone(radix, chunk, false)
                    .expect("Should not overflow nor error.");
                if !value.is_zero() {
                    fraction += F::iterative_pow(as_cast(value), radix, -digits);
                }
            }
            fraction
        },
    }
}

/// Convert the float string to a native floating-point number.
#[inline]
fn to_native<F: StablePower>(radix: u32, bytes: &[u8], offset: usize)
    -> Result<F>
{
    let state = RawFloatState::parse(radix, bytes, offset)?;
    let integer: F = process_integer(radix, &state);
    let fraction: F = process_fraction(radix, &state);
    let mut value = integer + fraction;
    let exponent = state.raw_exponent(radix);
    if !exponent.is_zero() && !value.is_zero() {
        value = value.iterative_pow(radix, exponent);
    }
    Ok(value)
}

// ATOF/ATOD
// ---------

/// Parse 32-bit float from string.
#[inline]
pub(crate) fn atof<'a>(radix: u32, bytes: &'a [u8], _: Sign, offset: usize)
    -> Result<f32>
{
    to_native::<f32>(radix, bytes, offset)
}

/// Parse 64-bit float from string.
#[inline]
pub(crate) fn atod<'a>(radix: u32, bytes: &'a [u8], _: Sign, offset: usize)
    -> Result<f64>
{
    to_native::<f64>(radix, bytes, offset)
}

/// Parse 32-bit float from string.
#[inline]
pub(crate) fn atof_lossy<'a>(radix: u32, bytes: &'a [u8], _: Sign, offset: usize)
    -> Result<f32>
{
    to_native::<f32>(radix, bytes, offset)
}

/// Parse 64-bit float from string.
#[inline]
pub(crate) fn atod_lossy<'a>(radix: u32, bytes: &'a [u8], _: Sign, offset: usize)
    -> Result<f64>
{
    to_native::<f64>(radix, bytes, offset)
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

    #[test]
    fn process_integer_test() {
        assert_eq!(1.0, process_integer::<f64>(10, &new_state(b"1", b"2345", b"")));
        assert_eq!(12.0, process_integer::<f64>(10, &new_state(b"12", b"345", b"")));
        assert_eq!(12345.0, process_integer::<f64>(10, &new_state(b"12345", b"6789", b"")));
    }

    #[test]
    fn process_fraction_test() {
        assert_eq!(0.2345, process_fraction::<f64>(10, &new_state(b"1", b"2345", b"")));
        assert_eq!(0.345, process_fraction::<f64>(10, &new_state(b"12", b"345", b"")));
        assert_eq!(0.6789, process_fraction::<f64>(10, &new_state(b"12345", b"6789", b"")));
    }

    #[test]
    fn atof_test() {
        let atof10 = move |x| atof(10, x, Sign::Positive, 0);

        assert_eq!(Ok(1.2345), atof10(b"1.2345"));
        assert_eq!(Ok(12.345), atof10(b"12.345"));
        assert_eq!(Ok(12345.6789), atof10(b"12345.6789"));
        assert_f32_eq!(1.2345e10, atof10(b"1.2345e10").unwrap());
    }

    #[test]
    fn atod_test() {
        let atod10 = move |x| atod(10, x, Sign::Positive, 0);

        assert_eq!(Ok(1.2345), atod10(b"1.2345"));
        assert_eq!(Ok(12.345), atod10(b"12.345"));
        assert_eq!(Ok(12345.6789), atod10(b"12345.6789"));
        assert_f64_eq!(1.2345e10, atod10(b"1.2345e10").unwrap());
    }

    // Lossy
    // Just a synonym for the regular overloads, since we're not using the
    // correct feature. Use the same tests.

    #[test]
    fn atof_lossy_test() {
        let atof10 = move |x| atof_lossy(10, x, Sign::Positive, 0);

        assert_eq!(Ok(1.2345), atof10(b"1.2345"));
        assert_eq!(Ok(12.345), atof10(b"12.345"));
        assert_eq!(Ok(12345.6789), atof10(b"12345.6789"));
        assert_f32_eq!(1.2345e10, atof10(b"1.2345e10").unwrap());
    }

    #[test]
    fn atod_lossy_test() {
        let atod10 = move |x| atod_lossy(10, x, Sign::Positive, 0);

        assert_eq!(Ok(1.2345), atod10(b"1.2345"));
        assert_eq!(Ok(12.345), atod10(b"12.345"));
        assert_eq!(Ok(12345.6789), atod10(b"12345.6789"));
        assert_f64_eq!(1.2345e10, atod10(b"1.2345e10").unwrap());
    }
}
