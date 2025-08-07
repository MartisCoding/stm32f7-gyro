#![no_std]
#![no_main]

mod fmt;
mod mpu;

use defmt::info;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::i2c::{Config, ErrorInterruptHandler, EventInterruptHandler, I2c};
use embassy_stm32::peripherals::I2C2;
use embassy_stm32::time::Hertz;
use crate::mpu::Mpu;

bind_interrupts!(struct Irqs {
    I2C2_EV => EventInterruptHandler<I2C2>;
    I2C2_ER => ErrorInterruptHandler<I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    
    let i2c = I2c::new(
        p.I2C2,
        p.PF1,
        p.PF0,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH3,
        Hertz(400_000),
        {
            let mut cfg = Config::default();
            cfg.scl_pullup = true;
            cfg.sda_pullup = true;
            cfg
        }
    );

    let mut accel_buff = [0.0, 0.0, 0.0];
    let mut gyro_buff = [0.0, 0.0, 0.0];
    
    let mut mpu = Mpu::new(i2c, 0x68);
    mpu.begin().await;
    
    info!("Reading values.");
    loop {
        mpu.read(&mut accel_buff, &mut gyro_buff).await;
        info!(
            "Acceleration: ({}, {}, {}), Rotation: ({}, {}, {})", 
            accel_buff[0].round(), accel_buff[1], accel_buff[2],
            gyro_buff[0], gyro_buff[1], gyro_buff[2],
        );
    }
}
