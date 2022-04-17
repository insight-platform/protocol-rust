use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub mod message_builder;
pub mod protocol;

mod utils;

pub const TRACK_NAME_MAX_LENGTH: usize = 16;
pub const STREAM_NAME_MAX_LENGTH: usize = 16;

pub type StreamName = [u8; STREAM_NAME_MAX_LENGTH];
pub type TrackName = [u8; TRACK_NAME_MAX_LENGTH];
pub type ElementType = i16;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum TrackType {
    Video,
    Meta,
    NotImplemented,
}


impl Default for TrackType {
    fn default() -> Self {
        TrackType::Video
    }
}


#[derive(PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Payload {
    pub data: Vec<u8>,
    pub attributes: HashMap<String, String>,
}

pub fn get_empty_track_name() -> TrackName {
    [0; TRACK_NAME_MAX_LENGTH]
}

pub fn pack_stream_name(stream_name: &Uuid) -> StreamName {
    stream_name.as_bytes().clone()
}

pub fn pack_track_name(track_name: &String) -> std::result::Result<TrackName, String> {
    let name_len = track_name.len();
    if name_len > TRACK_NAME_MAX_LENGTH {
        let error = format!(
            "Invalid track name length. Must me less than {} characters.",
            TRACK_NAME_MAX_LENGTH
        );
        Err(error)
    } else {
        let mut buf: TrackName = get_empty_track_name();
        buf[..name_len].clone_from_slice(track_name.as_bytes());
        Ok(buf)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Copy, Clone)]
pub struct TrackInfo {
    pub track_type: TrackType,
    pub track_name: TrackName,
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use uuid::Uuid;

    use crate::message_builder::MessageBuilder;
    use crate::{pack_stream_name, pack_track_name, Payload, TrackType};
    use crate::message_builder::services::ffprobe::*;
    use crate::message_builder::media_store::*;
    use crate::protocol::Message;

    fn get_avro_path() -> String {
        let mut base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base_dir.push("API/avro/protocol");
        return String::from(base_dir.to_str().unwrap());
    }

    #[test]
    fn test_load_schemas() {
        let mb = MessageBuilder::new(get_avro_path().as_str());
        for s in MessageBuilder::schema_files() {
            let _s = mb.get_schema(&s.1);
        }
    }

    #[test]
    fn test_notify_message() {
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let saved_ms = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
            .unwrap();

        let _m = build_notify_message(&mb,
                                      stream_name,
                                      &TrackType::Meta,
                                      track_name,
                                      0,
                                      saved_ms,
                                      Some(10),
        );

        let m =
            build_notify_message(&mb, stream_name, &TrackType::Meta, track_name, 0, saved_ms, None);
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::NotifyMessage { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_unit_element_message() {
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let mut attributes: HashMap<String, String> = HashMap::default();
        attributes.insert(String::from("key1"), String::from("value1"));
        attributes.insert(String::from("key2"), String::from("value2"));

        let m = build_unit_element_message(&mb,
                                           stream_name,
                                           &TrackType::Meta,
                                           track_name,
                                           0,
                                           1,
                                           &vec![0, 0],
                                           &HashMap::default(),
                                           false,
        );

        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::UnitElementMessage { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_stream_tracks_request() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_stream_tracks_request(&mb, 0, String::from("/ab/c"), stream_name);

        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTracksRequest { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_stream_tracks_response() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let _m = build_stream_tracks_response(&mb, 0, stream_name, &vec![]);
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let track_name2 = pack_track_name(&String::from("test2")).unwrap();
        let m = build_stream_tracks_response(&mb,
                                             0,
                                             stream_name,
                                             &vec![
                                                 (track_name, TrackType::Meta),
                                                 (track_name2, TrackType::Video),
                                             ],
        );

        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTracksResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_stream_track_unit_element_request() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_stream_track_unit_elements_request(
            &mb,
            0,
            String::from("/ab/c"),
            stream_name,
            &TrackType::Meta,
            track_name,
            0,
            100,
        );
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTrackUnitElementsRequest { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_ping_request() {
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_ping_request_response(&mb, 0, String::from("/ab/c"), false);
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::PingRequestResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_ping_response() {
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_ping_request_response(&mb, 0, String::from("/ab/c"), true);
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::PingRequestResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_services_ffprobe_request() {
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_services_ffprobe_request(&mb, 0, String::from("/ab/c"),
                                               String::from("abc"),
                                               HashMap::from([("a".to_string(), "b".to_string())]));
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::ServicesFFProbeRequest { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_services_ffprobe_response() {
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_services_ffprobe_response(&mb, 0,
                                                &ServicesFFProbeResponseType::Accepted,
                                                10,
                                                vec![
                                                    HashMap::from([("a".to_string(), "b".to_string())]),
                                                    HashMap::from([("x".to_string(), "y".to_string())]),
                                                ]);
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::ServicesFFProbeResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }


    #[test]
    fn test_stream_track_unit_element_response() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let _m = build_stream_track_unit_elements_response(&mb,
                                                           0,
                                                           stream_name,
                                                           &TrackType::Meta,
                                                           track_name,
                                                           0,
                                                           &vec![],
        );

        let m = build_stream_track_unit_elements_response(&mb,
                                                             0,
                                                             stream_name,
                                                             &TrackType::Meta,
                                                             track_name,
                                                             0,
                                                             &vec![
                                                                 Payload {
                                                                     data: vec![0, 1, 2],
                                                                     attributes: HashMap::default(),
                                                                 },
                                                                 Payload {
                                                                     data: vec![1, 2, 3],
                                                                     attributes: HashMap::default(),
                                                                 },
                                                             ],
        );

        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTrackUnitElementsResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_stream_track_units_request() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_stream_track_units_request(&mb,
                                                    0,
                                                    String::from("/ab/c"),
                                                    stream_name,
                                                    &TrackType::Meta,
                                                    track_name,
                                                    0,
                                                    100,
        );
        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTrackUnitsRequest { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }

    #[test]
    fn test_stream_track_units_response() {
        let stream_id = Uuid::parse_str("fa807469-fbb3-4f63-b1a9-f63fbbf90f41").unwrap();
        let stream_name = pack_stream_name(&stream_id);
        let track_name = pack_track_name(&String::from("test")).unwrap();
        let mb = MessageBuilder::new(get_avro_path().as_str());
        let m = build_stream_track_units_response(&mb,
                                                     0,
                                                     stream_name,
                                                     &TrackType::Meta,
                                                     track_name,
                                                     0,
                                                     100,
                                                     &vec![0, 1, 2],
        );

        let value = mb.read_protocol_message(&m).unwrap();
        let pm = Message::from(&value.0, value.1);
        match pm {
            Message::StreamTrackUnitsResponse { .. } => {
                let new_m = pm.dump(&mb).unwrap();
                assert_eq!(&m, &new_m);
            }
            _ => panic!("Unexpected ProtocolMessage kind"),
        }
    }
}
