use super::State;
use failure::Error;
use serde_json;
use std::fs;

pub fn save(data: &[Vec<State>], location: &'static str) -> Result<(), Error> {
    let j = serde_json::to_string(&flip_vector(data)).map_err(|e| SerializationError::Serialize(e))?;
    fs::write(location, j).map_err(|e| SerializationError::WriteToDisk(e))?;

    Ok(())
}

pub fn load(location: &'static str) -> Result<Vec<Vec<State>>, Error> {
    let json = fs::read_to_string(location).map_err(|e| SerializationError::ReadFromDisk(e))?;
    let prefab: Vec<Vec<State>> =
        serde_json::from_str(&json).map_err(|e| SerializationError::Deserialize(e))?;

    Ok(flip_vector(&prefab))
}

fn flip_vector(original: &[Vec<State>]) -> Vec<Vec<State>> {
    // iterate over the Vec:
    let mut ret: Vec<Vec<State>> = vec![];
    for _ in 0..original[0].len() {
        let mut this_one = Vec::with_capacity(original.len());
        for _ in 0..this_one.capacity() {
            this_one.push(State::Dead);
        }
        ret.push(this_one);
    }

    for (x, this_row) in original.iter().enumerate() {
        for (y, this_entity) in this_row.iter().enumerate() {
            ret[y][x] = *this_entity;
        }
    }

    ret
}

#[derive(Debug, Fail)]
enum SerializationError {
    #[fail(display = "Could not serialize.")]
    Serialize(#[fail(cause)] serde_json::error::Error),

    #[fail(display = "Could not deserialize.")]
    Deserialize(#[fail(cause)] serde_json::error::Error),

    #[fail(display = "Could not Write to Disk.")]
    WriteToDisk(#[fail(cause)] std::io::Error),

    #[fail(display = "Could not Read from Disk")]
    ReadFromDisk(#[fail(cause)] std::io::Error),
}
