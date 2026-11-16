//! Append-only Fn Lock records in the final 8 KiB, separate from the image.
use embedded_storage::nor_flash::NorFlash;

const START: u32 = 0xe000;
const END: u32 = 0x10000;
const PAGE: u32 = 2048;

pub struct Settings<F> {
    flash: F,
    latest: Option<(u32, u32, bool)>,
}

impl<F: NorFlash> Settings<F> {
    pub fn new(mut flash: F) -> Result<Self, F::Error> {
        let mut latest = None;
        for address in (START..END).step_by(8) {
            let mut record = [0; 8];
            flash.read(address, &mut record)?;
            if record[4..7] == *b"RMK" && record[7] <= 1 {
                let sequence = u32::from_le_bytes(record[..4].try_into().unwrap());
                if latest.is_none_or(|(_, old, _)| sequence > old) {
                    latest = Some((address, sequence, record[7] != 0));
                }
            }
        }
        Ok(Self { flash, latest })
    }

    pub fn fn_lock(&self) -> bool {
        self.latest.is_some_and(|(_, _, value)| value)
    }

    pub fn store(&mut self, value: bool) -> Result<(), F::Error> {
        if value == self.fn_lock() {
            return Ok(());
        }
        let (address, sequence) = self.latest.map_or((START, 0), |(address, sequence, _)| {
            let next = address + 8;
            (if next == END { START } else { next }, sequence + 1)
        });
        if address % PAGE == 0 {
            self.flash.erase(address, address + PAGE)?;
        }
        let mut record = [0; 8];
        record[..4].copy_from_slice(&sequence.to_le_bytes());
        record[4..7].copy_from_slice(b"RMK");
        record[7] = u8::from(value);
        self.flash.write(address, &record)?;
        self.latest = Some((address, sequence, value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_storage::nor_flash::{ErrorType, ReadNorFlash};

    use super::*;

    struct Flash {
        bytes: [u8; 8192],
        writes: usize,
        erases: usize,
    }
    impl ErrorType for Flash {
        type Error = core::convert::Infallible;
    }
    impl ReadNorFlash for Flash {
        const READ_SIZE: usize = 1;
        fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
            let start = (offset - START) as usize;
            bytes.copy_from_slice(&self.bytes[start..start + bytes.len()]);
            Ok(())
        }
        fn capacity(&self) -> usize {
            END as usize
        }
    }
    impl NorFlash for Flash {
        const WRITE_SIZE: usize = 8;
        const ERASE_SIZE: usize = PAGE as usize;
        fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
            assert_eq!(offset % 8, 0);
            let start = (offset - START) as usize;
            assert!(self.bytes[start..start + bytes.len()].iter().all(|&b| b == 255));
            self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
            self.writes += 1;
            Ok(())
        }
        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            assert_eq!(from % PAGE, 0);
            assert_eq!(to - from, PAGE);
            self.bytes[(from - START) as usize..(to - START) as usize].fill(255);
            self.erases += 1;
            Ok(())
        }
    }

    #[test]
    fn reboot_restores_lock_and_log_rotates_without_writing_unchanged_values() {
        let mut settings = Settings::new(Flash {
            bytes: [255; 8192],
            writes: 0,
            erases: 0,
        })
        .unwrap();
        assert!(!settings.fn_lock());
        for n in 0..1100 {
            let value = n % 2 == 0;
            settings.store(value).unwrap();
            settings.store(value).unwrap();
            settings = Settings::new(settings.flash).unwrap();
            assert_eq!(settings.fn_lock(), value);
        }
        assert_eq!(settings.flash.writes, 1100);
        assert_eq!(settings.flash.erases, 5);
    }
}
