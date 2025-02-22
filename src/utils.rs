pub fn read_u16(d: &[u8]) -> u16 {
    u16::from_be_bytes([d[0], d[1]])
}

pub fn read_u32(d: &[u8]) -> u32 {
    u32::from_be_bytes([d[0], d[1], d[2], d[3]])
}

/// Add a 8 byte word.
#[inline]
pub fn add_8bytes(start: u64, value: [u8; 8]) -> u64 {
    let (sum, carry) = start.overflowing_add(u64::from_ne_bytes(value));
    sum + (carry as u64)
}

/// Add a 4 byte word.
#[inline]
pub fn add_4bytes(start: u64, value: [u8; 4]) -> u64 {
    let (sum, carry) = start.overflowing_add(u64::from(u32::from_ne_bytes(value)));
    sum + (carry as u64)
}

/// Add a 2 byte word.
#[inline]
pub fn add_2bytes(start: u64, value: [u8; 2]) -> u64 {
    let (sum, carry) = start.overflowing_add(u64::from(u16::from_ne_bytes(value)));
    sum + (carry as u64)
}

/// Add the given slice to the checksum. In case the slice
/// has a length that is not multiple of 2 the last byte
/// will be padded with 0.
#[inline]
pub fn add_slice(start_sum: u64, slice: &[u8]) -> u64 {
    let mut sum: u64 = start_sum;

    // sum up all 4 byte values
    let end_64 = slice.len() - (slice.len() % 8);
    for i in (0..end_64).step_by(8) {
        sum = add_8bytes(
            sum,
            // SAFETY:
            // Guranteed to always have at least 8 bytes to read
            // from i. As end_64 is gurenateed to be a multiple of
            // 8 bytes with a size equal or less then slice.len().
            unsafe {
                [
                    *slice.get_unchecked(i),
                    *slice.get_unchecked(i + 1),
                    *slice.get_unchecked(i + 2),
                    *slice.get_unchecked(i + 3),
                    *slice.get_unchecked(i + 4),
                    *slice.get_unchecked(i + 5),
                    *slice.get_unchecked(i + 6),
                    *slice.get_unchecked(i + 7),
                ]
            },
        );
    }

    // in case 4 or more bytes are left add the first 4 bytes
    let end_32 = if slice.len() - end_64 >= 4 {
        sum = add_4bytes(
            sum,
            // SAFETY:
            // If check guarantees there to be at least
            // 2 bytes.
            unsafe {
                [
                    *slice.get_unchecked(end_64),
                    *slice.get_unchecked(end_64 + 1),
                    *slice.get_unchecked(end_64 + 2),
                    *slice.get_unchecked(end_64 + 3),
                ]
            },
        );

        // shift by 4
        end_64 + 4
    } else {
        end_64
    };

    // in case 2 bytes are left add them as an word
    if slice.len() - end_32 >= 2 {
        sum = add_2bytes(
            sum,
            // SAFETY:
            // If check guarantees there to be at least
            // 2 bytes.
            unsafe {
                [
                    *slice.get_unchecked(end_32),
                    *slice.get_unchecked(end_32 + 1),
                ]
            },
        );
    }

    // unaligned end pad the last byte with
    if 0 != slice.len() % 2 {
        sum = add_2bytes(
            sum,
            // SAFETY:
            // If check guarantees there to be at least
            // 2 bytes.
            unsafe { [*slice.get_unchecked(slice.len() - 1), 0] },
        );
    }

    // done
    sum
}

/// Converts summed up words from an u64 to an u16 with 0 being replaced by 0xffff (useful
/// for TCP and UDP headers).
///
/// This kind of checksum is used in TCP and udp headers.
#[inline]
pub fn ones_complement_with_no_zero(sum: u64) -> u16 {
    // In case of 0 use the ones complement (zero is reserved
    // value for no checksum).
    let u16value = ones_complement(sum);
    if u16value == 0 {
        0xffff
    } else {
        u16value
    }
}

/// Converts summed up words from an u64 to an u16 which can be used in a ipv4.
#[inline]
pub fn ones_complement(sum: u64) -> u16 {
    let first =
        ((sum >> 48) & 0xffff) + ((sum >> 32) & 0xffff) + ((sum >> 16) & 0xffff) + (sum & 0xffff);
    // Add the upper 16 bits to the lower 16 bits twice.
    //
    // Notes: Two carry adds are needed as the first one could
    //        result in an additional carry add.
    let second = ((first >> 16) & 0xffff) + (first & 0xffff);
    let u16value = (((second >> 16) & 0xffff) + (second & 0xffff)) as u16;

    // switch back to big endian (allows to use
    // native endinaess during calculations).
    !u16value
}
