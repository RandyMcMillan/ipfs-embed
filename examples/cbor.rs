use std::io::Cursor;

fn main() {
    // Tuple to be serialized
    let tuple = ("Hello", "World");

    // Serialize the tuple into a vector of bytes
    let mut vec = Vec::new();
    ciborium::ser::into_writer(&tuple, &mut vec).expect("Serialization of tuple");

    //print the serialized representation
    println!("Serialized CBOR: {:?}", vec);

    // Deserialize the CBOR bytes back into a Rust tuple
    let deserialized: (String, String) = ciborium::de::from_reader(&mut Cursor::new(vec))
        .expect("Deserialized back into a Rust tuple");

    // Assert equality (for demonstration, normally you'd use this deserialized data)
    assert_eq!(deserialized, ("Hello".to_string(), "World".to_string()));
    println!("Deserialized Data: {:?}", deserialized);
}
