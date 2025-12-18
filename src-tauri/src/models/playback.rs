use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub enum AudioQuality {
    /// Low quality (typically 96 kbps AAC)
    Low,
    /// High quality (typically 320 kbps AAC)
    High,
    /// Lossless quality (FLAC, typically 44.1 kHz / 16-bit)
    Lossless,
    /// Hi-Res Lossless quality (FLAC, up to 192 kHz / 24-bit)
    HiResLossless,
}

impl From<tidalrs::AudioQuality> for AudioQuality {
    fn from(value: tidalrs::AudioQuality) -> Self {
        match value {
            tidalrs::AudioQuality::Low => AudioQuality::Low,
            tidalrs::AudioQuality::High => AudioQuality::High,
            tidalrs::AudioQuality::Lossless => AudioQuality::Lossless,
            tidalrs::AudioQuality::HiResLossless => AudioQuality::HiResLossless,
        }
    }
}

