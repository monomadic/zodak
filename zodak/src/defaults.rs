use std::fs;
use std::io;

pub struct InstrumentDefaults {
    regions: Vec<RegionDefaults>,
}

pub struct RegionDefaults {
    file: String,
    pitch_keycenter: u32,
    lokey: u32,
    hikey: u32,
    lovel: u32,
    hivel: u32,
}

impl InstrumentDefaults {
    pub fn new() -> Self {
        InstrumentDefaults {
            regions: Vec::new(),
        }
    }

    pub fn parse_sfz(file: fs::File) -> io::Result<Self> {
        println!("Parsing: {:?}", file);

        Ok(Self {
            regions: Vec::new(),
        })
    }
}
