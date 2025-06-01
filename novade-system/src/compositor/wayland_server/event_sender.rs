// In novade-system/src/compositor/wayland_server/event_sender.rs
use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::protocol::{ObjectId, MESSAGE_HEADER_SIZE};
use crate::compositor::wayland_server::protocols::core::wl_callback::CallbackDoneEvent; // Example event
use crate::compositor::wayland_server::protocols::core::wl_registry::RegistryGlobalEvent; // Example event
use crate::compositor::wayland_server::protocols::core::wl_shm::ShmFormatEvent; // Example event
use bytes::{BytesMut, BufMut};
use nix::sys::socket::{sendmsg, ControlMessage, MsgFlags};
use nix::sys::uio::IoVec;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use tokio::net::UnixStream as TokioUnixStream;
use tokio::sync::Mutex; // For write access to streams
use tracing::{debug, error, warn, trace};
use std::fmt; // Required for the Debug derive on generic E in send_event


// Trait for types that can be serialized as Wayland event arguments
pub trait SerializeWaylandArgs {
    fn serialize(&self, buf: &mut BytesMut, fds: &mut Vec<RawFd>) -> Result<(), WaylandServerError>;
}

// Helper functions for writing Wayland data types

pub fn write_u32(buf: &mut BytesMut, val: u32) {
    buf.put_u32_le(val);
}

pub fn write_i32(buf: &mut BytesMut, val: i32) {
    buf.put_i32_le(val);
}

pub fn write_fixed(buf: &mut BytesMut, val: f64) {
    let scaled_val = (val * 256.0).round() as i32;
    write_i32(buf, scaled_val);
}

pub fn write_string(buf: &mut BytesMut, val: &str) -> Result<(), WaylandServerError> {
    let len_with_null = val.as_bytes().len() + 1;
    if len_with_null == 0 {
         return Err(WaylandServerError::Protocol("String length with null cannot be zero".to_string()));
    }
    write_u32(buf, len_with_null as u32);
    buf.put_slice(val.as_bytes());
    buf.put_u8(0); // Null terminator

    let padding = (4 - (len_with_null % 4)) % 4;
    for _ in 0..padding {
        buf.put_u8(0);
    }
    Ok(())
}

pub fn write_object_id(buf: &mut BytesMut, id: ObjectId) {
    write_u32(buf, id.value());
}

pub fn write_array(buf: &mut BytesMut, data: &[u8]) -> Result<(), WaylandServerError> {
    write_u32(buf, data.len() as u32);
    buf.put_slice(data);
    let padding = (4 - (data.len() % 4)) % 4;
    for _ in 0..padding {
        buf.put_u8(0);
    }
    Ok(())
}

pub fn write_fd_placeholder(buf: &mut BytesMut, _fd_to_be_sent: RawFd) {
    write_u32(buf, 0);
}


// Example event struct implementations for SerializeWaylandArgs
impl SerializeWaylandArgs for CallbackDoneEvent {
    fn serialize(&self, buf: &mut BytesMut, _fds: &mut Vec<RawFd>) -> Result<(), WaylandServerError> {
        write_u32(buf, self.callback_data);
        Ok(())
    }
}

impl SerializeWaylandArgs for RegistryGlobalEvent {
    fn serialize(&self, buf: &mut BytesMut, _fds: &mut Vec<RawFd>) -> Result<(), WaylandServerError> {
        write_u32(buf, self.name);
        write_string(buf, &self.interface)?;
        write_u32(buf, self.version);
        Ok(())
    }
}

