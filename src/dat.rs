use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// Initially generated using https://thomblin.github.io/xml_schema_generator/.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct Datafile {
    pub header: Header,
    pub game: Vec<Game>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Header {
    pub id: u32,
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Game {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub rom: Rom,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Rom {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@size")]
    pub size: u32,
    #[serde(rename = "@crc")]
    pub crc: String,
    #[serde(rename = "@md5")]
    pub md5: String,
    #[serde(rename = "@sha1")]
    pub sha1: String,
    #[serde(rename = "@sha256")]
    pub sha256: String,
    #[serde(rename = "@status")]
    pub status: Option<String>,
}

pub fn load_from_string(xml: String) -> Datafile {
    let dat: Datafile = serde_xml_rs::from_str(&xml).unwrap();
    dat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_game() {
        let xml = r#"<?xml version="1.0"?>
            <datafile xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://datomatic.no-intro.org/stuff https://datomatic.no-intro.org/stuff/schema_nointro_datfile_v3.xsd">
                <header>
                    <id>1</id>
                    <name>Test System</name>
                    <version>000000</version>
                </header>
                <game name="Test Game" id="0001">
                    <rom name="Test Game.ext" size="40976" crc="393a432f" md5="f94bb9bb55f325d9af8a0fff80b9376d" sha1="33d23c2f2cfa4c9efec87f7bc1321ce3ce6c89bd" sha256="0b3d9e1f01ed1668205bab34d6c82b0e281456e137352e4f36a9b2cfa3b66dea" status="verified" header="4E 45 53 1A 02 01 01 08 00 00 00 00 02 00 00 01"/>
                </game>
            </datafile>
            "#;

        let dat = load_from_string(xml.to_string());
        assert_eq!(dat.header.id, 1);
        assert_eq!(dat.header.name, "Test System");
        assert_eq!(dat.header.version, "000000");

        assert_eq!(dat.game.len(), 1);

        let game = &dat.game[0];
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.id, "0001");
        assert_eq!(game.rom.name, "Test Game.ext");
        assert_eq!(game.rom.size, 40976);
        assert_eq!(game.rom.crc, "393a432f");
        assert_eq!(game.rom.md5, "f94bb9bb55f325d9af8a0fff80b9376d");
        assert_eq!(game.rom.sha1, "33d23c2f2cfa4c9efec87f7bc1321ce3ce6c89bd");
        assert_eq!(
            game.rom.sha256,
            "0b3d9e1f01ed1668205bab34d6c82b0e281456e137352e4f36a9b2cfa3b66dea"
        );
    }

    #[test]
    fn parse_two_games() {
        let xml = r#"<?xml version="1.0"?>
            <datafile xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://datomatic.no-intro.org/stuff https://datomatic.no-intro.org/stuff/schema_nointro_datfile_v3.xsd">
                <header>
                    <id>1</id>
                    <name>Test System</name>
                    <version>000000</version>
                </header>
                <game name="Test Game" id="0001">
                    <rom name="Test Game.ext" size="40976" crc="393a432f" md5="f94bb9bb55f325d9af8a0fff80b9376d" sha1="33d23c2f2cfa4c9efec87f7bc1321ce3ce6c89bd" sha256="0b3d9e1f01ed1668205bab34d6c82b0e281456e137352e4f36a9b2cfa3b66dea" status="verified" header="4E 45 53 1A 02 01 01 08 00 00 00 00 02 00 00 01"/>
                </game>
                <game name="Test Game 2" id="0002">
                    <rom name="Test Game 2.ext" size="262160" crc="43507232" md5="55f7030dc6173f2a0145a97f369f49f4" sha1="d3f8cfd7822c1cf634c2132009f877b44244850f" sha256="41300bc4942a8a4f9b53148b404dd5cae3dd708ebdd9b617888d290a51a83e43" status="verified" header="4E 45 53 1A 08 10 40 08 00 00 07 00 00 00 00 01"/>
                </game>
            </datafile>
            "#;

        let dat = load_from_string(xml.to_string());
        assert_eq!(dat.header.id, 1);
        assert_eq!(dat.header.name, "Test System");
        assert_eq!(dat.header.version, "000000");

        assert_eq!(dat.game.len(), 2);

        let game1 = &dat.game[0];
        assert_eq!(game1.name, "Test Game");
        assert_eq!(game1.rom.name, "Test Game.ext");

        let game2 = &dat.game[1];
        assert_eq!(game2.name, "Test Game 2");
        assert_eq!(game2.id, "0002");
        assert_eq!(game2.rom.name, "Test Game 2.ext");
        assert_eq!(game2.rom.size, 262160);
        assert_eq!(game2.rom.crc, "43507232");
        assert_eq!(game2.rom.md5, "55f7030dc6173f2a0145a97f369f49f4");
        assert_eq!(game2.rom.sha1, "d3f8cfd7822c1cf634c2132009f877b44244850f");
        assert_eq!(
            game2.rom.sha256,
            "41300bc4942a8a4f9b53148b404dd5cae3dd708ebdd9b617888d290a51a83e43"
        );
    }
}
