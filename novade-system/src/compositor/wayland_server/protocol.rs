// In novade-system/src/compositor/wayland_server/protocol.rs

use crate::compositor::wayland_server::error::WaylandServerError;
use bytes::{Bytes, Buf}; // Using Bytes for efficient buffer handling
use std::convert::TryInto;
use tracing::{error, debug, trace};

// Wayland messages are:
// - sender object_id (u32)
// - size (u16) and opcode (u16) (packed into one u32)
// - arguments (variable size)

pub const MESSAGE_HEADER_SIZE: usize = 8; // object_id (4 bytes) + size_opcode (4 bytes)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32); // Newtype for object IDs

impl ObjectId {
    pub fn new(id: u32) -> Self {
        ObjectId(id)
    }
    pub fn value(&self) -> u32 {
        self.0
    }
}
impl From<u32> for ObjectId {
    fn from(id: u32) -> Self {
        ObjectId(id)
    }
}


#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub object_id: ObjectId,
    pub size: u16,
    pub opcode: u16,
}

#[derive(Debug, Clone)]
pub struct RawMessage {
    pub header: MessageHeader,
    pub content: Bytes, // The actual arguments part of the message
    // TODO: Later, this might include file descriptors if we handle ancillary data
}

#[derive(Debug, Clone, Copy)]
pub enum WaylandType {
    Int(i32),
    Uint(u32),
    Fixed(i32), // 24.8 fixed point
    String(u32), // Length prefixed, null terminated, padded
    Object(ObjectId),
    NewId(u32), // Placeholder, might be an object or just ID
    Array(u32), // Length prefixed
    Fd(i32), // File descriptor - special handling
}


impl MessageHeader {
    pub fn parse(buffer: &mut Bytes) -> Result<Self, WaylandServerError> {
        if buffer.len() < MESSAGE_HEADER_SIZE {
            trace!("Buffer too small for message header: {} bytes", buffer.len());
            return Err(WaylandServerError::Protocol("Buffer too small for message header".to_string()));
        }

        let object_id_raw = buffer.get_u32_le();
        let size_opcode_raw = buffer.get_u32_le();

        let object_id = ObjectId::new(object_id_raw);
        let size = (size_opcode_raw >> 16) as u16; // Higher 16 bits
        let opcode = (size_opcode_raw & 0xFFFF) as u16; // Lower 16 bits

        if size < (MESSAGE_HEADER_SIZE as u16) {
            error!("Invalid message size in header: {} (must be at least {})", size, MESSAGE_HEADER_SIZE);
            return Err(WaylandServerError::Protocol(format!(
                "Invalid message size in header: {} (must be at least {})",
                size, MESSAGE_HEADER_SIZE
            )));
        }

        // The size includes the header itself.
        // The remaining payload size is size - MESSAGE_HEADER_SIZE.

        debug!("Parsed message header: ObjectId({}), Size: {}, Opcode: {}", object_id.value(), size, opcode);

        Ok(MessageHeader {
            object_id,
            size,
            opcode,
        })
    }
}

pub struct MessageParser {
    buffer: Bytes, // Internal buffer for potentially fragmented messages
}

impl MessageParser {
    pub fn new() -> Self {
        MessageParser {
            buffer: Bytes::new(),
        }
    }

    pub fn append_data(&mut self, data: &[u8]) {
        // In a real scenario, this might be more sophisticated,
        // e.g. using a BytesMut and extending it.
        // For now, let's assume data is appended to some internal buffer.
        // This is a simplified append for now. A real BytesMut would be better.
        let mut new_buf = Vec::with_capacity(self.buffer.len() + data.len());
        new_buf.extend_from_slice(&self.buffer);
        new_buf.extend_from_slice(data);
        self.buffer = Bytes::from(new_buf);
        debug!("Appended {} bytes to parser buffer. Total size: {}", data.len(), self.buffer.len());
    }