impl SerializeWaylandArgs for ShmFormatEvent {
     fn serialize(&self, buf: &mut BytesMut, _fds: &mut Vec<RawFd>) -> Result<(), WaylandServerError> {
        write_u32(buf, self.format_code as u32);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EventSender {
    // For now, EventSender is stateless regarding client streams.
    // It takes the stream as an argument in send_event.
}

impl EventSender {
    pub fn new() -> Self {
        EventSender {}
    }

    pub async fn send_event<E: SerializeWaylandArgs + fmt::Debug>(
        &self,
        stream: &TokioUnixStream,
        target_object_id: ObjectId,
        opcode: u16,
        event_data: E,
        mut fds_to_send: Vec<RawFd>,
    ) -> Result<(), WaylandServerError> {

        let mut arg_buffer = BytesMut::new();
        // Pass fds_to_send mutably so serialize can add FDs if the event itself contains them
        event_data.serialize(&mut arg_buffer, &mut fds_to_send)?;

        let total_payload_size = arg_buffer.len();
        let message_size = MESSAGE_HEADER_SIZE + total_payload_size;

        if message_size > u16::MAX as usize {
            error!("Event message size {} exceeds u16::MAX", message_size);
            return Err(WaylandServerError::Protocol("Event message too large".to_string()));
        }

        let mut header_buf = BytesMut::with_capacity(MESSAGE_HEADER_SIZE);
        write_object_id(&mut header_buf, target_object_id);
        let size_opcode = ((message_size as u32) << 16) | (opcode as u32);
        write_u32(&mut header_buf, size_opcode);

        let iov = [
            IoVec::from_slice(&header_buf),
            IoVec::from_slice(&arg_buffer),
        ];

        let cmsgs_owned: Vec<ControlMessageOwned> = if !fds_to_send.is_empty() {
             vec![ControlMessageOwned::ScmRights(fds_to_send)]
        } else {
            vec![]
        };
        // Convert ControlMessageOwned to ControlMessage for sendmsg
        let cmsgs_ref: Vec<ControlMessage> = cmsgs_owned.iter().map(|cm_owned| cm_owned.into()).collect();


        trace!(
            "Sending event: TargetObj={}, Opcode={}, Size={}, NumFDs={}. EventData: {:?}",
            target_object_id.value(), opcode, message_size, cmsgs_ref.len(), event_data // Use cmsgs_ref.len()
        );

        stream.writable().await.map_err(|e| WaylandServerError::Io(e))?;
        let send_result = stream.try_io(tokio::io::Interest::WRITABLE, |std_stream| {
            sendmsg(std_stream.as_raw_fd(), &iov, &cmsgs_ref, MsgFlags::empty(), None)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("sendmsg nix error: {}", e)))
        }).await;

        match send_result {
            Ok(Ok(bytes_sent)) => {
                if bytes_sent < message_size {
                    warn!("Partial send: sent {} of {} bytes for event. TODO: Handle this.", bytes_sent, message_size);
                }
                debug!("Successfully sent {} bytes for event (TargetObj={}, Opcode={})", bytes_sent, target_object_id.value(), opcode);
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Failed to send event (sendmsg error): {}", e);
                Err(WaylandServerError::Io(e))
            }
            Err(e) => {
                 error!("Failed to send event (try_io error): {}", e);
                Err(WaylandServerError::Io(e))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::protocols::core::wl_callback::EVT_DONE_OPCODE; // For test
    use bytes::Buf; // For reading from BytesMut in tests
    use tokio::net::UnixListener as TokioListener;
    use tempfile::tempdir;

    async fn create_event_stream_pair() -> (TokioUnixStream, TokioUnixStream) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_event_sending.sock");
        let listener = TokioListener::bind(&socket_path).await.unwrap();
        let client_fut = TokioUnixStream::connect(&socket_path);
        let server_fut = listener.accept();
        let (client_res, server_res) = tokio::join!(client_fut, server_fut);
        let client_stream = client_res.unwrap();
        let (server_stream_conn, _addr) = server_res.unwrap();
        (client_stream, server_stream_conn)
    }

    async fn read_test_message(stream: &TokioUnixStream) -> Result<(BytesMut, Vec<RawFd>), String> {
        let mut header_buf = BytesMut::with_capacity(MESSAGE_HEADER_SIZE);
        // For this test, we simplify FD reading. A real client would use recvmsg.
        // We primarily test the byte stream here. FD sending is tested by ensuring sendmsg is called correctly.
        let mut received_fds = Vec::new();

        stream.readable().await.map_err(|e| format!("readable header: {}",e))?;
        header_buf.resize(MESSAGE_HEADER_SIZE, 0); // Pre-allocate for read_exact style
        stream.try_io(tokio::io::Interest::READABLE, |std_s| {
            match std_s.peek(&mut header_buf) { // Peek to not consume yet, check if enough data
                Ok(n) if n == MESSAGE_HEADER_SIZE => Ok(()),
                Ok(n) => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, format!("peeked {} bytes for header", n))),
                Err(e) => Err(e),
            }
        }).await.map_err(|e| format!("try_io peek header: {}",e))?.map_err(|e| format!("peek header: {}",e))?;

        let mut temp_header_buf = header_buf.clone(); // Clone for reading values without advancing original
        let object_id = ObjectId::new(temp_header_buf.get_u32_le());
        let size_opcode = temp_header_buf.get_u32_le();
        let size = (size_opcode >> 16) as u16;
        let _opcode = (size_opcode & 0xFFFF) as u16; // _opcode as it's not used further in this helper

        if size < MESSAGE_HEADER_SIZE as u16 {
            return Err(format!("Invalid size in received header: {}", size));
        }
        let body_len = size as usize - MESSAGE_HEADER_SIZE;
        let mut full_message_buf = BytesMut::with_capacity(size as usize);
        full_message_buf.resize(size as usize, 0);

        stream.readable().await.map_err(|e| format!("readable full message: {}",e))?;
        stream.try_io(tokio::io::Interest::READABLE, |std_s| {
             std_s.read_exact(&mut full_message_buf)
        }).await.map_err(|e| format!("try_io read_exact: {}",e))?.map_err(|e| format!("read_exact full message: {}",e))?;

        Ok((full_message_buf, received_fds))
    }

