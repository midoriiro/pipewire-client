pub const PIPEWIRE_RUNTIME_DIR_ENVIRONMENT_KEY: &str = "PIPEWIRE_RUNTIME_DIR";
pub const PIPEWIRE_CORE_ENVIRONMENT_KEY: &str = "PIPEWIRE_CORE";
pub const PIPEWIRE_REMOTE_ENVIRONMENT_KEY: &str = "PIPEWIRE_REMOTE";
pub const XDG_RUNTIME_DIR_ENVIRONMENT_KEY: &str = "XDG_RUNTIME_DIR";
pub const PULSE_RUNTIME_PATH_ENVIRONMENT_KEY: &str = "PULSE_RUNTIME_PATH";
pub const PIPEWIRE_REMOTE_ENVIRONMENT_DEFAULT: &str = "pipewire-0";

pub const PIPEWIRE_CORE_SYNC_INITIALIZATION_SEQ :u32 = 0;
pub const PIPEWIRE_CORE_SYNC_CREATE_DEVICE_SEQ :u32 = 1;

pub const MEDIA_TYPE_PROPERTY_VALUE_AUDIO: &str = "Audio";
pub const MEDIA_CLASS_PROPERTY_KEY: &str = "media.class";
pub const MEDIA_CLASS_PROPERTY_VALUE_AUDIO_SOURCE: &str = "Audio/Source";
pub const MEDIA_CLASS_PROPERTY_VALUE_AUDIO_SINK: &str = "Audio/Sink";
pub const MEDIA_CLASS_PROPERTY_VALUE_AUDIO_DUPLEX: &str = "Audio/Duplex";
pub const MEDIA_CLASS_PROPERTY_VALUE_AUDIO_DEVICE: &str = "Audio/Device";
pub const MEDIA_CLASS_PROPERTY_VALUE_STREAM_OUTPUT_AUDIO: &str = "Stream/Output/Audio";
pub const MEDIA_CLASS_PROPERTY_VALUE_STREAM_INPUT_AUDIO: &str = "Stream/Input/Audio";
pub const METADATA_NAME_PROPERTY_KEY: &str = "metadata.name";
pub const METADATA_NAME_PROPERTY_VALUE_SETTINGS: &str = "settings";
pub const METADATA_NAME_PROPERTY_VALUE_DEFAULT: &str = "default";
pub const CLOCK_RATE_PROPERTY_KEY: &str = "clock.rate";
pub const CLOCK_QUANTUM_PROPERTY_KEY: &str = "clock.quantum";
pub const CLOCK_QUANTUM_MIN_PROPERTY_KEY: &str = "clock.min-quantum";
pub const CLOCK_QUANTUM_MAX_PROPERTY_KEY: &str = "clock.max-quantum";
pub const CLOCK_ALLOWED_RATES_PROPERTY_KEY: &str = "clock.allowed-rates";
pub const MONITOR_CHANNEL_VOLUMES_PROPERTY_KEY: &str = "monitor.channel-volumes";
pub const MONITOR_PASSTHROUGH_PROPERTY_KEY: &str = "monitor.passthrough";
pub const DEFAULT_AUDIO_SINK_PROPERTY_KEY: &str = "default.audio.sink";
pub const DEFAULT_AUDIO_SOURCE_PROPERTY_KEY: &str = "default.audio.source";
pub const AUDIO_POSITION_PROPERTY_KEY: &str = "audio.position";
pub const APPLICATION_NAME_PROPERTY_KEY: &str = "application.name";
pub const APPLICATION_NAME_PROPERTY_VALUE_WIRE_PLUMBER: &str = "WirePlumber";
pub const APPLICATION_NAME_PROPERTY_VALUE_PIPEWIRE_MEDIA_SESSION: &str = "pipewire-media-session";