    pub fn next_message(&mut self) -> Result<Option<RawMessage>, WaylandServerError> {
        if self.buffer.len() < MESSAGE_HEADER_SIZE {
            trace!("Parser buffer too small for a header ({} bytes). Waiting for more data.", self.buffer.len());
            return Ok(None); // Not enough data for even a header
        }

        // Try to parse the header without consuming the buffer yet, to check size
        let mut header_peek_buf = self.buffer.slice(0..MESSAGE_HEADER_SIZE);
        let _object_id_raw_peek = header_peek_buf.get_u32_le(); // object_id not needed for size check
        let size_opcode_raw_peek = header_peek_buf.get_u32_le();
        let size_peek = (size_opcode_raw_peek >> 16) as u16;

        if size_peek < (MESSAGE_HEADER_SIZE as u16) {
             error!("Invalid message size in peeked header: {} (must be at least {})", size_peek, MESSAGE_HEADER_SIZE);
            // This is a protocol error, potentially clear buffer or handle error state
            self.buffer.clear(); // Clear buffer on critical error
            return Err(WaylandServerError::Protocol(format!(
                "Invalid message size in peeked header: {}", size_peek
            )));
        }


        if self.buffer.len() < size_peek as usize {
            trace!("Partial message in buffer: Expected {} bytes, have {}. Waiting for more data.", size_peek, self.buffer.len());
            return Ok(None); // Not enough data for the full message as per header
        }

        // Now we know we have enough data for one full message, parse it properly.
        // This will advance self.buffer automatically.
        let header = MessageHeader::parse(&mut self.buffer)?;

        let content_size = header.size as usize - MESSAGE_HEADER_SIZE;
        let message_content = self.buffer.split_to(content_size); // Consumes from self.buffer

        debug!("Successfully parsed one message. Remaining buffer size: {}", self.buffer.len());
        Ok(Some(RawMessage {
            header,
            content: message_content,
        }))
    }

    // Basic argument deserializers (will be expanded)
    // These would typically be called on RawMessage.content by handler functions
    // based on the (object_id, opcode) which implies a specific signature.

    pub fn read_u32(buf: &mut Bytes) -> Result<u32, WaylandServerError> {
        if buf.remaining() < 4 {
            return Err(WaylandServerError::Protocol("Buffer too small for u32".to_string()));
        }
        Ok(buf.get_u32_le())
    }

    pub fn read_i32(buf: &mut Bytes) -> Result<i32, WaylandServerError> {
        if buf.remaining() < 4 {
            return Err(WaylandServerError::Protocol("Buffer too small for i32".to_string()));
        }
        Ok(buf.get_i32_le())
    }

    pub fn read_fixed(buf: &mut Bytes) -> Result<f64, WaylandServerError> {
        // Wayland fixed is 24.8 fixed-point, stored as i32
        let raw_fixed = Self::read_i32(buf)?;
        Ok(raw_fixed as f64 / 256.0)
    }

    pub fn read_string(buf: &mut Bytes) -> Result<String, WaylandServerError> {
        let len = Self::read_u32(buf)? as usize;
        if len == 0 {
             return Err(WaylandServerError::Protocol("String length cannot be zero".to_string()));
        }
        if buf.remaining() < len {
            return Err(WaylandServerError::Protocol(format!("Buffer too small for string of length {}", len)));
        }
        let str_bytes = buf.slice(0..len-1); // Exclude null terminator for String::from_utf8
        let s = String::from_utf8(str_bytes.to_vec())
            .map_err(|e| WaylandServerError::Protocol(format!("Invalid UTF-8 string: {}", e)))?;

        buf.advance(len-1); // Advance past string content

        // Null terminator check
        if buf.remaining() < 1 || buf.get_u8() != 0 {
            return Err(WaylandServerError::Protocol("String not null-terminated".to_string()));
        }

        // Padding: Wayland strings are padded to 32-bit alignment
        let padding = (4 - (len % 4)) % 4;
        if buf.remaining() < padding {
             return Err(WaylandServerError::Protocol("Buffer too small for string padding".to_string()));
        }
        buf.advance(padding);
        Ok(s)
    }

