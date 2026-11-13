use core::cmp::max;
use core::convert::TryFrom;
use ethnum::u256;

use crate::{Bn254, ZkError};

/// Dense BN254 polynomials with a compile-time cap on coefficient storage.
///
/// `MAX_COEFFS` bounds the canonical coefficient count. The `len` field tracks
/// how many leading coefficients are valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DensePolynomial<const MAX_COEFFS: usize> {
    pub coeffs: [u256; MAX_COEFFS],
    pub len: usize,
}

/// Sparse BN254 polynomials with a compile-time cap on term storage.
///
/// `MAX_TERMS` bounds the canonical `(exponent, coefficient)` term count. The
/// `len` field tracks how many leading entries are valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparsePolynomial<const MAX_TERMS: usize> {
    pub terms: [(usize, u256); MAX_TERMS],
    pub len: usize,
}

fn zero() -> u256 {
    u256::from(0u8)
}

fn one() -> u256 {
    u256::from(1u8)
}

fn canonical_len(values: &[u256]) -> usize {
    let mut len = values.len();
    while len > 0 && values[len - 1] == zero() {
        len -= 1;
    }
    len
}

impl<const MAX_COEFFS: usize> DensePolynomial<MAX_COEFFS> {
    pub fn from_coefficients_slice(coeffs: &[u256]) -> Result<Self, ZkError> {
        let len = canonical_len(coeffs);
        if len > MAX_COEFFS {
            return Err(ZkError::InvalidInput);
        }

        let mut out = [zero(); MAX_COEFFS];
        out[..len].copy_from_slice(&coeffs[..len]);

        Ok(Self { coeffs: out, len })
    }

    pub fn zero() -> Self {
        Self {
            coeffs: [zero(); MAX_COEFFS],
            len: 0,
        }
    }

    pub fn one() -> Self {
        if MAX_COEFFS == 0 {
            panic!();
        }

        let mut coeffs = [zero(); MAX_COEFFS];
        coeffs[0] = one();

        Self { coeffs, len: 1 }
    }

    pub fn is_zero(&self) -> bool {
        self.len == 0
    }

    pub fn degree(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            self.len - 1
        }
    }

    pub fn coeffs(&self) -> &[u256] {
        &self.coeffs[..self.len]
    }

    pub fn evaluate(&self, x: u256) -> u256 {
        if self.is_zero() {
            return zero();
        }

        let mut result = zero();
        for coeff in self.coeffs().iter().rev() {
            result = Bn254::mul(result, x);
            result = Bn254::add(result, *coeff);
        }
        result
    }

    pub fn add(&self, other: &Self) -> Self {
        let len = max(self.len, other.len);
        debug_assert!(len <= MAX_COEFFS);

        let mut coeffs = [zero(); MAX_COEFFS];
        for (i, slot) in coeffs.iter_mut().enumerate().take(len) {
            let a = if i < self.len { self.coeffs[i] } else { zero() };
            let b = if i < other.len {
                other.coeffs[i]
            } else {
                zero()
            };
            *slot = Bn254::add(a, b);
        }

        Self {
            coeffs,
            len: canonical_len(&coeffs[..len]),
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let len = max(self.len, other.len);
        debug_assert!(len <= MAX_COEFFS);

        let mut coeffs = [zero(); MAX_COEFFS];
        for (i, slot) in coeffs.iter_mut().enumerate().take(len) {
            let a = if i < self.len { self.coeffs[i] } else { zero() };
            let b = if i < other.len {
                other.coeffs[i]
            } else {
                zero()
            };
            *slot = Bn254::sub(a, b);
        }

        Self {
            coeffs,
            len: canonical_len(&coeffs[..len]),
        }
    }

    pub fn try_mul(&self, other: &Self) -> Result<Self, ZkError> {
        if self.is_zero() || other.is_zero() {
            return Ok(Self::zero());
        }

        let len = self.len + other.len - 1;
        if len > MAX_COEFFS {
            return Err(ZkError::InvalidInput);
        }

        let mut coeffs = [zero(); MAX_COEFFS];
        for i in 0..self.len {
            for j in 0..other.len {
                let product = Bn254::mul(self.coeffs[i], other.coeffs[j]);
                coeffs[i + j] = Bn254::add(coeffs[i + j], product);
            }
        }

        Ok(Self { coeffs, len })
    }

    pub fn mul(&self, other: &Self) -> Self {
        match self.try_mul(other) {
            Ok(poly) => poly,
            Err(_) => panic!(),
        }
    }

    pub fn scalar_mul(&self, scalar: u256) -> Self {
        let mut coeffs = [zero(); MAX_COEFFS];
        for (i, slot) in coeffs.iter_mut().enumerate().take(self.len) {
            *slot = Bn254::mul(self.coeffs[i], scalar);
        }

        Self {
            coeffs,
            len: canonical_len(&coeffs[..self.len]),
        }
    }
}

