pub(crate) struct U16;
pub(crate) struct U32;
pub(crate) struct U64;

trait BitRound<U>: Sized {
    fn bitround(value: U) -> Self;
}
impl<T> BitRound<Self> for T {
    fn bitround(value: Self) -> Self {
        value
    }
}

/// Bitround an unsigned integer `ui` to another bitsize `UIntN1`.
/// Rounds/downcasts using round to nearest or upcasts (append with zeros).
macro_rules! impl_bitround {
    ($Ux:ty, $ux:ty, ($($to:ident),+)) => {
        impl $Ux {
            pub const fn bitround<const BITS: u32>(mut ui: $ux) -> u64 {
                let d_bits: u32 = <$ux>::BITS - BITS; // difference in bits

                // ROUND TO NEAREST, tie to even: create ulp/2 = ..007ff.. or ..0080..
                let mut ulp_half = (<$ux>::MAX >> 1) >> BITS; // create ..007ff.. (just smaller than ulp/2)
                ulp_half += (ui >> d_bits) & 0x1; // turn into ..0080.. for odd (=round up if tie)
                ui += ulp_half; // +ulp/2 and
                ui as u64 >> d_bits // round down via >> is round nearest
            }
        }
        $(
            impl BitRound<$ux> for $to {
                fn bitround(value: $ux) -> $to {
                    <$Ux>::bitround::<{<$to>::BITS}>(value) as _
                }
            }
        )+
    };
}

impl_bitround!(U16, u16, (u8));
impl_bitround!(U32, u32, (u8, u16));
impl_bitround!(U64, u64, (u8, u16, u32));

#[test]
/// Bitround round to nearest, tie to even
fn test_bitround() {
    const N: u32 = 100_000;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // u16 -> u8
    for _ in 0..N {
        let ui8: u8 = rng.gen();

        // pad with zeros
        let ui16 = (ui8 as u16) << 8;
        assert_eq!(u8::bitround(ui16), ui8);

        // pad with ones
        // let ui16_ones = ui16 | 0x00ff;
        // assert_eq!(u8::bitround(ui16_ones), ui8 + 0x1);

        // just less than ulp (all round down)
        let ui16_rd = ui16 | 0x007f;
        assert_eq!(u8::bitround(ui16_rd), ui8);

        // just more than ulp (all round up)
        //let ui16_ru = ui16 | 0x0081;
        //assert_eq!(u8::bitround(ui16_ru), ui8 + 0x1);

        // tie to even
        //let ui16_tie = ui16 | 0x0080;
        //assert_eq!(u8::bitround(ui16_tie), ui8 + (ui8 & 0x1));
    }

    // u32 -> u16
    for _ in 0..N {
        let ui16: u16 = rng.gen();

        // pad with zeros
        let ui32 = (ui16 as u32) << 16;
        assert_eq!(u16::bitround(ui32), ui16);

        // pad with ones
        // let ui32_ones = ui32 | 0x0000_ffff;
        // assert_eq!(u16::bitround(ui32_ones), ui16 + 0x1);

        // just less than ulp (all round down)
        let ui32_rd = ui32 | 0x0000_7fff;
        assert_eq!(u16::bitround(ui32_rd), ui16);

        // just more than ulp (all round up)
        //    let ui32_ru = ui32 | 0x0000_8001;
        //    assert_eq!(u16::bitround(ui32_ru), ui16 + 0x1);

        // tie to even
        //    let ui32_tie = ui32 | 0x0000_8000;
        //    assert_eq!(u16::bitround(ui32_tie), ui16 + (ui16 & 0x1));
    }
    /*
    # UInt64 -> UInt32
    for ui32 in rand(UInt32, N)
        ui64 = (ui32 % UInt64) << 32    # pad with zeros
        @test SoftPosit.bitround(UInt32, ui64) == ui32

        ui64_ones = ui64 | 0x0000_0000_ffff_ffff  # pad with ones
        @test SoftPosit.bitround(UInt32, ui64_ones) == ui32 + 0x1

        ui64_rd = ui64 | 0x0000_0000_7fff_ffff    # just less than ulp (all round down)
        @test SoftPosit.bitround(UInt32, ui64_rd) == ui32

        ui64_ru = ui64 | 0x0000_0000_8000_0001    # just more than ulp (all round up)
        @test SoftPosit.bitround(UInt32, ui64_ru) == ui32 + 0x1

        ui64_tie = ui64 | 0x0000_8000_0000        # tie to even
        @test SoftPosit.bitround(UInt32, ui64_tie) == ui32 + (ui32 & 0x1)
    end

    # UInt32 -> UInt8
    for ui8 in rand(UInt8, N)
        ui32 = (ui8 % UInt32) << 24     # pad with zeros
        @test SoftPosit.bitround(UInt8, ui32) == ui8

        ui32_ones = ui32 | 0x00ff_ffff  # pad with ones
        @test SoftPosit.bitround(UInt8, ui32_ones) == ui8 + 0x1

        ui32_rd = ui32 | 0x007f_ffff    # just less than ulp (all round down)
        @test SoftPosit.bitround(UInt8, ui32_rd) == ui8

        ui32_ru = ui32 | 0x0080_0001    # just more than ulp (all round up)
        @test SoftPosit.bitround(UInt8, ui32_ru) == ui8 + 0x1

        ui32_tie = ui32 | 0x0080_0000   # tie to even
        @test SoftPosit.bitround(UInt8, ui32_tie) == ui8 + (ui8 & 0x1)
    end
     */
}
