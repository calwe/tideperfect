use serde::{Deserialize, Serialize};
use specta::Type;
use structural_convert::StructuralConvert;
use tideperfect::services::player::CommandDevice;

#[derive(Debug, Serialize, Deserialize, Clone, Type, StructuralConvert)]
#[convert(from(CommandDevice))]
#[convert(into(CommandDevice))]
pub struct CommandDeviceDTO {
    name: String,
    id: String,
}