    pub fn read_object_id(buf: &mut Bytes) -> Result<ObjectId, WaylandServerError> {
        Ok(ObjectId::new(Self::read_u32(buf)?))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn create_test_header_bytes(object_id: u32, size: u16, opcode: u16) -> Bytes {
        let mut buf = BytesMut::with_capacity(MESSAGE_HEADER_SIZE);
        buf.extend_from_slice(&object_id.to_le_bytes());
        let size_opcode = ((size as u32) << 16) | (opcode as u32);
        buf.extend_from_slice(&size_opcode.to_le_bytes());
        buf.freeze()
    }

    #[test]
    fn test_message_header_parse_success() {
        let mut data = create_test_header_bytes(10, 12, 1); // Size 12, means 4 bytes of payload
        let header = MessageHeader::parse(&mut data).unwrap();
        assert_eq!(header.object_id.value(), 10);
        assert_eq!(header.size, 12);
        assert_eq!(header.opcode, 1);
        assert_eq!(data.remaining(), 0); // Header parsing consumes the header bytes
    }

    #[test]
    fn test_message_header_parse_too_small_buffer() {
        let mut data = Bytes::from_static(&[1, 0, 0, 0, 8, 0]); // 6 bytes, too small
        let result = MessageHeader::parse(&mut data);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Buffer too small for message header"));
        } else {
            panic!("Expected Protocol error");
        }
    }

