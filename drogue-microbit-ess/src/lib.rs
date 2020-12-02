#![no_std]

use core::cmp;
use rubble::att::{AttUuid, Attribute, AttributeProvider, Handle, HandleRange};
use rubble::uuid::Uuid16;
use rubble::Error;

pub trait Sensor {
    fn value(&self) -> u32;
}

/// An `AttributeProvider` that will enumerate as a Environmental Sensing Service.
pub struct EnvironmentSensingService {
    attributes: [Attribute<'static>; 3],
}

const ATT_PRIMARY_SERVICE: AttUuid = AttUuid::Uuid16(Uuid16(0x2800));
const ATT_CHARACTERISTIC_DEFINITION: AttUuid = AttUuid::Uuid16(Uuid16(0x2803));
const ATT_CHARACTERISTIC_TEMPERATURE_MEASUREMENT: AttUuid = AttUuid::Uuid16(Uuid16(0x2A1C));

impl EnvironmentSensingService {
    pub fn new(data: &'static [u8]) -> Self {
        Self {
            attributes: [
                Attribute::new(ATT_PRIMARY_SERVICE, Handle::from_raw(0x0001), &[0x1A, 0x18]), // "ES Service" = 0x181A
                // Define temperature measurement
                Attribute::new(
                    ATT_CHARACTERISTIC_DEFINITION,
                    Handle::from_raw(0x0002),
                    &[
                        0x02, // 1 byte properties: READ = 0x02, NOTIFY = 0x10
                        0x03, 0x00, // 2 bytes handle = 0x0003
                        0x1C, 0x2A, // 2 bytes UUID = 0x2A1C (Temperature measurement)
                    ],
                ),
                // Characteristic value (Temperature measurement)
                Attribute::new(
                    ATT_CHARACTERISTIC_TEMPERATURE_MEASUREMENT,
                    Handle::from_raw(0x0003),
                    data,
                ),
                /*
                // Define properties
                Attribute {
                    att_type: Uuid16(0x2803).into(), // "Characteristic"
                    handle: Handle::from_raw(0x0005),
                    value: HexSlice(&[
                        0x02, // 1 byte properties: READ = 0x02
                        0x06, 0x00, // 2 bytes handle = 0x0006
                        0x0C, 0x29, // 2 bytes UUID = 0x2A1C (ES measurement)
                    ]),
                },
                // Characteristic
                Attribute {
                    att_type: AttUuid::Uuid16(Uuid16(0x290C)),
                    handle: Handle::from_raw(0x0006),
                    value: HexSlice(&[
                        0x00, 0x00, // Flags
                        0x02, // Sampling function: Arithmetic mean
                        0x00, 0x00, 0x00, // Measurement period
                        0x00, 0x00, 0x00, // Update interval
                        0x01, // Application: Air
                        0x00, // Uncertainty
                    ]),
                },*/
            ],
        }
    }
}

impl AttributeProvider for EnvironmentSensingService {
    fn for_attrs_in_range(
        &mut self,
        range: HandleRange,
        mut f: impl FnMut(&Self, Attribute<'_>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let count = self.attributes.len();
        let start = usize::from(range.start().as_u16() - 1); // handles start at 1, not 0
        let end = usize::from(range.end().as_u16() - 1);

        // Update temperature before invoking callback

        let attrs = if start >= count {
            &[]
        } else {
            let end = cmp::min(count - 1, end);
            &self.attributes[start..=end]
        };

        for attr in attrs {
            f(
                self,
                Attribute {
                    att_type: attr.att_type,
                    handle: attr.handle,
                    value: attr.value,
                },
            )?;
        }

        /*
         let value = self.sensor.value();

         att.set_value(&data);
        att*/
        Ok(())
    }

    fn is_grouping_attr(&self, uuid: AttUuid) -> bool {
        uuid == Uuid16(0x2800) // FIXME not characteristics?
    }

    fn group_end(&self, handle: Handle) -> Option<&Attribute<'_>> {
        match handle.as_u16() {
            0x0001 => Some(&self.attributes[2]),
            0x0002 => Some(&self.attributes[2]),
            _ => None,
        }
    }
}
