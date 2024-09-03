pub type Sha1 = [u8; 20];

#[derive(Debug, Default, Clone)]
pub struct BitField {
    value: u8,
}

impl BitField {
    pub fn new(value: u8) -> Self {
        BitField { value }
    }

    pub fn get_bit(&self, position: u8) -> bool {
        if position < 8 {
            (self.value & (1 << position)) != 0
        } else {
            panic!("BitField::get_bit out of bounds")
        }
    }

    pub fn set_bit(&mut self, position: u8, bit_value: bool) {
        if position < 8 {
            if bit_value {
                self.value |= 1 << position;
            } else {
                self.value &= !(1 << position);
            }
        } else {
            panic!("BitField::set_bit out of bounds")
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }
}

pub struct BitFieldIterator {
    bit_field: BitField,
    position: u8,
}

impl Iterator for BitFieldIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= 8 {
            return None;
        }
        let value = self.bit_field.get_bit(self.position);
        self.position += 1;
        Some(value)
    }
}

impl From<BitField> for BitFieldIterator {
    fn from(value: BitField) -> Self {
        Self {
            bit_field: value,
            position: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{BitField, BitFieldIterator};

    #[test]
    fn bitfield_get() {
        let bitfield = BitField::new(0b0100_1001);
        assert_eq!(bitfield.get_bit(7), false);
        assert_eq!(bitfield.get_bit(6), true);
        assert_eq!(bitfield.get_bit(5), false);
        assert_eq!(bitfield.get_bit(4), false);
        assert_eq!(bitfield.get_bit(3), true);
        assert_eq!(bitfield.get_bit(2), false);
        assert_eq!(bitfield.get_bit(1), false);
        assert_eq!(bitfield.get_bit(0), true);
    }

    #[test]
    fn bitfield_set() {
        let mut bitfield = BitField::new(0b0100_1001);
        bitfield.set_bit(7, true);
        bitfield.set_bit(6, false);
        assert_eq!(bitfield.get_bit(7), true);
        assert_eq!(bitfield.get_bit(6), false);
        assert_eq!(bitfield.get_bit(5), false);
        assert_eq!(bitfield.get_bit(4), false);
        assert_eq!(bitfield.get_bit(3), true);
        assert_eq!(bitfield.get_bit(2), false);
        assert_eq!(bitfield.get_bit(1), false);
        assert_eq!(bitfield.get_bit(0), true);
    }

    #[test]
    fn bitfield_iterator() {
        let mut iterator: BitFieldIterator = BitField::new(0b0100_1001).into();
        assert_eq!(iterator.next().unwrap(), true);
        assert_eq!(iterator.next().unwrap(), false);
        assert_eq!(iterator.next().unwrap(), false);
        assert_eq!(iterator.next().unwrap(), true);
        assert_eq!(iterator.next().unwrap(), false);
        assert_eq!(iterator.next().unwrap(), false);
        assert_eq!(iterator.next().unwrap(), true);
        assert_eq!(iterator.next().unwrap(), false);
        assert_eq!(iterator.next(), None);
    }
}