    #[test]
    fn test_wayland_arg_serialization() {
        let mut buf = BytesMut::new();
        write_u32(&mut buf, 123);
        assert_eq!(buf.as_ref(), &123u32.to_le_bytes());
        buf.clear();

        write_string(&mut buf, "hello").unwrap();
        let expected_str: [u8; 12] = [6,0,0,0, b'h',b'e',b'l',b'l',b'o',0, 0,0];
        assert_eq!(buf.as_ref(), &expected_str);
        buf.clear();

        write_fixed(&mut buf, 1.5);
        assert_eq!(buf.as_ref(), &384i32.to_le_bytes());
        buf.clear();

        let array_data = [1u8, 2, 3, 4, 5];
        write_array(&mut buf, &array_data).unwrap();
        let mut expected_array = BytesMut::new();
        expected_array.put_u32_le(5);
        expected_array.put_slice(&array_data);
        expected_array.put_bytes(0, 3);
        assert_eq!(buf.as_ref(), expected_array.as_ref());
    }

    #[tokio::test]
    async fn test_event_sender_send_simple_event_no_fds() {
        let (client_stream_to_send_on, server_stream_to_receive) = create_event_stream_pair().await;
        let event_sender = EventSender::new();

        let target_obj_id = ObjectId::new(10);
        let opcode = EVT_DONE_OPCODE;
        let event_payload = CallbackDoneEvent { callback_data: 12345 };

        let result = event_sender.send_event(
            &client_stream_to_send_on,
            target_obj_id,
            opcode,
            event_payload,
            vec![]
        ).await;
        assert!(result.is_ok(), "send_event failed: {:?}", result.err());

        let (mut received_data, _fds) = read_test_message(&server_stream_to_receive).await.unwrap();

        assert_eq!(received_data.get_u32_le(), target_obj_id.value());
        let size_opcode = received_data.get_u32_le();
        let total_size = (size_opcode >> 16) as usize;
        assert_eq!((size_opcode & 0xFFFF) as u16, opcode);

        let expected_arg_size = 4;
        assert_eq!(total_size, MESSAGE_HEADER_SIZE + expected_arg_size);

        assert_eq!(received_data.get_u32_le(), 12345);
    }

    #[tokio::test]
    async fn test_event_sender_send_event_with_fds_placeholder() {
        let (client_stream_to_send_on, server_stream_to_receive) = create_event_stream_pair().await;
        let event_sender = EventSender::new();

        #[derive(Debug)]
        struct EventWithFd { fd_val: RawFd }
        impl SerializeWaylandArgs for EventWithFd {
            fn serialize(&self, buf: &mut BytesMut, fds: &mut Vec<RawFd>) -> Result<(), WaylandServerError> {
                write_fd_placeholder(buf, self.fd_val);
                fds.push(self.fd_val);
                Ok(())
            }
        }

        let mut pipe_fds = [-1; 2];
        nix::unistd::pipe(&mut pipe_fds).expect("Failed to create pipe for test FD");
        let fd_to_send = pipe_fds[0];

        let target_obj_id = ObjectId::new(20);
        let opcode = 0;
        let event_payload = EventWithFd { fd_val: fd_to_send };
        let initial_fds = vec![];

        let result = event_sender.send_event(
            &client_stream_to_send_on,
            target_obj_id,
            opcode,
            event_payload,
            initial_fds
        ).await;
        assert!(result.is_ok(), "send_event with FD failed: {:?}", result.err());

        // To fully test FD receipt, server_stream_to_receive should use a recvmsg-based reader.
        // The existing socket::read_from_stream_with_fds can be used here.
        let mut byte_buf = BytesMut::new();
        let mut fd_buf = Vec::new();
        let _bytes_read = crate::compositor::wayland_server::socket::read_from_stream_with_fds(&server_stream_to_receive, &mut byte_buf, &mut fd_buf).await.unwrap();

        assert_eq!(fd_buf.len(), 1, "Should have received one FD");
        // Further checks on byte_buf would be similar to test_event_sender_send_simple_event_no_fds
        // to ensure header and placeholder are correct.
        let mut header_check_buf = byte_buf.split_to(MESSAGE_HEADER_SIZE);
        assert_eq!(header_check_buf.get_u32_le(), target_obj_id.value());
        let size_opcode = header_check_buf.get_u32_le();
        assert_eq!((size_opcode & 0xFFFF) as u16, opcode);
        let placeholder_val = byte_buf.get_u32_le();
        assert_eq!(placeholder_val, 0); // Check placeholder

        nix::unistd::close(pipe_fds[0]).ok();
        nix::unistd::close(pipe_fds[1]).ok();
        if !fd_buf.is_empty() {
            nix::unistd::close(fd_buf[0]).ok();
        }
    }
}
