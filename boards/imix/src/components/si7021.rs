//! Components for the SI7021 on the imix board.
//!
//! This provides three Components, SI7021Component, which provides
//! access to the SI7021 over I2C, TemperatureComponent, which
//! provides a temperature system call driver, and HumidityComponent,
//! which provides a humidity system call driver. SI7021Component is
//! a paremeter to both TemperatureComponent and HumidityComponent.
//!
//! Usage
//! -----
//! ```rust
//! let si7021 = SI7021Component::new(mux_i2c, mux_alarm).finalize();
//! let temp = TemperatureComponent::new(si7021).finalize();
//! let humidity = HumidityComponent::new(si7021).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use sam4l;
use capsules::humidity::HumiditySensor;
use capsules::si7021::SI7021;
use capsules::temperature::TemperatureSensor;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use hil;
use kernel::component::Component;
use kernel::Grant;

pub struct SI7021Component {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl SI7021Component {
    pub fn new(i2c: &'static MuxI2C<'static>, alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>) -> Self {
        SI7021Component {
            i2c_mux: i2c,
            alarm_mux: alarm,
        }
    }
}

static mut I2C_BUF: [u8; 14] = [0; 14];

impl Component for SI7021Component {
    type Output = &'static SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let si7021_i2c = static_init!(I2CDevice,
                                      I2CDevice::new(self.i2c_mux, 0x40)
        );
        let si7021_alarm = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let si7021 = static_init!(
            SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
            SI7021::new(
                si7021_i2c,
                si7021_alarm,
                &mut I2C_BUF
            )
        );

        si7021_i2c.set_client(si7021);
        si7021_alarm.set_client(si7021);
        si7021
    }

}


pub struct TemperatureComponent {
    si7021: &'static SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>
}

impl TemperatureComponent {
    pub fn new(si: &'static SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>) -> TemperatureComponent {
        TemperatureComponent {
            si7021: si
        }
    }
}

impl Component for TemperatureComponent {
    type Output = &'static TemperatureSensor<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let temp = static_init!(
            TemperatureSensor<'static>,
            TemperatureSensor::new(self.si7021, Grant::create())
        );

        hil::sensors::TemperatureDriver::set_client(self.si7021, temp);
        temp
    }
}

pub struct HumidityComponent {
    si7021: &'static SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>
}

impl HumidityComponent {
    pub fn new(si: &'static SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>) -> HumidityComponent {
        HumidityComponent {
            si7021: si
        }
    }
}

impl Component for HumidityComponent {
    type Output = &'static HumiditySensor<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let hum = static_init!(
            HumiditySensor<'static>,
            HumiditySensor::new(self.si7021, Grant::create())
        );

        hil::sensors::HumidityDriver::set_client(self.si7021, hum);
        hum
    }
}