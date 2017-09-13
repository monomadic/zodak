struct WavFile {
    reader: File,
    file_len: u32,
    // format_chunk: FormatChunk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InstrumentChunk {
    /// The unshifted note field has the same meaning as the sampler chunk's MIDI Unity Note which specifies the
    /// musical note at which the sample will be played at it's original sample rate (the sample rate specified
    /// in the format chunk). (0-127)
    unshifted_note: u8,
    fine_tune: u8,
    gain: u8,
    low_note: u8,
    high_note: u8,
    low_vel: u8,
    high_vel: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CuePoint {
    id: u32,
    position: u32,
    data_chunk_id: u32,
    chunk_start: u32,
    block_start: u32,
    sample_offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SamplerChunk {

    /// The manufacturer field specifies the MIDI Manufacturer's Association (MMA) Manufacturer
    /// code for the sampler intended to receive this file's waveform. Each manufacturer of a
    /// MIDI product is assigned a unique ID which identifies the company. If no particular
    /// manufacturer is to be specified, a value of 0 should be used.
    ///
    /// The value is stored with some extra information to enable translation to the value used in
    /// a MIDI System Exclusive transmission to the sampler. The high byte indicates the number of
    /// low order bytes (1 or 3) that are valid for the manufacturer code. For example, the value
    /// for Digidesign will be 0x01000013 (0x13) and the value for Microsoft will be 0x30000041
    /// (0x00, 0x00, 0x41). See the MIDI Manufacturers List for a list.
    manufacturer: u32,

    /// The product field specifies the MIDI model ID defined by the manufacturer corresponding to
    /// the Manufacturer field. Contact the manufacturer of the sampler to get the model ID. If no
    /// particular manufacturer's product is to be specified, a value of 0 should be used.
    product: u32,

    sample_period: u32,

    midi_unity_note: u32,

    midi_pitch_fraction: u32,

    smpte_format: u32,

    smpte_offset: u32,

    num_sample_loops: u32,

    sampler_data: u32,

}