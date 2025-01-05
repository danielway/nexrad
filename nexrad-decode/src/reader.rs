use crate::messages::message_header::MessageHeader;
use crate::messages::{decode_message_header, MessageType};
use std::io;
use std::io::Read;
use uom::si::information::byte;

pub(crate) struct SegmentedMessageReader<R> {
    inner: R,
    headers: Vec<MessageHeader>,
    current_segment_bytes_left: usize,
    message_finished: bool,
}

impl<R: Read> SegmentedMessageReader<R> {
    pub(crate) fn new(mut inner: R) -> crate::result::Result<(Self, MessageType)> {
        let (header, segment_size_bytes, is_final_segment) =
            SegmentedMessageReader::decode_message_header(&mut inner)?;
        let message_type = header.message_type();

        let segment_count = header.segment_count().unwrap_or(1);
        let mut headers = Vec::with_capacity(segment_count as usize);
        headers[0] = header;

        Ok((
            SegmentedMessageReader {
                inner,
                headers,
                current_segment_bytes_left: segment_size_bytes,
                message_finished: is_final_segment,
            },
            message_type,
        ))
    }

    pub(crate) fn into_headers(self) -> Vec<MessageHeader> {
        self.headers
    }

    fn decode_message_header(
        reader: &mut R,
    ) -> crate::result::Result<(MessageHeader, usize, bool)> {
        let header = decode_message_header(reader)?;
        let segment_size_bytes = header.message_size().get::<byte>() as usize;
        let is_final_segment = header.segment_number() == header.segment_count();
        Ok((header, segment_size_bytes, is_final_segment))
    }
}

impl<R: Read> Read for SegmentedMessageReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.message_finished {
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