    #[test]
    fn test_message_header_parse_invalid_size() {
        let mut data = create_test_header_bytes(10, MESSAGE_HEADER_SIZE as u16 - 1, 1); // Size less than header
        let result = MessageHeader::parse(&mut data);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Invalid message size in header"));
        } else {
            panic!("Expected Protocol error");
        }
    }

    #[test]
    fn test_message_parser_single_message() {
        let mut parser = MessageParser::new();
        let mut msg_bytes = BytesMut::new();
        // Header: obj_id=1, size=12, opcode=0
        msg_bytes.extend_from_slice(&1u32.to_le_bytes());
        msg_bytes.extend_from_slice(&((12u32 << 16) | 0u32).to_le_bytes());
        // Payload: u32 = 12345
        msg_bytes.extend_from_slice(&12345u32.to_le_bytes());

        parser.append_data(&msg_bytes.freeze());

        let raw_message_opt = parser.next_message().unwrap();
        assert!(raw_message_opt.is_some());
        let raw_message = raw_message_opt.unwrap();

        assert_eq!(raw_message.header.object_id.value(), 1);
        assert_eq!(raw_message.header.size, 12);
        assert_eq!(raw_message.header.opcode, 0);
        assert_eq!(raw_message.content.len(), 4); // 12 (total) - 8 (header) = 4

        let mut content_buf = raw_message.content.clone();
        assert_eq!(MessageParser::read_u32(&mut content_buf).unwrap(), 12345);

        assert!(parser.next_message().unwrap().is_none(), "Should be no more messages");
        assert_eq!(parser.buffer.len(), 0, "Parser buffer should be empty");
    }

    #[test]
    fn test_message_parser_fragmented_message() {
        let mut parser = MessageParser::new();
        // Message: obj_id=2, size=16, opcode=1, payload=two u32s (8 bytes)
        let mut full_msg = BytesMut::new();
        full_msg.extend_from_slice(&2u32.to_le_bytes()); // obj_id
        full_msg.extend_from_slice(&((16u32 << 16) | 1u32).to_le_bytes()); // size_opcode
        full_msg.extend_from_slice(&111u32.to_le_bytes()); // arg1
        full_msg.extend_from_slice(&222u32.to_le_bytes()); // arg2

        // Fragment 1: Header only
        parser.append_data(&full_msg.slice(0..MESSAGE_HEADER_SIZE));
        assert!(parser.next_message().unwrap().is_none(), "Should not parse with header only");
        assert_eq!(parser.buffer.len(), MESSAGE_HEADER_SIZE);

        // Fragment 2: Part of payload
        parser.append_data(&full_msg.slice(MESSAGE_HEADER_SIZE..MESSAGE_HEADER_SIZE + 4)); // first arg
        assert!(parser.next_message().unwrap().is_none(), "Should not parse with partial payload");
        assert_eq!(parser.buffer.len(), MESSAGE_HEADER_SIZE + 4);

        // Fragment 3: Rest of payload
        parser.append_data(&full_msg.slice(MESSAGE_HEADER_SIZE + 4..)); // second arg
        assert_eq!(parser.buffer.len(), 16);

        let raw_message_opt = parser.next_message().unwrap();
        assert!(raw_message_opt.is_some(), "Should parse complete message now");
        let raw_message = raw_message_opt.unwrap();

        assert_eq!(raw_message.header.object_id.value(), 2);
        assert_eq!(raw_message.header.size, 16);
        assert_eq!(raw_message.header.opcode, 1);
        assert_eq!(raw_message.content.len(), 8);

        let mut content_buf = raw_message.content.clone();
        assert_eq!(MessageParser::read_u32(&mut content_buf).unwrap(), 111);
        assert_eq!(MessageParser::read_u32(&mut content_buf).unwrap(), 222);

        assert!(parser.next_message().unwrap().is_none(), "Should be no more messages");
    }

    #[test]
    fn test_message_parser_multiple_messages() {
        let mut parser = MessageParser::new();
        let mut msg_bytes = BytesMut::new();
        // Msg 1: obj_id=1, size=12, opcode=0, payload=12345u32
        msg_bytes.extend_from_slice(&1u32.to_le_bytes());
        msg_bytes.extend_from_slice(&((12u32 << 16) | 0u32).to_le_bytes());
        msg_bytes.extend_from_slice(&12345u32.to_le_bytes());
        // Msg 2: obj_id=2, size=8, opcode=1 (no payload)
        msg_bytes.extend_from_slice(&2u32.to_le_bytes());
        msg_bytes.extend_from_slice(&((8u32 << 16) | 1u32).to_le_bytes());

        parser.append_data(&msg_bytes.freeze());

        // Parse first message
        let msg1 = parser.next_message().unwrap().unwrap();
        assert_eq!(msg1.header.object_id.value(), 1);
        assert_eq!(msg1.content.len(), 4);

        // Parse second message
        let msg2 = parser.next_message().unwrap().unwrap();
        assert_eq!(msg2.header.object_id.value(), 2);
        assert_eq!(msg2.content.len(), 0);

        assert!(parser.next_message().unwrap().is_none(), "Should be no more messages");
    }

    #[test]
    fn test_read_fixed_point() {
        // 1.0 in 24.8 fixed point is 256
        let mut data = Bytes::from_static(&256i32.to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data).unwrap(), 1.0);

        // -2.5 in 24.8 fixed point is -2.5 * 256 = -640
        let mut data_neg = Bytes::from_static(&(-640i32).to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data_neg).unwrap(), -2.5);

        // 0.5 in 24.8 fixed point is 128
        let mut data_half = Bytes::from_static(&128i32.to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data_half).unwrap(), 0.5);
    }

    #[test]
    fn test_read_string_simple() {
        let mut buf = BytesMut::new();
        let test_str = "hello";
        let len_with_null = test_str.len() + 1; // 5 + 1 = 6
        buf.extend_from_slice(&(len_with_null as u32).to_le_bytes()); // length
        buf.extend_from_slice(test_str.as_bytes()); // string data
        buf.put_u8(0); // null terminator
        // Padding: "hello\0" is 6 bytes. Padded to 8 bytes. (4 - (6 % 4)) % 4 = (4 - 2) % 4 = 2 bytes padding
        buf.put_u8(0); // padding
        buf.put_u8(0); // padding

        let mut data = buf.freeze();
        let s = MessageParser::read_string(&mut data).unwrap();
        assert_eq!(s, "hello");
        assert_eq!(data.remaining(), 0, "Buffer should be empty after reading string and padding");
    }

    #[test]
    fn test_read_string_exact_multiple_of_4_len() {
        let mut buf = BytesMut::new();
        let test_str = "bye"; // "bye\0" is 4 bytes, len = 4
        let len_with_null = test_str.len() + 1; // 3 + 1 = 4
        buf.extend_from_slice(&(len_with_null as u32).to_le_bytes());
        buf.extend_from_slice(test_str.as_bytes());
        buf.put_u8(0);
        // No padding needed as 4 bytes is already aligned.

        let mut data = buf.freeze();
        let s = MessageParser::read_string(&mut data).unwrap();
        assert_eq!(s, "bye");
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_string_invalid_utf8() {
        let mut buf = BytesMut::new();
        let len_with_null = 4u32; // len = 4 for "hi\xff\0" (invalid char + null)
        buf.extend_from_slice(&len_with_null.to_le_bytes());
        buf.extend_from_slice(&[b'h', b'i', 0xff]); // Invalid UTF-8 sequence
        buf.put_u8(0); // Null terminator
        // Padding: len is 4, so no padding needed.

        let mut data = buf.freeze();
        let result = MessageParser::read_string(&mut data);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Invalid UTF-8 string"));
        } else {
            panic!("Expected Protocol error for invalid UTF-8");
        }
    }
}
