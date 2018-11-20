use super::*;
use crate::{MulAddType, WithSign};

impl P32E2 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn round(self) -> Self {
        round(self)
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        sqrt(self)
    }
}

#[allow(unused_assignments)]
fn mul_add(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> P32E2 {
    let mut u_z: u32;
    let mut bit_n_plus_one = false;
    let mut bits_more = false;
    let mut k_z = 0_i16;
    let mut exp_z: i32;
    let mut frac_z: u32 = 0; // possibly uninitialized
                             //NaR
    if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
        return P32E2::from_bits(0x8000_0000);
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P32E2::from_bits(ui_c.wrapping_neg()),
            _ => P32E2::from_bits(ui_c),
        };
    }

    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_c = P32E2::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
    let mut sign_z = sign_a ^ sign_b; // ^ (op == softposit_mulAdd_subProd);

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }
    if sign_c {
        ui_c = ui_c.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = (tmp << 2) | 0x8000_0000;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * (((tmp << 2) | 0x8000_0000) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }

    let rcarry = (frac64_z >> 63) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
        frac64_z >>= 1;
    }

    if ui_c != 0 {
        let (k_c, exp_c, frac_c) = P32E2::separate_bits(ui_c);
        let mut frac64_c = (frac_c as u64) << 32;
        let mut shift_right = ((k_a - k_c) << 2) + (exp_a - exp_c) as i16;

        exp_z = if shift_right < 0 {
            // |ui_c| > |Prod|
            if shift_right <= -63 {
                bits_more = true;
                frac64_z = 0;
                shift_right = 0;
            //set bits_more to one?
            } else if (frac64_z << (64 + shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z = frac64_c + (frac64_z >> -shift_right);
            } else {
                //different signs
                frac64_z = frac64_c - (frac64_z >> -shift_right);
                sign_z = sign_c;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_c;
            exp_c
        } else if shift_right > 0 {
            // |ui_c| < |Prod|
            //if frac32C&((1<<shift_right)-1) {bits_more = true;}
            if shift_right >= 63 {
                bits_more = true;
                frac64_c = 0;
                shift_right = 0;
            } else if (frac64_c << (64 - shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z += frac64_c >> shift_right;
            } else {
                frac64_z -= frac64_c >> shift_right;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_a;
            exp_a
        } else {
            if (frac64_c == frac64_z) && (sign_z != sign_c) {
                //check if same number
                return P32E2::from_bits(0);
            } else if sign_z == sign_c {
                frac64_z += frac64_c;
            } else if frac64_z < frac64_c {
                frac64_z = frac64_c - frac64_z;
                sign_z = sign_c;
            } else {
                frac64_z -= frac64_c;
            }
            k_z = k_a; // actually can be k_c too, no diff
            exp_a //same here
        };
        let rcarry = (frac64_z >> 63) != 0; //first left bit

        if rcarry {
            exp_z += 1;
            if exp_z > 3 {
                k_z += 1;
                exp_z &= 0x3;
            }
            frac64_z = (frac64_z >> 1) & 0x7FFF_FFFF_FFFF_FFFF;
        } else {
            //for subtract cases
            if frac64_z != 0 {
                while (frac64_z >> 59) == 0 {
                    k_z -= 1;
                    frac64_z <<= 4;
                }
                while (frac64_z >> 62) == 0 {
                    exp_z -= 1;
                    frac64_z <<= 1;
                    if exp_z < 0 {
                        k_z -= 1;
                        exp_z = 3;
                    }
                }
            }
        }
    } else {
        k_z = k_a;
        exp_z = exp_a;
    }
    let reg_z: u32;
    let (reg_sz, regime) = if k_z < 0 {
        reg_z = (-k_z) as u32;
        (false, 0x4000_0000 >> reg_z)
    } else {
        reg_z = (k_z + 1) as u32;
        (true, 0x7FFF_FFFF - (0x7FFF_FFFF >> reg_z))
    };

    if reg_z > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            u_z = 0x7FFF_FFFF;
        } else {
            u_z = 0x1;
        }
    } else {
        if reg_z <= 28 {
            //remove hidden bits
            frac64_z &= 0x3FFF_FFFF_FFFF_FFFF;
            frac_z = (frac64_z >> (reg_z + 34)) as u32; //frac32Z>>16;
            bit_n_plus_one |= (0x2_0000_0000 & (frac64_z >> reg_z)) != 0;
            exp_z <<= 28 - reg_z;
        } else {
            if reg_z == 30 {
                bit_n_plus_one = (exp_z & 0x2) != 0;
                bits_more = (exp_z & 0x1) != 0;
                exp_z = 0;
            } else if reg_z == 29 {
                bit_n_plus_one = (exp_z & 0x1) != 0;
                exp_z >>= 1;
            }
            if frac_z > 0 {
                frac_z = 0;
                bits_more = true;
            }
        }
        u_z = P32E2::pack_to_ui(regime, exp_z as u32, frac_z);

        if bit_n_plus_one {
            if (frac64_z << (32 - reg_z)) != 0
            /* &0xFFFF_FFFF_FFFF_FFFF */
            {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u32);
        }
    }
    P32E2::from_bits(u_z.with_sign(sign_z))
}

fn round(p_a: P32E2) -> P32E2 {
    let mut mask = 0x2000_0000_u32;
    let mut scale = 0_u32;

    let u_a: u32;

    let mut ui_a = p_a.to_bits();
    let sign = (ui_a >> 31) != 0;

    // sign is True if pA > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg();
    } // A is now |A|.
    if ui_a <= 0x3800_0000 {
        // 0 <= |pA| <= 1/2 rounds to zero.
        return P32E2::from_bits(0);
    } else if ui_a < 0x4400_0000 {
        // 1/2 < x < 3/2 rounds to 1.
        u_a = 0x4000_0000;
    } else if ui_a <= 0x4A00_0000 {
        // 3/2 <= x <= 5/2 rounds to 2.
        u_a = 0x4800_0000;
    } else if ui_a >= 0x7E80_0000 {
        // If |A| is 0x7E80_0000 (posit is pure integer value), leave it unchanged.
        return p_a; // This also takes care of the NaR case, 0x8000_0000.
    } else {
        // 34% of the cases, we have to decode the posit.

        while (mask & ui_a) != 0 {
            scale += 4;
            mask >>= 1;
        }
        mask >>= 1;

        //Exponential (2 bits)
        if (mask & ui_a) != 0 {
            scale += 2;
        }
        mask >>= 1;
        if (mask & ui_a) != 0 {
            scale += 1;
        }
        mask >>= scale;

        //the rest of the bits
        let bit_last = (ui_a & mask) != 0;
        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp != 0;
        ui_a ^= tmp; // Erase the bit, if it was set.
        tmp = ui_a & (mask - 1); // this is actually bits_more

        ui_a ^= tmp;

        if bit_n_plus_one && (((bit_last as u32) | tmp) != 0) {
            ui_a += mask << 1;
        }
        u_a = ui_a;
    }
    P32E2::from_bits(u_a.with_sign(sign))
}

