//! Asset format converters (PSX → modern formats)

use crate::{AssetError, Result};
use psxutils::{Tim, Vab, Vag};

/// Converts PSX assets to modern formats
pub struct AssetConverter;

impl AssetConverter {
    /// Convert TIM texture to PNG
    pub fn tim_to_png(tim_data: &[u8]) -> Result<Vec<u8>> {
        // Parse TIM
        let tim = Tim::parse(tim_data)
            .map_err(|e| AssetError::ConversionError(format!("Failed to parse TIM: {}", e)))?;

        // Convert to RGBA8
        let rgba = tim.to_rgba8().map_err(|e| {
            AssetError::ConversionError(format!("Failed to convert TIM to RGBA: {}", e))
        })?;

        // Create PNG using the image crate
        let width = tim.width() as u32;
        let height = tim.height() as u32;

        let img = image::RgbaImage::from_raw(width, height, rgba).ok_or_else(|| {
            AssetError::ConversionError("Failed to create image from RGBA data".to_string())
        })?;

        // Encode to PNG
        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )
        .map_err(|e| AssetError::ConversionError(format!("Failed to encode PNG: {}", e)))?;

        Ok(png_data)
    }

    /// Convert VAB audio to OGG
    ///
    /// Note: This is complex because VAB contains multiple samples.
    /// For now, we return an error directing users to extract individual VAGs.
    pub fn vab_to_ogg(_vab_data: &[u8]) -> Result<Vec<u8>> {
        // Parse VAB
        let _vab = Vab::parse(_vab_data)
            .map_err(|e| AssetError::ConversionError(format!("Failed to parse VAB: {}", e)))?;

        // TODO: Extract individual VAG samples and convert each to OGG
        // This requires an OGG encoder (like lewton or similar)
        Err(AssetError::ConversionError(
            "VAB to OGG conversion not yet implemented. Extract individual VAG samples instead."
                .to_string(),
        ))
    }

    /// Convert VAG audio sample to WAV
    pub fn vag_to_wav(vag_data: &[u8]) -> Result<Vec<u8>> {
        // Parse VAG
        let vag = Vag::parse(vag_data)
            .map_err(|e| AssetError::ConversionError(format!("Failed to parse VAG: {}", e)))?;

        // Decode ADPCM to PCM
        let pcm_data = vag.decode_to_pcm();

        // Build WAV file
        let wav_data = Self::build_wav(&pcm_data, vag.sample_rate, 1)?;

        Ok(wav_data)
    }

    /// Build a WAV file from PCM samples
    fn build_wav(samples: &[i16], sample_rate: u32, channels: u16) -> Result<Vec<u8>> {
        let mut wav = Vec::new();

        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
        let block_align = channels * bits_per_sample / 8;
        let data_size = (samples.len() * 2) as u32; // 2 bytes per i16

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        // PCM data
        for &sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        Ok(wav)
    }
}
