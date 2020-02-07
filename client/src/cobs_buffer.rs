/// For more information on Consistent Overhead Byte Stuffing (COBS) see:
/// https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
use postcard::from_bytes_cobs;
use serde::de::DeserializeOwned;

pub enum BufferResult<'a, T> {
    Consumed,
    Overfull(&'a [u8]),
    DeserErr(&'a [u8]),
    Success { data: T, remaining: &'a [u8] },
}

pub struct Buffer {
    buffer: [u8; 256],
    index: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buffer: [0u8; 256],
            index: 0,
        }
    }

    pub fn write<'a, T: DeserializeOwned>(&mut self, data: &'a [u8]) -> BufferResult<'a, T> {
        if data.is_empty() {
            return BufferResult::Consumed;
        }

        let delimiter = data.iter().position(|&b| b == 0);

        if let Some(i) = delimiter {
            let (take, release) = data.split_at(i + 1);

            if (self.index + i) <= 256 {
                self.append_unchecked(take);

                let result = match from_bytes_cobs::<T>(&mut self.buffer[..self.index]) {
                    Ok(t) => BufferResult::Success {
                        data: t,
                        remaining: release,
                    },
                    Err(_) => BufferResult::DeserErr(release),
                };

                self.index = 0;

                result
            } else {
                self.index = 0;
                BufferResult::Overfull(release)
            }
        } else if (self.index + data.len()) > 256 {
            let new_start = 256 - self.index;
            self.index = 0;

            BufferResult::Overfull(&data[new_start..])
        } else {
            self.append_unchecked(data);

            BufferResult::Consumed
        }
    }

    fn append_unchecked(&mut self, data: &[u8]) {
        let new_end = self.index + data.len();
        self.buffer[self.index..new_end].copy_from_slice(data);
        self.index = new_end;
    }
}
