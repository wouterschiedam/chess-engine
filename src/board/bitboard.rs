use crate::defs::{Bitboard, Square, EMPTY};
/// Iterator over squares in a bitboard
/// 
/// Iterates over each set bit in the bitboard, returning the square index (0-63)
/// for each set bit. This is useful for move generation - you can get all pieces
/// of a certain type, then iterate over their squares.
///
/// # Example
/// ```
/// use chess::board::bitboard::BitboardIter;
/// use chess::defs::Bitboard;
///
/// let bb: Bitboard = 0b1010; // bits set at squares 1 and 3
/// let squares: Vec<usize> = BitboardIter::new(bb).collect();
/// assert_eq!(squares, vec![1, 3]);
/// ```
pub struct BitboardIter {
    bits: Bitboard,
}

impl BitboardIter {
    /// Create a new iterator over the set bits in a bitboard
    #[inline(always)]
    pub fn new(bitboard: Bitboard) -> Self {
        Self { bits: bitboard }
    }
}

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == EMPTY {
            return None;
        }

        // Find the least significant set bit
        let square = self.bits.trailing_zeros() as Square;
        
        // Clear the least significant set bit
        self.bits &= self.bits.wrapping_sub(1);
        
        Some(square)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = pop_count(self.bits);
        (count, Some(count))
    }
}

impl ExactSizeIterator for BitboardIter {
    #[inline(always)]
    fn len(&self) -> usize {
        pop_count(self.bits)
    }
}

/// Count the number of set bits in a bitboard (population count)
/// 
/// Uses the built-in `count_ones()` which is typically optimized to use
/// hardware POPCNT instruction on modern CPUs.
#[inline(always)]
pub fn pop_count(bitboard: Bitboard) -> usize {
    bitboard.count_ones() as usize
}

/// Get the least significant bit (LSB) of a bitboard
/// 
/// Returns the bitboard with only the least significant set bit.
/// Returns EMPTY if the bitboard is empty.
#[inline(always)]
pub fn lsb(bitboard: Bitboard) -> Bitboard {
    if bitboard == EMPTY {
        EMPTY
    } else {
        bitboard & bitboard.wrapping_neg()
    }
}

/// Get the most significant bit (MSB) of a bitboard
/// 
/// Returns the bitboard with only the most significant set bit.
/// Returns EMPTY if the bitboard is empty.
#[inline(always)]
pub fn msb(bitboard: Bitboard) -> Bitboard {
    if bitboard == EMPTY {
        EMPTY
    } else {
        let msb_index = 63 - bitboard.leading_zeros();
        1u64 << msb_index
    }
}

/// Find the first (least significant) set bit in a bitboard
/// 
/// Returns the square index (0-63) of the first set bit.
/// Returns None if the bitboard is empty.
#[inline(always)]
pub fn bit_scan_forward(bitboard: Bitboard) -> Option<Square> {
    if bitboard == EMPTY {
        None
    } else {
        Some(bitboard.trailing_zeros() as Square)
    }
}

/// Find the last (most significant) set bit in a bitboard
/// 
/// Returns the square index (0-63) of the last set bit.
/// Returns None if the bitboard is empty.
#[inline(always)]
pub fn bit_scan_reverse(bitboard: Bitboard) -> Option<Square> {
    if bitboard == EMPTY {
        None
    } else {
        Some((63 - bitboard.leading_zeros()) as Square)
    }
}

/// Clear the least significant set bit
/// 
/// Returns the bitboard with the LSB cleared.
#[inline(always)]
pub fn clear_lsb(bitboard: Bitboard) -> Bitboard {
    bitboard & bitboard.wrapping_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_iter_empty() {
        let iter = BitboardIter::new(EMPTY);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_bitboard_iter_single_bit() {
        let iter = BitboardIter::new(1 << 5); // Square 5
        let squares: Vec<Square> = iter.collect();
        assert_eq!(squares, vec![5]);
    }

    #[test]
    fn test_bitboard_iter_multiple_bits() {
        let iter = BitboardIter::new(0b1010); // Squares 1 and 3
        let squares: Vec<Square> = iter.collect();
        assert_eq!(squares, vec![1, 3]);
    }

    #[test]
    fn test_bitboard_iter_all_bits() {
        let iter = BitboardIter::new(!0u64); // All 64 bits set
        assert_eq!(iter.count(), 64);
    }

    #[test]
    fn test_pop_count() {
        assert_eq!(pop_count(0), 0);
        assert_eq!(pop_count(1), 1);
        assert_eq!(pop_count(0b1010), 2);
        assert_eq!(pop_count(!0u64), 64);
    }

    #[test]
    fn test_lsb() {
        assert_eq!(lsb(EMPTY), EMPTY);
        assert_eq!(lsb(0b1010), 0b0010); // LSB is bit 1
        assert_eq!(lsb(1 << 63), 1 << 63); // MSB is also LSB when only one bit
    }

    #[test]
    fn test_msb() {
        assert_eq!(msb(EMPTY), EMPTY);
        assert_eq!(msb(0b1010), 0b1000); // MSB is bit 3
        assert_eq!(msb(1), 1); // LSB is also MSB when only one bit
    }

    #[test]
    fn test_bit_scan_forward() {
        assert_eq!(bit_scan_forward(EMPTY), None);
        assert_eq!(bit_scan_forward(0b1010), Some(1));
        assert_eq!(bit_scan_forward(1 << 63), Some(63));
    }

    #[test]
    fn test_bit_scan_reverse() {
        assert_eq!(bit_scan_reverse(EMPTY), None);
        assert_eq!(bit_scan_reverse(0b1010), Some(3));
        assert_eq!(bit_scan_reverse(1), Some(0));
    }

    #[test]
    fn test_clear_lsb() {
        assert_eq!(clear_lsb(EMPTY), EMPTY);
        assert_eq!(clear_lsb(0b1010), 0b1000); // Clear bit 1
        assert_eq!(clear_lsb(1), EMPTY); // Clear only bit
    }
}
