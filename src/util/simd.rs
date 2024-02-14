/*
 * Sanctaphraxx, a UAI Ataxx engine
 * Copyright (C) 2024 Ciekce
 *
 * Sanctaphraxx is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Sanctaphraxx is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Sanctaphraxx. If not, see <https://www.gnu.org/licenses/>.
 */

use std::arch::x86_64::*;

#[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
pub type Register16 = __m512i;

#[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
pub type Register32 = __m512i;

#[cfg(all(
    target_feature = "avx2",
    not(all(target_feature = "avx512f", target_feature = "avx512bw"))
))]
pub type Register16 = __m256i;

#[cfg(all(
    target_feature = "avx2",
    not(all(target_feature = "avx512f", target_feature = "avx512bw"))
))]
pub type Register32 = __m256i;

#[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
pub type Register16 = __m128i;

#[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
pub type Register32 = __m128i;

// avx512 and avx2 imply sse 4.1
#[cfg(not(target_feature = "sse4.1"))]
pub type Register16 = i16;

#[cfg(not(target_feature = "sse4.1"))]
pub type Register32 = i32;

pub const CHUNK_SIZE_I16: usize = std::mem::size_of::<Register16>() / std::mem::size_of::<i16>();

#[inline(always)]
pub fn zero16() -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_setzero_si512()
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_setzero_si256()
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_setzero_si128()
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            0
        }
    }
}

#[inline(always)]
pub fn set1_i16(v: i16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_set1_epi16(v)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_set1_epi16(v)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_set1_epi16(v)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            v
        }
    }
}

//#[inline(always)]
pub unsafe fn load16(ptr: *const Register16) -> Register16 {
    debug_assert_eq!(
        ptr.cast::<u8>()
            .align_offset(std::mem::size_of::<Register16>()),
        0
    );

    #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
    {
        _mm512_load_si512(ptr as *const i32)
    }

    #[cfg(all(
        target_feature = "avx2",
        not(all(target_feature = "avx512f", target_feature = "avx512bw"))
    ))]
    {
        _mm256_load_si256(ptr)
    }

    #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
    {
        _mm_load_si128(ptr)
    }

    #[cfg(not(target_feature = "sse4.1"))]
    {
        *ptr
    }
}

#[inline(always)]
pub unsafe fn store16(ptr: *mut Register16, v: Register16) {
    debug_assert_eq!(
        ptr.cast::<u8>()
            .align_offset(std::mem::size_of::<Register16>()),
        0
    );

    #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
    {
        _mm512_store_si512(ptr as *mut i32, v)
    }

    #[cfg(all(
        target_feature = "avx2",
        not(all(target_feature = "avx512f", target_feature = "avx512bw"))
    ))]
    {
        _mm256_store_si256(ptr, v)
    }

    #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
    {
        _mm_store_si128(ptr, v)
    }

    #[cfg(not(target_feature = "sse4.1"))]
    {
        *ptr = v
    }
}

#[inline(always)]
pub fn min_i16(a: Register16, b: Register16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_min_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_min_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_min_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a.min(b)
        }
    }
}

#[inline(always)]
pub fn max_i16(a: Register16, b: Register16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_max_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_max_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_max_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a.max(b)
        }
    }
}

#[inline(always)]
pub fn clamp_i16(v: Register16, min: Register16, max: Register16) -> Register16 {
    #[cfg(target_feature = "sse4.1")]
    {
        min_i16(max_i16(v, min), max)
    }

    #[cfg(not(target_feature = "sse4.1"))]
    {
        v.clamp(min, max)
    }
}

#[inline(always)]
pub fn add_i16(a: Register16, b: Register16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_add_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_add_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_add_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a + b
        }
    }
}

#[inline(always)]
pub fn sub_i16(a: Register16, b: Register16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_sub_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_sub_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_sub_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a - b
        }
    }
}

#[inline(always)]
pub fn mul_i16(a: Register16, b: Register16) -> Register16 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_mullo_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_mullo_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_mullo_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a * b
        }
    }
}

#[inline(always)]
pub fn mul_add_adj_i16(a: Register16, b: Register16) -> Register32 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_madd_epi16(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_madd_epi16(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_madd_epi16(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a as i32 * b as i32
        }
    }
}

#[inline(always)]
pub fn zero32() -> Register32 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_setzero_si512()
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_setzero_si256()
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_setzero_si128()
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            0
        }
    }
}

#[inline(always)]
pub fn add_i32(a: Register32, b: Register32) -> Register32 {
    unsafe {
        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            _mm512_add_epi32(a, b)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            _mm256_add_epi32(a, b)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            _mm_add_epi32(a, b)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            a + b
        }
    }
}

#[inline(always)]
pub fn horizontal_sum_i32(v: Register32) -> i32 {
    // _MM_SHUFFLE is unstable for some reason?
    const fn mm_shuffle(z: u32, y: u32, x: u32, w: u32) -> i32 {
        ((z << 6) | (y << 4) | (x << 2) | w) as i32
    }

    unsafe {
        #[cfg(target_feature = "sse4.1")]
        #[inline(always)]
        unsafe fn impl_sse41(v: __m128i) -> i32 {
            let high64 = _mm_unpackhi_epi64(v, v);
            let sum64 = _mm_add_epi32(v, high64);

            let high32 = _mm_shuffle_epi32::<{ mm_shuffle(2, 3, 0, 1) }>(sum64);
            let sum32 = _mm_add_epi32(sum64, high32);

            _mm_cvtsi128_si32(sum32)
        }

        #[cfg(target_feature = "avx2")]
        #[inline(always)]
        unsafe fn impl_avx2(v: __m256i) -> i32 {
            let high128 = _mm256_extracti128_si256::<1>(v);
            let low128 = _mm256_castsi256_si128(v);

            let sum128 = _mm_add_epi32(high128, low128);

            impl_sse41(sum128)
        }

        #[cfg(all(target_feature = "avx512f", target_feature = "avx512bw"))]
        {
            let high256 = _mm512_extracti64x4_epi64::<1>(v);
            let low256 = _mm512_castsi512_si256(v);

            let sum256 = _mm256_add_epi32(high256, low256);

            impl_avx2(sum256)
        }

        #[cfg(all(
            target_feature = "avx2",
            not(all(target_feature = "avx512f", target_feature = "avx512bw"))
        ))]
        {
            impl_avx2(v)
        }

        #[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
        {
            impl_sse41(v)
        }

        #[cfg(not(target_feature = "sse4.1"))]
        {
            v
        }
    }
}
