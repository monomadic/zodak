pub fn name_to_note_num(note_name:&str) -> u8 {

    /// breaks a string up into its components eg C#-1 into C, #, -, 1
    fn parse_note_name(note_name:&str) -> (String, String, i8) {
        use regex::Regex;

        let re = Regex::new(r"([A-Ga-g])([#bB]?)([\-]?[0-8])").expect("regular expression to parse");
        let capture = &re.captures(note_name).unwrap();

        (
            capture[1].to_string(),
            capture[2].to_string(),
            capture[3].parse::<i8>().expect("i8 to parse from str")
        )        
    }

    // println!("Extracted note {:?} from filename {:?}", parse_note_name(note_name), note_name);

    let (note, augment, octave) = parse_note_name(note_name);

    use std::collections::HashMap;
    let mut notes:HashMap<&str,u8> = HashMap::new();
    notes.insert("c", 0);
    notes.insert("d", 2);
    notes.insert("e", 4);
    notes.insert("f", 5);
    notes.insert("g", 7);
    notes.insert("a", 9);
    notes.insert("b", 11);

    let base_note = *notes.get(&note.clone().to_lowercase().to_string().as_str()).expect(format!("note to convert to midi number: {}", note).as_str());

    // adjust for octave
    let mut adjusted_note = base_note as i8 + (octave * 12);

    // account for sharps and flats
    match augment.as_str() {
        "#" => { adjusted_note = adjusted_note + 1 },
        "b"|"B" => { adjusted_note = adjusted_note - 1 },
        _ => {},
    }

    (adjusted_note as u8) + 24
}
