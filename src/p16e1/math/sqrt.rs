use super::P16E1;

impl P16E1 {
    pub const fn sqrt(self) -> Self {
        let mut ui_a = self.to_bits();

        // If sign bit is set, return NaR.
        if (ui_a & 0x_8000) != 0 {
            return Self::NAR;
        }
        // If the argument is zero, return zero.
        if ui_a == 0 {
            return Self::ZERO;
        }
        // Compute the square root. Here, k_z is the net power-of-2 scaling of the result.
        // Decode the regime and exponent bit; scale the input to be in the range 1 to 4:
        let mut k_z: i16;
        if (ui_a >> 14) != 0 {
            k_z = -1;
            while (ui_a & 0x4000) != 0 {
                k_z += 1;
                ui_a <<= 1;
            }
        } else {
            k_z = 0;
            while (ui_a & 0x4000) == 0 {
                k_z -= 1;
                ui_a <<= 1;
            }
        }
        ui_a &= 0x3fff;
        let exp_a = 1 - (ui_a >> 13);
        let frac_a = (ui_a | 0x2000) >> 1;

        // Use table look-up of first four bits for piecewise linear approx. of 1/sqrt:
        let index = (((frac_a >> 8) & 0xE) + exp_a) as usize;

        let r0 = (crate::APPROX_RECIP_SQRT0[index] as u32
            - (((crate::APPROX_RECIP_SQRT1[index] as u32) * ((frac_a & 0x1FF) as u32)) >> 13))
            as u16 as u32;
        // Use Newton-Raphson refinement to get more accuracy for 1/sqrt:
        let mut e_sqr_r0 = (r0 * r0) >> 1;

        if exp_a != 0 {
            e_sqr_r0 >>= 1;
        }
        let sigma0 = 0xFFFF ^ ((0xFFFF & (((e_sqr_r0 as u64) * (frac_a as u64)) >> 18)) as u16); //~(u16) ((e_sqr_r0 * frac_a) >> 18);
        let recip_sqrt = (r0 << 2) + ((r0 * (sigma0 as u32)) >> 23);

        // We need 17 bits of accuracy for posit16 square root approximation.
        // Multiplying 16 bits and 18 bits needs 64-bit scratch before the right shift:
        let mut frac_z = (((frac_a as u64) * (recip_sqrt as u64)) >> 13) as u32;

        // Figure out the regime and the resulting right shift of the fraction:
        let shift: u16;
        let mut ui_z: u16 = if k_z < 0 {
            shift = ((-1 - k_z) >> 1) as u16;
            0x2000 >> shift
        } else {
            shift = (k_z >> 1) as u16;
            0x7fff - (0x7FFF >> (shift + 1))
        };
        // Set the exponent bit in the answer, if it is nonzero:
        if (k_z & 1) != 0 {
            ui_z |= 0x1000 >> shift;
        }

        // Right-shift fraction bits, accounting for 1 <= a < 2 versus 2 <= a < 4:
        frac_z >>= exp_a + shift;

        // Trick for eliminating off-by-one cases that only uses one multiply:
        frac_z += 1;
        if (frac_z & 7) == 0 {
            let shifted_frac_z = frac_z >> 1;
            let neg_rem = (shifted_frac_z * shifted_frac_z) & 0x3_FFFF;
            if (neg_rem & 0x2_0000) != 0 {
                frac_z |= 1;
            } else if neg_rem != 0 {
                frac_z -= 1;
            }
        }
        // Strip off the hidden bit and round-to-nearest using last 4 bits.
        frac_z -= 0x1_0000 >> shift;
        let bit_n_plus_one = ((frac_z >> 3) & 1) != 0;
        if bit_n_plus_one && ((((frac_z >> 4) & 1) | (frac_z & 7)) != 0) {
            frac_z += 0x10;
        }
        // Assemble the result and return it.
        Self::from_bits(ui_z | ((frac_z >> 4) as u16))
    }
}

#[test]
fn test_sqrt() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let p = p_a.sqrt();
        let f = f_a.sqrt();
        assert_eq!(p, P16E1::from(f));
    }
}
