use crate::messages::message_header::MessageHeader;
use crate::messages::{decode_message_header, MessageType};
use crate::result::Result;
use std::io;
use std::io::{Read, Seek};

const MESSAGE_HEADER_SIZE: usize = size_of::<MessageHeader>();

pub(crate) struct SegmentedMessageReader<R> {
    inner: R,
    headers: Vec<MessageHeader>,
    _current_segment_number: Option<u16>,
    _segment_count: Option<u16>,
    current_segment_bytes_left: usize,
    message_finished: bool,
}

impl<R: Read + Seek> SegmentedMessageReader<R> {
    pub(crate) fn new(mut inner: R) -> Result<(Self, MessageType)> {
        let (header, segment_size_bytes, is_final_segment) =
            SegmentedMessageReader::decode_message_header(&mut inner)?;
        let message_type = header.message_type();
        let segment_number = header.segment_number();
        let segment_count = header.segment_count();

        let mut headers = Vec::with_capacity(segment_count.unwrap_or(1) as usize);
        headers.push(header);

        Ok((
            SegmentedMessageReader {
                inner,
                headers,
                _current_segment_number: segment_number,
                _segment_count: segment_count,
                current_segment_bytes_left: segment_size_bytes,
                message_finished: is_final_segment,
            },
            message_type,
        ))
    }

    pub(crate) fn into_headers(mut self) -> Result<Vec<MessageHeader>> {
        self.consume_remaining()?;
        Ok(self.headers)
    }

    fn decode_message_header(reader: &mut R) -> Result<(MessageHeader, usize, bool)> {
        let header = decode_message_header(reader)?;
        // let mut segment_size_bytes = header.message_size().get::<byte>() as usize;
        // segment_size_bytes -= MESSAGE_HEADER_SIZE - 12 - 4;
        let segment_size_bytes = 2432 - MESSAGE_HEADER_SIZE;
        let is_final_segment = header.segment_number() == header.segment_count();
        Ok((header, segment_size_bytes, is_final_segment))
    }

    fn consume_remaining(&mut self) -> Result<()> {
        let mut buf = [0u8; 1024];
        while !self.message_finished || self.current_segment_bytes_left > 0 {
            match self.read(&mut buf) {
                Ok(0) => {
                    break;
                } // EOF
                Ok(_bytes) => {
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}

impl<R: Read + Seek> Read for SegmentedMessageReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.message_finished && self.current_segment_bytes_left == 0 {
            return Ok(0);
        }

        if self.current_segment_bytes_left == 0 {
            let (header, segment_size_bytes, is_final_segment) =
                SegmentedMessageReader::decode_message_header(&mut self.inner)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            self.current_segment_bytes_left = segment_size_bytes;
            self.message_finished = is_final_segment;
            self.headers.push(header);

            if self.current_segment_bytes_left == 0 && self.message_finished {
                return Ok(0);
            }
        }

        let to_read = self.current_segment_bytes_left.min(buf.len());
        let bytes_read = self.inner.read(&mut buf[..to_read])?;
        if bytes_read == 0 {
            // TODO: EOF
            return Ok(0);
        }

        self.current_segment_bytes_left -= bytes_read;

        Ok(bytes_read)
    }
}
