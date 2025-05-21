use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::path::app_dir;
use tauri::Manager;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use dotenv::dotenv;
use std::env;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

// Import our network detector
use crate::network::NetworkDetector;

// Structure to hold the transcription result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
}

// Whisper API response structure
#[derive(Deserialize, Debug)]
struct WhisperAPIResponse {
    text: String,
}

// Gemini Live API response structures
#[derive(Deserialize, Debug)]
struct GeminiLiveResponse {
    text: Option<String>,
    error: Option<String>,
}

// Speech-to-text mode enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SttMode {
    Online,  // Use Gemini Live API
    Offline, // Use local Whisper model via Candle
    Auto,    // Automatically detect and choose
}

// Speech-to-text service
pub struct SpeechToTextService {
    recording: Arc<Mutex<bool>>,
    temp_dir: PathBuf,
    client: Client,
    openai_api_key: String,
    gemini_api_key: String,
    mode: Arc<Mutex<SttMode>>,
    network_detector: NetworkDetector,
}

impl SpeechToTextService {
    // Initialize the speech-to-text service
    pub fn new() -> Result<Self, String> {
        dotenv().ok();
        
        // Get API keys from environment variables
        let openai_api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY not found in environment variables".to_string())?;
        
        let gemini_api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| "GEMINI_API_KEY not found in environment variables".to_string())?;
        
        // Create temporary directory for audio files
        let temp_dir = std::env::temp_dir().join("plates_audio");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        }
        
        Ok(Self {
            recording: Arc::new(Mutex::new(false)),
            temp_dir,
            client: Client::new(),
            openai_api_key,
            gemini_api_key,
            mode: Arc::new(Mutex::new(SttMode::Auto)),
            network_detector: NetworkDetector::new(),
        })
    }
    
    // Set the STT mode
    pub fn set_mode(&self, mode: SttMode) {
        let mut current_mode = self.mode.lock().unwrap();
        *current_mode = mode;
    }
    
    // Get the current STT mode
    pub fn get_mode(&self) -> SttMode {
        *self.mode.lock().unwrap()
    }
    
    // Start recording audio
    pub fn start_recording(&self) -> Result<(), String> {
        let mut recording = self.recording.lock().unwrap();
        if *recording {
            return Err("Already recording".to_string());
        }
        
        *recording = true;
        println!("Recording started");
        
        // In a real implementation, this would start recording audio using a platform-specific API
        // For now, we'll just set the flag
        
        Ok(())
    }
    
    // Stop recording and save the audio to a file
    pub fn stop_recording(&self) -> Result<PathBuf, String> {
        let mut recording = self.recording.lock().unwrap();
        if !*recording {
            return Err("Not recording".to_string());
        }
        
        *recording = false;
        println!("Recording stopped");
        
        // In a real implementation, this would stop recording and save the audio to a file
        // For now, we'll create a dummy WAV file
        
        // Generate a timestamp for the filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let audio_path = self.temp_dir.join(format!("recording_{}.wav", timestamp));
        
        // Use the correct path to the dummy WAV file
        let dummy_wav_data = include_bytes!("../resources/dummy.wav");
        let mut file = fs::File::create(&audio_path)
            .map_err(|e| format!("Failed to create audio file: {}", e))?;
        
        file.write_all(dummy_wav_data)
            .map_err(|e| format!("Failed to write audio data: {}", e))?;
        
        Ok(audio_path)
    }
    
    // Transcribe audio using the appropriate method based on mode and network status
    pub async fn transcribe_audio(&self, audio_path: PathBuf) -> Result<TranscriptionResult, String> {
        println!("Transcribing audio from: {}", audio_path.display());
        
        // Check if the file exists
        if !audio_path.exists() {
            return Err(format!("Audio file not found: {}", audio_path.display()));
        }
        
        // Determine which mode to use
        let mode = match self.get_mode() {
            SttMode::Online => SttMode::Online,
            SttMode::Offline => SttMode::Offline,
            SttMode::Auto => {
                // Check network status
                if self.network_detector.is_online().await {
                    SttMode::Online
                } else {
                    SttMode::Offline
                }
            }
        };
        
        // Use the appropriate transcription method
        match mode {
            SttMode::Online => self.transcribe_with_gemini_live(audio_path).await,
            SttMode::Offline => self.transcribe_with_whisper_offline(audio_path).await,
        }
    }
    
    // Transcribe audio using OpenAI's Whisper API (online fallback)
    async fn transcribe_with_whisper_api(&self, audio_path: PathBuf) -> Result<TranscriptionResult, String> {
        // Read the file
        let file_data = fs::read(&audio_path)
            .map_err(|e| format!("Failed to read audio file: {}", e))?;
        
        // Create a multipart form with the audio file
        let part = Part::bytes(file_data)
            .file_name(audio_path.file_name().unwrap().to_string_lossy().to_string())
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to create multipart form: {}", e))?;
        
        let form = Form::new()
            .part("file", part)
            .text("model", "whisper-1")
            .text("language", "en");
        
        // Send the request to the Whisper API
        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Whisper API: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            return Err(format!("Whisper API error: {}", error_text));
        }
        
        // Parse the response
        let whisper_response: WhisperAPIResponse = response.json().await
            .map_err(|e| format!("Failed to parse Whisper API response: {}", e))?;
        
        // Return the transcription result
        Ok(TranscriptionResult {
            text: whisper_response.text,
            language: "en".to_string(),
        })
    }
    
    // Transcribe audio using Google's Gemini Live API
    async fn transcribe_with_gemini_live(&self, audio_path: PathBuf) -> Result<TranscriptionResult, String> {
        // Read the audio file
        let audio_data = fs::read(&audio_path)
            .map_err(|e| format!("Failed to read audio file: {}", e))?;
        
        // Connect to Gemini Live API WebSocket
        let ws_url = format!(
            "wss://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-live-001:streamGenerateContent?key={}",
            self.gemini_api_key
        );
        
        let (mut ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("Failed to connect to Gemini Live API: {}", e))?;
        
        // Configure the session
        let config = json!({
            "config": {
                "response_modalities": ["TEXT"],
                "system_instruction": {
                    "parts": [{
                        "text": "You are a speech-to-text transcription service. Transcribe the audio accurately."
                    }]
                }
            }
        });
        
        // Send configuration
        ws_stream.send(Message::Text(config.to_string()))
            .await
            .map_err(|e| format!("Failed to send configuration to Gemini Live API: {}", e))?;
        
        // Send audio data
        let audio_message = json!({
            "clientContent": {
                "turns": [{
                    "role": "user",
                    "parts": [{
                        "inline_data": {
                            "mime_type": "audio/wav",
                            "data": base64::encode(&audio_data)
                        }
                    }]
                }],
                "turnComplete": true
            }
        });
        
        ws_stream.send(Message::Text(audio_message.to_string()))
            .await
            .map_err(|e| format!("Failed to send audio data to Gemini Live API: {}", e))?;
        
        // Collect response
        let mut transcription = String::new();
        
        // Set a timeout for receiving messages
        let timeout = Duration::from_secs(10);
        let start_time = SystemTime::now();
        
        while let Some(msg) = ws_stream.next().await {
            // Check timeout
            if SystemTime::now().duration_since(start_time).unwrap() > timeout {
                break;
            }
            
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(response) = serde_json::from_str::<GeminiLiveResponse>(&text) {
                        if let Some(error) = response.error {
                            return Err(format!("Gemini Live API error: {}", error));
                        }
                        
                        if let Some(text_part) = response.text {
                            transcription.push_str(&text_part);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => return Err(format!("WebSocket error: {}", e)),
                _ => {}
            }
        }
        
        // Close the connection
        ws_stream.close(None).await.ok();
        
        if transcription.is_empty() {
            return Err("No transcription received from Gemini Live API".to_string());
        }
        
        Ok(TranscriptionResult {
            text: transcription,
            language: "en".to_string(),
        })
    }
    
    // Transcribe audio using local Whisper model via Candle (offline mode)
    async fn transcribe_with_whisper_offline(&self, audio_path: PathBuf) -> Result<TranscriptionResult, String> {
        // In a real implementation, this would use Candle to run Whisper locally
        // For now, we'll fall back to the OpenAI API if we have connectivity, or return a placeholder
        
        // Check if we have internet connectivity (for fallback)
        if self.network_detector.is_online().await {
            return self.transcribe_with_whisper_api(audio_path).await;
        }
        
        // Simulate local processing
        println!("Using offline Whisper model via Candle");
        
        // In a real implementation, this would load and run the Whisper model locally
        // For now, return a placeholder result
        Ok(TranscriptionResult {
            text: "[Offline transcription placeholder - would use Candle with Whisper model]".to_string(),
            language: "en".to_string(),
        })
    }
}