impl<const MAX_TERMS: usize> SparsePolynomial<MAX_TERMS> {
    pub fn from_terms_slice(terms: &[(usize, u256)]) -> Result<Self, ZkError> {
        let mut out = [(0usize, zero()); MAX_TERMS];
        let mut len = 0;

        for &(exp, coeff) in terms {
            if coeff == zero() {
                continue;
            }

            let mut insert_at = 0;
            while insert_at < len && out[insert_at].0 < exp {
                insert_at += 1;
            }

            if insert_at < len && out[insert_at].0 == exp {
                let new_coeff = Bn254::add(out[insert_at].1, coeff);
                if new_coeff == zero() {
                    for i in insert_at..(len - 1) {
                        out[i] = out[i + 1];
                    }
                    len -= 1;
                } else {
                    out[insert_at].1 = new_coeff;
                }
                continue;
            }

            if len == MAX_TERMS {
                return Err(ZkError::InvalidInput);
            }

            for i in (insert_at..len).rev() {
                out[i + 1] = out[i];
            }
            out[insert_at] = (exp, coeff);
            len += 1;
        }

        Ok(Self { terms: out, len })
    }

    pub fn zero() -> Self {
        Self {
            terms: [(0usize, zero()); MAX_TERMS],
            len: 0,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.len == 0
    }

    pub fn degree(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            self.terms[self.len - 1].0
        }
    }

    pub fn terms(&self) -> &[(usize, u256)] {
        &self.terms[..self.len]
    }

    pub fn evaluate(&self, x: u256) -> u256 {
        let mut result = zero();

        for &(exp, coeff) in self.terms() {
            let x_pow = if exp == 0 {
                one()
            } else {
                Bn254::pow(x, u256::from(exp as u128))
            };
            let term = Bn254::mul(coeff, x_pow);
            result = Bn254::add(result, term);
        }

        result
    }
}

impl<const MAX_COEFFS: usize, const MAX_TERMS: usize> TryFrom<&SparsePolynomial<MAX_TERMS>>
    for DensePolynomial<MAX_COEFFS>
{
    type Error = ZkError;

    fn try_from(value: &SparsePolynomial<MAX_TERMS>) -> Result<Self, Self::Error> {
        if value.is_zero() {
            return Ok(Self::zero());
        }

        let degree = value.degree();
        if degree + 1 > MAX_COEFFS {
            return Err(ZkError::InvalidInput);
        }

        let mut coeffs = [zero(); MAX_COEFFS];
        for &(exp, coeff) in value.terms() {
            coeffs[exp] = coeff;
        }

        Ok(Self {
            coeffs,
            len: degree + 1,
        })
    }
}

impl<const MAX_COEFFS: usize, const MAX_TERMS: usize> TryFrom<&DensePolynomial<MAX_COEFFS>>
    for SparsePolynomial<MAX_TERMS>
{
    type Error = ZkError;

    fn try_from(value: &DensePolynomial<MAX_COEFFS>) -> Result<Self, Self::Error> {
        let mut out = [(0usize, zero()); MAX_TERMS];
        let mut len = 0;

        for (exp, coeff) in value.coeffs().iter().copied().enumerate() {
            if coeff == zero() {
                continue;
            }

            if len == MAX_TERMS {
                return Err(ZkError::InvalidInput);
            }

            out[len] = (exp, coeff);
            len += 1;
        }

        Ok(Self { terms: out, len })
    }
}

#[cfg(test)]
#[cfg(not(miri))]
mod tests {
    use super::*;

    type Dense4 = DensePolynomial<4>;
    type Sparse4 = SparsePolynomial<4>;

    #[test]
    fn dense_strips_trailing_zeros_and_respects_bound() {
        let poly = Dense4::from_coefficients_slice(&[one(), zero(), zero()]).unwrap();
        assert_eq!(poly.coeffs(), &[one()]);
        assert_eq!(poly.degree(), 0);
    }

    #[test]
    fn dense_rejects_over_limit_after_canonicalization() {
        let coeffs = [one(), one(), one(), one(), one()];
        assert_eq!(
            Dense4::from_coefficients_slice(&coeffs),
            Err(ZkError::InvalidInput)
        );
    }

    #[test]
    fn sparse_combines_duplicates_and_filters_zeroes() {
        let poly =
            Sparse4::from_terms_slice(&[(0, one()), (2, one()), (2, zero()), (2, one())]).unwrap();

        assert_eq!(poly.terms(), &[(0, one()), (2, Bn254::add(one(), one()))]);
    }

    #[test]
    fn sparse_rejects_over_limit_after_dedup() {
        let terms = [(0, one()), (1, one()), (2, one()), (3, one()), (4, one())];

        assert_eq!(
            Sparse4::from_terms_slice(&terms),
            Err(ZkError::InvalidInput)
        );
    }

    #[test]
    fn dense_and_sparse_conversions_round_trip() {
        let dense = Dense4::from_coefficients_slice(&[one(), zero(), one()]).unwrap();
        let sparse: Sparse4 = Sparse4::try_from(&dense).unwrap();
        let dense_round_trip: Dense4 = Dense4::try_from(&sparse).unwrap();

        assert_eq!(dense_round_trip, dense);
    }

    #[test]
    fn dense_multiplication_enforces_bound() {
        let a = DensePolynomial::<3>::from_coefficients_slice(&[one(), one(), one()]).unwrap();
        let b = DensePolynomial::<3>::from_coefficients_slice(&[one(), one(), one()]).unwrap();

        assert_eq!(a.try_mul(&b), Err(ZkError::InvalidInput));
    }
}
