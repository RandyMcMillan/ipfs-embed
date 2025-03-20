use std::io::Cursor;

use ciborium::ser::into_writer;

fn create_boxed_u8_slice(data: Vec<u8>) -> Box<[u8]> {
    data.into_boxed_slice()
}

fn serialize_my_vec(
    my_vec: &(i32, String),
) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
    let mut buffer = Vec::new();
    into_writer(my_vec, &mut buffer)?;
    Ok(buffer)
}

fn main() {
    let my_vec = (42, "hello".to_string());
    match serialize_my_vec(&my_vec) {
        Ok(serialized_data) => {
            println!("Serialized data: {:?}", serialized_data);
            let boxed_slice = serialized_data.into_boxed_slice();
            println!("Boxed slice: {:?}", boxed_slice);
        }
        Err(e) => {
            eprintln!("Serialization error: {}", e);
        }
    }
}

//fn main() {
//
//
//
//    let my_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
//    let boxed_slice: Box<[u8]> = create_boxed_u8_slice(my_vec);
//    println!("{:?}", boxed_slice);
//    ciborium::ser::into_writer(&my_vec, &mut
// boxed_slice).expect("Serialization of tuple");
//
//    // Tuple to be serialized
//    let tuple = ("Hello", "World");
//
//    // Serialize the tuple into a vector of bytes
//    let mut vec = Vec::new();
//    ciborium::ser::into_writer(&tuple, &mut vec).expect("Serialization of
// tuple");
//
//    //print the serialized representation
//    println!("Serialized CBOR: {:?}", vec);
//
//    // Deserialize the CBOR bytes back into a Rust tuple
//    let deserialized: (String, String) = ciborium::de::from_reader(&mut
// Cursor::new(vec))        .expect("Deserialized back into a Rust tuple");
//
//    // Assert equality (for demonstration, normally you'd use this
// deserialized data)    assert_eq!(deserialized, ("Hello".to_string(),
// "World".to_string()));    println!("Deserialized Data: {:?}", deserialized);
//}
