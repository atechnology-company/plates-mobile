use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::path::app_dir;
use tauri::Manager;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use dotenv::dotenv;
use std::env;

// Structure to hold the transcription result
#[derive(Serialize, Deserialize, Debug)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
}

// Whisper API response structure
#[derive(Deserialize, Debug)]
struct WhisperAPIResponse {
    text: String,
}

// Audio recorder state
pub struct AudioRecorder {
    recording: Arc<Mutex<bool>>,
    temp_dir: PathBuf,
    client: Client,
    api_key: String,
}

impl AudioRecorder {
    // Initialize the audio recorder
    pub fn new() -> Result<Self, String> {
        dotenv().ok();
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY not found in environment variables".to_string())?;
        
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
            api_key,
        })
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
    
    // Transcribe audio using OpenAI's Whisper API
    pub async fn transcribe_audio(&self, audio_path: PathBuf) -> Result<TranscriptionResult, String> {
        println!("Transcribing audio from: {}", audio_path.display());
        
        // Check if the file exists
        if !audio_path.exists() {
            return Err(format!("Audio file not found: {}", audio_path.display()));
        }
        
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
            .header("Authorization", format!("Bearer {}", self.api_key))
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
}

// Tauri command to initialize the STT system
#[tauri::command]
pub fn initialize_stt(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Create a new audio recorder
    let recorder = AudioRecorder::new()?;
    
    // Store the recorder in the app state
    app_handle.manage(recorder);
    
    println!("STT system initialized");
    Ok(())
}

// Tauri command to start recording
#[tauri::command]
pub fn start_recording(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Get the recorder from the app state
    let recorder = app_handle.state::<AudioRecorder>();
    
    // Start recording
    recorder.start_recording()
}

// Tauri command to stop recording and transcribe
#[tauri::command]
pub async fn stop_recording(app_handle: tauri::AppHandle) -> Result<TranscriptionResult, String> {
    // Get the recorder from the app state
    let recorder = app_handle.state::<AudioRecorder>();
    
    // Stop recording and get the audio file path
    let audio_path = recorder.stop_recording()?;
    
    // Transcribe the audio
    recorder.transcribe_audio(audio_path).await
}

// Tauri command to transcribe audio from a file path
#[tauri::command]
pub async fn transcribe_audio(app_handle: tauri::AppHandle, audio_path: String) -> Result<TranscriptionResult, String> {
    // Get the recorder from the app state
    let recorder = app_handle.state::<AudioRecorder>();
    
    // Transcribe the audio
    recorder.transcribe_audio(PathBuf::from(audio_path)).await
}