// novade-system/src/input/voice_capture_service.rs
//! Service for capturing voice input, potentially with hotword detection.
use crate::error::SystemError;
use std::sync::mpsc::Receiver; // For streaming audio data or events

#[derive(Debug)]
pub enum VoiceInputEvent {
    HotwordDetected(String),    // Name of the detected hotword
    VoiceDataSegment(Vec<u8>),  // Raw audio data (e.g., PCM chunks)
    SpeechRecognized(String),   // If local VAD/STT is performed (less likely here, more in domain)
    Error(String),              // Errors from the capture process
    Started,                    // Indicates capture has started
    Stopped,                    // Indicates capture has stopped
}

pub trait VoiceCaptureService: Send + Sync {
    /// Starts voice capture.
    /// This might involve initializing audio input, listening for a hotword,
    /// or beginning continuous recording based on implementation.
    ///
    /// # Returns
    /// A `Receiver` for `VoiceInputEvent`s, allowing the caller to react to
    /// hotwords, receive audio data, or handle errors.
    fn start_capture(&self) -> Result<Receiver<VoiceInputEvent>, SystemError>;

    /// Stops active voice capture.
    /// Releases audio resources and terminates any ongoing processing.
    fn stop_capture(&self) -> Result<(), SystemError>;

    /// Gets the current status of the voice capture service.
    /// (e.g., "idle", "listening_for_hotword", "recording", "processing", "error").
    fn get_status(&self) -> Result<String, SystemError>;

    // TODO: Consider methods for:
    // - Listing available microphones or input sources.
    // - Setting preferred input device.
    // - Querying supported hotwords if dynamically configurable.
}

// TODO: Assistant Integration: This service will provide the raw voice input (or pre-processed text/events)
// to the Domain Layer's AIInteractionLogicService.
// TODO: Requires integration with audio capture backends (e.g., PipeWire, ALSA, PulseAudio).
// TODO: Hotword detection (e.g., using Porcupine, Snowboy, or custom models) is a key feature.
// TODO: Voice Activity Detection (VAD) might be implemented here to reduce data flow.
// TODO: Define SystemError variants for voice capture specific errors (e.g., DeviceNotFound, CaptureFailed, HotwordInitFailed).
