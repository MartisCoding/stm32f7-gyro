use defmt::{debug, error, info};
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Async;
use embassy_time::Timer;



pub struct Mpu<'a> {
    bus: I2c<'a, Async, Master>,
    slv_addr: u8,
    buff: [u8; 14],
}

impl<'a> Mpu<'a> {
    
    /// Configures default I2C mode for MPU-9250. You have to provide pull up in the I2C configuration.
    /// Consumes the driver. The slave address must be 0x68 if AD0 on GND or 0x69 if on VCC.
    pub const fn new(
        bus: I2c<'a, Async, Master>, 
        slv_addr: u8,
    ) -> Self {
        Self { bus, slv_addr, buff: [0; 14]}
    }

    async fn internal_write_register(&mut self, reg_addr: u8, value: u8) -> bool {
        let res = self.bus.write(
            self.slv_addr,
            &[reg_addr, value],
        ).await;

        match res {
            Ok(_) => {
                debug!("Bus write transfer success!");
                true
            },
            Err(e) => {
                error!("Bus write transfer failed! {:?}", e);
                false
            }
        }
    }

    async fn internal_read_register(&mut self, reg_addr: u8) -> Option<u8>  {
        let mut data = [0u8];
        let res = self.bus.write_read(
            self.slv_addr,
            &[reg_addr],
            &mut data
        ).await;

        match res {
            Ok(_) => {
                debug!( "Bus read transfer success!");
                Some(data[0])
            }
            Err(e) => {
                error!("Bus read transfer failed! {:?}", e);
                None
            }
        }
    }
    ///Sets up mpu-9250. If something goes wrong, will panic.
    pub async fn begin(&mut self) {
        let Some(reg) = self.internal_read_register(0x75).await else {
            error!("Could not read WHO_AM_I register!");
            panic!("Failed to setup MPU!");
        };
        if reg != 0x71 && reg != 0x48 && reg != 0x75 {
            error!("Not an MPU: {}", reg);
            panic!("Failed to setup MPU!");
        };

        if !self.internal_write_register(0x6B, 0x80).await {
            error!("Failed to reset PWR_MGMT_1");
            panic!("Failed to setup MPU!");
        }

        Timer::after_millis(100).await;

        if !self.internal_write_register(0x6B, 0x01).await {
            error!("Failed to wake up MPU");
            panic!("Failed to setup MPU!");
        }

        Timer::after_millis(50).await;

        if !self.internal_write_register(0x6A, 0x00).await {
            error!("Failed to disable I2C master mode on MPU");
            panic!("Failed to setup MPU!");
        }

        if !self.internal_write_register(0x19, 0x07).await {
            error!("Failed to set sample rate!");
            panic!("Failed to setup MPU!");
        }

        if !self.internal_write_register(0x1A, 0x03).await {
            error!("Failed to set DLPF frequency!");
            panic!("Failed to setup MPU!");
        }

        if !self.internal_write_register(0x1B, 0x00).await {
            error!("Failed to reset GYRO_CONFIG");
            panic!("Failed to setup MPU!");
        }

        if !self.internal_write_register(0x1C, 0x00).await {
            error!("Failed to reset ACCEL_CONFIG");
            panic!("Failed to setup MPU!");
        }

        if !self.internal_write_register(0x1D, 0x03).await {
            error!("Failed to reset ACCEL_CONFIG2");
            panic!("Failed to setup MPU!");
        }

        Timer::after_millis(20).await;

        info!("MPU initialized!");
    }
    
    ///By providing references to accelerator and gyroscope buffer, will propagate them
    /// with values. No align is needed, everything is working out of the box. 
    pub async fn read(&mut self, acc_buf_ref: &mut [f32], gyro_buf_ref: &mut [f32]) {
        assert_eq!(acc_buf_ref.len(), 3);
        assert_eq!(gyro_buf_ref.len(), 3);
        
        let res = self.bus.write_read(
            self.slv_addr,
            &[0x3B],
            &mut self.buff
        ).await;

        match res {
            Ok(_) => {
                debug!("Bus read raw success! Formatting");
                let ax = ((self.buff[0] as i16) << 8 | (self.buff[1] as i16)) as f32 / 16384.0;
                let ay = ((self.buff[2] as i16) << 8 | (self.buff[3] as i16)) as f32 / 16384.0;
                let az = ((self.buff[4] as i16) << 8 | (self.buff[5] as i16)) as f32 / 16384.0;
                
                let gx  = ((self.buff[8] as i16) << 8 | (self.buff[9] as i16)) as f32 / 131.0;
                let gy = ((self.buff[10] as i16) << 8 | (self.buff[11] as i16)) as f32 / 131.0;
                let gz = ((self.buff[12] as i16) << 8 | (self.buff[13] as i16)) as f32 / 131.0;
                
                acc_buf_ref[0] = ax;
                acc_buf_ref[1] = ay;
                acc_buf_ref[2] = az;
                
                gyro_buf_ref[0] = gx;
                gyro_buf_ref[1] = gy;
                gyro_buf_ref[2] = gz;
                
                debug!("Done!");
            },
            Err(e) => {
                error!("Bus read transfer failed! {:?}", e);
            }
        }
        Timer::after_millis(100).await;
    }
}