// Tauri command to initialize the STT system
#[tauri::command]
pub fn initialize_stt(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Create a new speech-to-text service
    let stt_service = SpeechToTextService::new()?;
    
    // Store the service in the app state
    app_handle.manage(stt_service);
    
    println!("STT system initialized");
    Ok(())
}

// Tauri command to set the STT mode
#[tauri::command]
pub fn set_stt_mode(app_handle: tauri::AppHandle, mode: SttMode) -> Result<(), String> {
    // Get the service from the app state
    let stt_service = app_handle.state::<SpeechToTextService>();
    
    // Set the mode
    stt_service.set_mode(mode);
    
    println!("STT mode set to: {:?}", mode);
    Ok(())
}

// Tauri command to get the current STT mode
#[tauri::command]
pub fn get_stt_mode(app_handle: tauri::AppHandle) -> Result<SttMode, String> {
    // Get the service from the app state
    let stt_service = app_handle.state::<SpeechToTextService>();
    
    // Get the mode
    Ok(stt_service.get_mode())
}

// Tauri command to start recording
#[tauri::command]
pub fn start_recording(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Get the service from the app state
    let stt_service = app_handle.state::<SpeechToTextService>();
    
    // Start recording
    stt_service.start_recording()
}

// Tauri command to stop recording and transcribe
#[tauri::command]
pub async fn stop_recording(app_handle: tauri::AppHandle) -> Result<TranscriptionResult, String> {
    // Get the service from the app state
    let stt_service = app_handle.state::<SpeechToTextService>();
    
    // Stop recording and get the audio file path
    let audio_path = stt_service.stop_recording()?;
    
    // Transcribe the audio
    stt_service.transcribe_audio(audio_path).await
}

// Tauri command to transcribe audio from a file path
#[tauri::command]
pub async fn transcribe_audio(app_handle: tauri::AppHandle, audio_path: String) -> Result<TranscriptionResult, String> {
    // Get the service from the app state
    let stt_service = app_handle.state::<SpeechToTextService>();
    
    // Transcribe the audio
    stt_service.transcribe_audio(PathBuf::from(audio_path)).await
}