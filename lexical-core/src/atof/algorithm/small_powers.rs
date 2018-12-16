//! Precalculated small powers.

use super::math::Limb;

#[cfg(target_pointer_width = "16")]
use super::small_powers_16::*;

#[cfg(target_pointer_width = "32")]
use super::small_powers_32::*;

#[cfg(target_pointer_width = "64")]
use super::small_powers_64::*;

// ASSERTIONS

const_assert!(small_powers_radix5; POW5[1] / POW5[0] == 5);
const_assert!(small_powers_radix10; POW10[1] / POW10[0] == 10);

cfg_if! {
if #[cfg(feature = "radix")] {
// Ensure our small powers are valid.
const_assert!(small_powers_radix2; POW2[1] / POW2[0] == 2);
const_assert!(small_powers_radix3; POW3[1] / POW3[0] == 3);
const_assert!(small_powers_radix4; POW4[1] / POW4[0] == 4);
const_assert!(small_powers_radix6; POW6[1] / POW6[0] == 6);
const_assert!(small_powers_radix7; POW7[1] / POW7[0] == 7);
const_assert!(small_powers_radix8; POW8[1] / POW8[0] == 8);
const_assert!(small_powers_radix9; POW9[1] / POW9[0] == 9);
const_assert!(small_powers_radix11; POW11[1] / POW11[0] == 11);
const_assert!(small_powers_radix12; POW12[1] / POW12[0] == 12);
const_assert!(small_powers_radix13; POW13[1] / POW13[0] == 13);
const_assert!(small_powers_radix14; POW14[1] / POW14[0] == 14);
const_assert!(small_powers_radix15; POW15[1] / POW15[0] == 15);
const_assert!(small_powers_radix16; POW16[1] / POW16[0] == 16);
const_assert!(small_powers_radix17; POW17[1] / POW17[0] == 17);
const_assert!(small_powers_radix18; POW18[1] / POW18[0] == 18);
const_assert!(small_powers_radix19; POW19[1] / POW19[0] == 19);
const_assert!(small_powers_radix20; POW20[1] / POW20[0] == 20);
const_assert!(small_powers_radix21; POW21[1] / POW21[0] == 21);
const_assert!(small_powers_radix22; POW22[1] / POW22[0] == 22);
const_assert!(small_powers_radix23; POW23[1] / POW23[0] == 23);
const_assert!(small_powers_radix24; POW24[1] / POW24[0] == 24);
const_assert!(small_powers_radix25; POW25[1] / POW25[0] == 25);
const_assert!(small_powers_radix26; POW26[1] / POW26[0] == 26);
const_assert!(small_powers_radix27; POW27[1] / POW27[0] == 27);
const_assert!(small_powers_radix28; POW28[1] / POW28[0] == 28);
const_assert!(small_powers_radix29; POW29[1] / POW29[0] == 29);
const_assert!(small_powers_radix30; POW30[1] / POW30[0] == 30);
const_assert!(small_powers_radix31; POW31[1] / POW31[0] == 31);
const_assert!(small_powers_radix32; POW32[1] / POW32[0] == 32);
const_assert!(small_powers_radix33; POW33[1] / POW33[0] == 33);
const_assert!(small_powers_radix34; POW34[1] / POW34[0] == 34);
const_assert!(small_powers_radix35; POW35[1] / POW35[0] == 35);
const_assert!(small_powers_radix36; POW36[1] / POW36[0] == 36);

}}  //cfg_if

// HELPER

/// Get the correct small power from the radix.
pub(in atof::algorithm) fn get_small_powers(radix: u32)
    -> &'static [Limb]
{
    #[cfg(not(feature = "radix"))] {
        match radix {
            5  => &POW5,
            10 => &POW10,
            _  => unreachable!()
        }
    }

    #[cfg(feature = "radix")] {
        match radix {
            2  => &POW2,
            3  => &POW3,
            4  => &POW4,
            5  => &POW5,
            6  => &POW6,
            7  => &POW7,
            8  => &POW8,
            9  => &POW9,
            10  => &POW10,
            11  => &POW11,
            12  => &POW12,
            13  => &POW13,
            14  => &POW14,
            15  => &POW15,
            16  => &POW16,
            17  => &POW17,
            18  => &POW18,
            19  => &POW19,
            20  => &POW20,
            21  => &POW21,
            22  => &POW22,
            23  => &POW23,
            24  => &POW24,
            25  => &POW25,
            26  => &POW26,
            27  => &POW27,
            28  => &POW28,
            29  => &POW29,
            30  => &POW30,
            31  => &POW31,
            32  => &POW32,
            33  => &POW33,
            34  => &POW34,
            35  => &POW35,
            36  => &POW36,
            _  => unreachable!(),
        }
    }
}