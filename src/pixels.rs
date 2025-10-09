use serde_json::{self, Error};
use serde::{Deserialize, Serialize};
use serde;
use std::collections::HashMap;
use std::fs;


#[derive(Clone)]
#[derive(Serialize, Deserialize ,Debug)]
pub struct Pixel{
    pub x : i64,
    pub y : i64,
    #[serde(rename = "c")]
    pub color : String,
}
impl Pixel {
    pub fn new(x : i64, y : i64, color : String) -> Self{
        Self { x: x, y: y, color: color }
    }


    pub fn from_json(json_data : serde_json::Value) -> Result<Self, Error>{
        Ok(Self {
            x : serde_json::from_value(json_data["x"].clone())?,
            y : serde_json::from_value(json_data["y"].clone())?,
            
            color : serde_json::from_value(json_data["c"].clone())?,
        })
    }
    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).expect("Failed to convert to Value")
    }
}


#[derive(Clone)]
#[derive(Serialize, Debug)]
pub struct ChunkOfPixels {
    pub pixels : HashMap<(i64, i64), String>
}

impl ChunkOfPixels {
    pub fn new() -> Self {
        Self { pixels: HashMap::new() }
    }
    pub fn add(&mut self, pixel : Pixel) {
        self.pixels.insert((pixel.x, pixel.y), pixel.color);
    }
    pub fn get_all_pixels_as_vec(&self) -> Vec<Pixel> {
        let mut vec_of_pixels : Vec<Pixel> = Vec::new();
        for (k, v) in &self.pixels{
            vec_of_pixels.push(Pixel::new (k.0, k.1, v.to_string()));
        }
        vec_of_pixels
    }
    pub fn to_json(&self) -> String{
        serde_json::to_string(&self.get_all_pixels_as_vec()).expect("Failed to convert to Value")
    }
    pub fn from_json(json : &str) -> Result<Self, Error> {
        let pixles : Vec<Pixel> = serde_json::from_str(json)?;
        let mut self1 = Self::new();
        for i in pixles.into_iter() {
            self1.add(i);
        }
        Ok(self1)
    }
    pub fn save_on_disk(&self, path : &str) {
        fs::write(path, &self.to_json()).expect("Writing file error");
    }
    pub fn load_from_disk(path : &str) -> Self {
        Self::from_json(&String::from_utf8(fs::read(path)
        .expect("Reading file error"))
        .expect("Reading text from file error"))
        .expect("Failed converting json to ChunkOfPixels")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let pixel_json_str = "{\"x\": 10, \"y\": 5, \"c\": \"#000000\"}";
        let pixel = Pixel::from_json(serde_json::from_str(pixel_json_str).unwrap()).unwrap();
        assert_eq!(pixel.x, 10);
        assert_eq!(pixel.y, 5);
        assert_eq!(pixel.color, "#000000");
    }
    #[test]
    fn test2() {
        let pixel_json_str = "{\"x\": -12, \"y\": 6, \"c\": \"#078000\"}";
        let pixel = Pixel::from_json(serde_json::from_str(pixel_json_str).unwrap()).unwrap();
        assert_eq!(pixel.x, -12);
        assert_eq!(pixel.y, 6);
        assert_eq!(pixel.color, "#078000");
    }
    #[test]
    fn test3() {
        let pixel = Pixel::new(-34, -66, "#078280".to_string());
        assert_eq!(pixel.x, -34);
        assert_eq!(pixel.y, -66);
        assert_eq!(pixel.color, "#078280");
    }
}