#[inline]
fn sqrt(p_a: P32E2) -> P32E2 {
    let mut ui_a = p_a.to_bits();

    // If NaR or a negative number, return NaR.
    if (ui_a & 0x8000_0000) != 0 {
        return P32E2::from_bits(0x8000_0000);
    }
    // If the argument is zero, return zero.
    else if ui_a == 0 {
        return p_a;
    }
    // Compute the square root; shift_z is the power-of-2 scaling of the result.
    // Decode regime and exponent; scale the input to be in the range 1 to 4:
    let mut shift_z: i32;
    if (ui_a & 0x4000_0000) != 0 {
        shift_z = -2;
        while (ui_a & 0x4000_0000) != 0 {
            shift_z += 2;
            ui_a <<= 1 /*() & 0xFFFF_FFFF*/;
        }
    } else {
        shift_z = 0;
        while (ui_a & 0x4000_0000) == 0 {
            shift_z -= 2;
            ui_a <<= 1 /*90 & 0xFFFF_FFFF*/;
        }
    }

    ui_a &= 0x3FFF_FFFF;
    let mut exp_a = ui_a >> 28;
    shift_z += (exp_a >> 1) as i32;
    exp_a = 0x1 ^ (exp_a & 0x1);
    ui_a &= 0x0FFF_FFFF;
    let frac_a = ui_a | 0x1000_0000;

    // Use table look-up of first 4 bits for piecewise linear approx. of 1/sqrt:
    let index = (((frac_a >> 24) & 0xE) + exp_a) as usize;
    let eps = ((frac_a >> 9) & 0xFFFF) as i32;
    let r0: u32 = (crate::APPROX_RECIP_SQRT0[index] as u32)
        - (((crate::APPROX_RECIP_SQRT1[index] as u32) * (eps as u32)) >> 20);

    // Use Newton-Raphson refinement to get 33 bits of accuracy for 1/sqrt:
    let mut e_sqr_r0 = (r0 as u64) * (r0 as u64);
    if exp_a == 0 {
        e_sqr_r0 <<= 1;
    }
    let sigma0: u64 = 0xFFFF_FFFF & (0xFFFF_FFFF ^ ((e_sqr_r0 * (frac_a as u64)) >> 20));
    let mut recip_sqrt: u64 = ((r0 as u64) << 20) + (((r0 as u64) * sigma0) >> 21);

    let sqr_sigma0 = (sigma0 * sigma0) >> 35;
    recip_sqrt += ((recip_sqrt + (recip_sqrt >> 2) - ((r0 as u64) << 19)) * sqr_sigma0) >> 46;

    let mut frac_z = ((frac_a as u64) * recip_sqrt) >> 31;
    if exp_a != 0 {
        frac_z >>= 1;
    }

    // Find the exponent of Z and encode the regime bits.
    let exp_z = (shift_z as u32) & 0x3;
    let shift: u32;
    let ui_z: u32 = if shift_z < 0 {
        shift = ((-1 - shift_z) >> 2) as u32;
        0x2000_0000 >> shift
    } else {
        shift = (shift_z >> 2) as u32;
        0x7FFF_FFFF - (0x3FFF_FFFF >> shift)
    };

    // Trick for eliminating off-by-one cases that only uses one multiply:
    frac_z += 1;

    if (frac_z & 0xF) == 0 {
        let shifted_frac_z = frac_z >> 1;
        let neg_rem = (shifted_frac_z * shifted_frac_z) & 0x1_FFFF_FFFF;
        if (neg_rem & 0x1_0000_0000) != 0 {
            frac_z |= 1;
        } else if neg_rem != 0 {
            frac_z -= 1;
        }
    }
    // Strip off the hidden bit and round-to-nearest using last shift+5 bits.
    frac_z &= 0xFFFF_FFFF;
    let mask = 1 << (4 + shift);
    if ((mask & frac_z) != 0) && ((((mask - 1) & frac_z) | ((mask << 1) & frac_z)) != 0) {
        frac_z += mask << 1;
    }
    // Assemble the result and return it.
    P32E2::from_bits(ui_z | (exp_z << (27 - shift)) | (frac_z >> (5 + shift)) as u32)
}
/*
quire32_t q32_fdp_add( quire32_t q, posit32_t pA, posit32_t pB ){

    union ui32_p32 u_a, uB;
    union ui512_q32 u_z, u_z1, u_z2;
    u32 ui_a, ui_b;
    u32 frac_a, tmp;
    bool sign_a, sign_b, sign_z2, reg_sa, reg_sb, rcarry;
    i32 exp_a;
    i16 k_a=0, shift_right=0;
    u64 frac64_z;
    //For add
    bool rcarryb, b1, b2, rcarryZ=0;

    u_z1.q = q;

    u_a.p = pA;
    ui_a = u_a.ui;
    uB.p = pB;
    ui_b = uB.ui;

    u_z2.q = q32Clr(u_z2.q); //set it to zero
    //NaR
    if (isNaRQ32(q) || isNaRP32E2UI(u_a.ui) || isNaRP32E2UI(uB.ui)){
        //set to all zeros
        u_z2.ui[0]=0x8000_0000_0000_0000;
        return u_z2.q;
    }
    else if (ui_a==0 || ui_b==0)
        return q;


    //max pos (sign plus and minus)
    sign_a = signP32E2UI( ui_a );
    sign_b = signP32E2UI( ui_b );
    sign_z2 = sign_a ^ sign_b;

    if(sign_a) ui_a = (-ui_a /* & 0xFFFF_FFFF*/);
    if(sign_b) ui_b = (-ui_b /* & 0xFFFF_FFFF*/);

    reg_sa = signregP32E2UI(ui_a);
    reg_sb = signregP32E2UI(ui_b);

    tmp = ui_a<<2 /* & 0xFFFF_FFFF*/;
    if (reg_sa){
        while (tmp>>31){
            k_a += 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
    }
    else{
        k_a=-1;
        while (!(tmp>>31)){
            k_a -= 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
        tmp&=0x7FFF_FFFF;
    }
    exp_a = tmp>>29; //to get 2 bits
    frac_a = ((tmp<<2) | 0x8000_0000) /* & 0xFFFF_FFFF*/;


    tmp = (ui_b<<2) /* & 0xFFFF_FFFF*/;
    if (reg_sb){
        while (tmp>>31){
            k_a += 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
    }
    else{
        k_a -= 1;
        while (!(tmp>>31)){
            k_a -= 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
        tmp&=0x7FFF_FFFF;
    }
    exp_a += tmp>>29;
    frac64_z = (u64) frac_a * (((tmp<<2) | 0x8000_0000) /* & 0xFFFF_FFFF*/);

    if (exp_a>3){
        k_a += 1;
        exp_a&=0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    rcarry = frac64_z>>63;//1st bit of frac64_z
    if (rcarry){
        exp_a += 1;
        if (exp_a>3){
            k_a += 1;
            exp_a&=0x3;
        }
    }
    else
        frac64_z<<=1;

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    int firstPos = 271 - (k_a<<2) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    int i;
    for (i=0; i<8; i += 1){
        if (firstPos<(i+1)*64){
            //Need to check how much of the fraction is in the next 64 bits
            shift_right = firstPos - (i*64);
            u_z2.ui[i] = frac64_z >> shift_right;

            if (i!=7 && shift_right!=0) u_z2.ui[i+1] = frac64_z << (64 - shift_right);
            break;
        }
    }

    if (sign_z2){
        for (i=7; i>=0; i -= 1){
            if (u_z2.ui[i]>0){
                u_z2.ui[i] = - u_z2.ui[i];
                i -= 1;
                while(i>=0){
                    u_z2.ui[i] = ~u_z2.ui[i];
                    i -= 1;
                }
                break;
            }
        }
    }

    //Addition
    for (i=7; i>=0; i -= 1){
        b1 = u_z1.ui[i] & 0x1;
        b2 = u_z2.ui[i] & 0x1;
        if (i==7){
            rcarryb = b1 & b2;
            u_z.ui[i] = (u_z1.ui[i]>>1) + (u_z2.ui[i]>>1) + rcarryb;
            rcarryZ = u_z.ui[i]>>63;
            u_z.ui[i] = (u_z.ui[i]<<1 | (b1^b2) );
        }
        else{
            i8 rcarryb3 =  b1 + b2 + rcarryZ;
            u_z.ui[i] = (u_z1.ui[i]>>1) + (u_z2.ui[i]>>1) + (rcarryb3>>1);
            rcarryZ = u_z.ui[i]>>63;
            u_z.ui[i] = (u_z.ui[i]<<1 | (rcarryb3 & 0x1) );
        }

    }

    //Exception handling
    if (isNaRQ32(u_z.q) ) u_z.q.v[0]=0;

    return u_z.q;
}




quire32_t q32_fdp_sub( quire32_t q, posit32_t pA, posit32_t pB ){

    union ui32_p32 u_a, uB;
    union ui512_q32 u_z, u_z1, u_z2;
    u32 ui_a, ui_b;
    u32 frac_a, tmp;
    bool sign_a, sign_b, sign_z2, reg_sa, reg_sb, rcarry;
    i32 exp_a;
    i16 k_a=0, shift_right=0;
    u64 frac64_z;
    //For sub
    bool rcarryb, b1, b2, rcarryZ;

    u_z1.q = q;

    u_a.p = pA;
    ui_a = u_a.ui;
    uB.p = pB;
    ui_b = uB.ui;

    u_z2.q = q32Clr(u_z2.q); //set it to zero
    //NaR
    if (isNaRQ32(q) || isNaRP32E2UI(u_a.ui) || isNaRP32E2UI(uB.ui)){
        //set to all zeros
        u_z2.ui[0]=0x8000_0000_0000_0000;
        return u_z2.q;
    }
    else if (ui_a==0 || ui_b==0)
        return q;

    //max pos (sign plus and minus)
    sign_a = signP32E2UI( ui_a );
    sign_b = signP32E2UI( ui_b );
    sign_z2 = sign_a ^ sign_b;

    if(sign_a) ui_a = (-ui_a /* & 0xFFFF_FFFF*/);
    if(sign_b) ui_b = (-ui_b /* & 0xFFFF_FFFF*/);

    reg_sa = signregP32E2UI(ui_a);
    reg_sb = signregP32E2UI(ui_b);

    tmp = ui_a<<2 /* & 0xFFFF_FFFF*/;
    if (reg_sa){
        while (tmp>>31){
            k_a += 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
    }
    else{
        k_a=-1;
        while (!(tmp>>31)){
            k_a -= 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
        tmp&=0x7FFF_FFFF;
    }
    exp_a = tmp>>29; //to get 2 bits
    frac_a = ((tmp<<2) | 0x8000_0000) /* & 0xFFFF_FFFF*/;

    tmp = ui_b<<2 /* & 0xFFFF_FFFF*/;
    if (reg_sb){
        while (tmp>>31){
            k_a += 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
    }
    else{
        k_a -= 1;
        while (!(tmp>>31)){
            k_a -= 1;
            tmp <<= 1 /* & 0xFFFF_FFFF*/;
        }
        tmp&=0x7FFF_FFFF;
    }
    exp_a += tmp>>29;
    frac64_z = (u64) frac_a * (((tmp<<2) | 0x8000_0000) /* & 0xFFFF_FFFF*/);

    if (exp_a>3){
        k_a += 1;
        exp_a&=0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    rcarry = frac64_z>>63;//1st bit of frac64_z
    if (rcarry){
        exp_a += 1;
        if (exp_a>3){
            k_a += 1;
            exp_a&=0x3;
        }
        //frac64_z>>=1;
    }
    else
        frac64_z<<=1;

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    int firstPos = 271 - (k_a<<2) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    int i;
    for (i=0; i<8; i += 1){
        if (firstPos<(i+1)*64){
            //Need to check how much of the fraction is in the next 64 bits
            shift_right = firstPos - (i*64);
            u_z2.ui[i] = frac64_z >> shift_right;
            if (i!=7 && shift_right!=0) u_z2.ui[i+1] = frac64_z << (64 - shift_right);
            break;
        }
    }


    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if (!sign_z2){
        for (i=7; i>=0; i -= 1){
            if (u_z2.ui[i]>0){
                u_z2.ui[i] = - u_z2.ui[i];
                i -= 1;
                while(i>=0){
                    u_z2.ui[i] = ~u_z2.ui[i];
                    i -= 1;
                }
                break;
            }
        }

    }

    //Subtraction
    for (i=7; i>=0; i -= 1){
        b1 = u_z1.ui[i] & 0x1;
        b2 = u_z2.ui[i] & 0x1;
        if (i==7){
            rcarryb = b1 & b2;
            u_z.ui[i] = (u_z1.ui[i]>>1) + (u_z2.ui[i]>>1) + rcarryb;
            rcarryZ = u_z.ui[i]>>63;
            u_z.ui[i] = (u_z.ui[i]<<1 | (b1^b2) );
        }
        else{
            i8 rcarryb3 =  b1 + b2 + rcarryZ;
            u_z.ui[i] = (u_z1.ui[i]>>1) + (u_z2.ui[i]>>1) + (rcarryb3>>1);
            rcarryZ = u_z.ui[i]>>63;
            u_z.ui[i] = (u_z.ui[i]<<1 | (rcarryb3 & 0x1) );
        }

    }

    //Exception handling
    if (isNaRQ32(u_z.q) ) u_z.q.v[0]=0;

    return u_z.q;
}
*/
