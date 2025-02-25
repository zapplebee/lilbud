// ==========================================================================
// Register and Configuration Definitions
//
// The following register addresses and bit–patterns are chosen to mirror the
// definitions in the original C code (see QMI8658.h and QMI8658.c). You must
// verify that these values match your hardware and driver documentation.
// ==========================================================================
const REG_WHO_AM_I: u8 = 0x00;      // Expected chip ID is 0x05.
const REG_CTRL1: u8 = 0x02;         // e.g. in C: QMI8658_write_reg(QMI8658Register_Ctrl1, 0x60)
const REG_CTRL2: u8 = 0x03;         // Accelerometer configuration.
const REG_CTRL3: u8 = 0x04;         // Gyroscope configuration.
const REG_CTRL4: u8 = 0x05;         // Magnetometer (if used).
const REG_CTRL5: u8 = 0x06;         // Filter settings.
const REG_CTRL6: u8 = 0x07;         // Attitude Engine settings.
const REG_CTRL7: u8 = 0x08;         // Sensor enable register.
const REG_TEMP_L: u8 = 51;          // Temperature low byte.
const REG_AX_L: u8 = 53;            // Accelerometer X-axis low byte.
const REG_GX_L: u8 = 59;            // Gyroscope X-axis low byte.

// --------------------------------------------------------------------------
// Enumerations to match the C code’s configuration values.
// The C code uses bit–patterns (e.g. QMI8658AccRange_8g is 0x02<<4).
// --------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum AccRange {
    G2,
    G4,
    G8,
    G16,
}
impl AccRange {
    pub fn to_u8(self) -> u8 {
        match self {
            AccRange::G2 => 0x00 << 4,
            AccRange::G4 => 0x01 << 4,
            AccRange::G8 => 0x02 << 4,
            AccRange::G16 => 0x03 << 4,
        }
    }
}

#[derive(Copy, Clone)]
pub enum AccOdr {
    Hz8000,
    Hz4000,
    Hz2000,
    Hz1000,
    Hz500,
    Hz250,
    Hz125,
    Hz62_5,
    LowPower128,
    LowPower21,
    LowPower11,
    LowPower3,
}
impl AccOdr {
    pub fn to_u8(self) -> u8 {
        match self {
            AccOdr::Hz8000 => 0x00,
            AccOdr::Hz4000 => 0x01,
            AccOdr::Hz2000 => 0x02,
            AccOdr::Hz1000 => 0x03,
            AccOdr::Hz500  => 0x04,
            AccOdr::Hz250  => 0x05,
            AccOdr::Hz125  => 0x06,
            AccOdr::Hz62_5 => 0x07,
            AccOdr::LowPower128 => 0x0c,
            AccOdr::LowPower21  => 0x0d,
            AccOdr::LowPower11  => 0x0e,
            AccOdr::LowPower3   => 0x0f,
        }
    }
}

#[derive(Copy, Clone)]
pub enum GyrRange {
    Dps32,
    Dps64,
    Dps128,
    Dps256,
    Dps512,
    Dps1024,
    Dps2048,
    Dps4096,
}
impl GyrRange {
    pub fn to_u8(self) -> u8 {
        match self {
            GyrRange::Dps32 => 0x00 << 4,
            GyrRange::Dps64 => 0x01 << 4,
            GyrRange::Dps128 => 0x02 << 4,
            GyrRange::Dps256 => 0x03 << 4,
            GyrRange::Dps512 => 0x04 << 4,
            GyrRange::Dps1024 => 0x05 << 4,
            GyrRange::Dps2048 => 0x06 << 4,
            GyrRange::Dps4096 => 0x07 << 4,
        }
    }
}

#[derive(Copy, Clone)]
pub enum GyrOdr {
    Hz8000,
    Hz4000,
    Hz2000,
    Hz1000,
    Hz500,
    Hz250,
    Hz125,
    Hz62_5,
    Hz31_25,
}
impl GyrOdr {
    pub fn to_u8(self) -> u8 {
        match self {
            GyrOdr::Hz8000 => 0x00,
            GyrOdr::Hz4000 => 0x01,
            GyrOdr::Hz2000 => 0x02,
            GyrOdr::Hz1000 => 0x03,
            GyrOdr::Hz500  => 0x04,
            GyrOdr::Hz250  => 0x05,
            GyrOdr::Hz125  => 0x06,
            GyrOdr::Hz62_5 => 0x07,
            GyrOdr::Hz31_25 => 0x08,
        }
    }
}

#[derive(Copy, Clone)]
pub enum LpfConfig {
    Disable,
    Enable,
}

#[derive(Copy, Clone)]
pub enum StConfig {
    Disable,
    Enable,
}

// A simplified configuration structure; you may expand this to mirror
// the full C structure.
pub struct SensorConfig {
    pub input_selection: u8, // Bitmask: e.g. 0x01 = acc, 0x02 = gyro.
    pub acc_range: AccRange,
    pub acc_odr: AccOdr,
    pub gyr_range: GyrRange,
    pub gyr_odr: GyrOdr,
}

// ==========================================================================
// Low–Level Sensor Driver
//
// This struct wraps an I²C instance and implements the basic read/write,
// configuration, and initialization routines as found in the C code.
// ==========================================================================
pub struct QMI8658<I2C> {
    i2c: I2C,
    slave_addr: u8,
}

impl<I2C, E> QMI8658<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    fn new(i2c: I2C, slave_addr: u8) -> Self {
        QMI8658 { i2c, slave_addr }
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), E> {
        self.i2c.write(self.slave_addr, &[reg, value])
    }

    fn read_reg(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(self.slave_addr, &[reg], buf)
    }

    /// Perform basic initialization of the sensor.
    ///
    /// This function reads the WHO_AM_I register (expected 0x05) and writes
    /// a default value (0x60) to Ctrl1 as in the C code.
    fn init_sensor(&mut self) -> Result<(), E> {
        let mut id = [0u8; 1];
        self.read_reg(REG_WHO_AM_I, &mut id)?;
        if id[0] != 0x05 {
            // In the C code, failure here might trigger retries or error out.
            // For now we simply continue (you may wish to return an error).
        }
        self.write_reg(REG_CTRL1, 0x60)?;
        Ok(())
    }

    /// Configure the accelerometer.
    ///
    /// This function mirrors the C routine QMI8658_config_acc.
    /// It computes a sensitivity factor (acc_lsb_div) and writes to Ctrl2 and Ctrl5.
    fn config_acc(
        &mut self,
        range: AccRange,
        odr: AccOdr,
        lpf: LpfConfig,
        st: StConfig,
    ) -> Result<u16, E> {
        let acc_lsb_div = match range {
            AccRange::G2 => 1 << 14,
            AccRange::G4 => 1 << 13,
            AccRange::G8 => 1 << 12,
            AccRange::G16 => 1 << 11,
        };
        let mut ctl_data = range.to_u8() | odr.to_u8();
        if let StConfig::Enable = st {
            ctl_data |= 0x80;
        }
        self.write_reg(REG_CTRL2, ctl_data)?;
        let ctl5 = if let LpfConfig::Enable = lpf { 0x0F } else { 0x00 };
        self.write_reg(REG_CTRL5, ctl5)?;
        Ok(acc_lsb_div)
    }

    /// Configure the gyroscope.
    ///
    /// This function mirrors QMI8658_config_gyro from the C code.
    fn config_gyro(
        &mut self,
        range: GyrRange,
        odr: GyrOdr,
        lpf: LpfConfig,
        st: StConfig,
    ) -> Result<u16, E> {
        let gyro_lsb_div = match range {
            GyrRange::Dps32 => 1024,
            GyrRange::Dps64 => 512,
            GyrRange::Dps128 => 256,
            GyrRange::Dps256 => 128,
            GyrRange::Dps512 => 64,
            GyrRange::Dps1024 => 32,
            GyrRange::Dps2048 => 16,
            GyrRange::Dps4096 => 8,
        };
        let mut ctl_data = range.to_u8() | odr.to_u8();
        if let StConfig::Enable = st {
            ctl_data |= 0x80;
        }
        self.write_reg(REG_CTRL3, ctl_data)?;
        let ctl5 = if let LpfConfig::Enable = lpf { 0x0A } else { 0x00 };
        self.write_reg(REG_CTRL5, ctl5)?;
        Ok(gyro_lsb_div)
    }

    /// Read the accelerometer values for x, y, and z.
    fn read_acc_xyz(&mut self) -> Result<[i16; 3], E> {
        let mut buf = [0u8; 6];
        self.read_reg(REG_AX_L, &mut buf)?;
        let ax = i16::from_le_bytes([buf[0], buf[1]]);
        let ay = i16::from_le_bytes([buf[2], buf[3]]);
        let az = i16::from_le_bytes([buf[4], buf[5]]);
        Ok([ax, ay, az])
    }

    /// Read the gyroscope values for x, y, and z.
    fn read_gyro_xyz(&mut self) -> Result<[i16; 3], E> {
        let mut buf = [0u8; 6];
        self.read_reg(REG_GX_L, &mut buf)?;
        let gx = i16::from_le_bytes([buf[0], buf[1]]);
        let gy = i16::from_le_bytes([buf[2], buf[3]]);
        let gz = i16::from_le_bytes([buf[4], buf[5]]);
        Ok([gx, gy, gz])
    }
}

// ==========================================================================
// High–Level Sensors API
//
// The Sensors struct encapsulates the sensor driver (and any configuration
// details such as sensitivity factors). It exposes a simple read() method
// that returns (accelerometer, gyroscope) readings.
// ==========================================================================
pub struct Sensors<I2C> {
    sensor: QMI8658<I2C>,
    // The sensitivity divisors computed during configuration can be stored here.
    pub acc_div: u16,
    pub gyro_div: u16,
}

impl<I2C, E> Sensors<I2C>
where I2C: Write<Error = E> + WriteRead<Error = E>
{
    /// Read the accelerometer and gyroscope data.
    ///
    /// This method calls the low–level read_acc_xyz and read_gyro_xyz functions.
    pub fn read(&mut self) -> Result<([i16; 3], [i16; 3]), E> {
        let acc = self.sensor.read_acc_xyz()?;
        let gyro = self.sensor.read_gyro_xyz()?;
        Ok((acc, gyro))
    }
}

// ==========================================================================
// I2C Initialization
//
// This is a dummy function to illustrate that the I2C bus setup should
// be performed here. Replace this with your actual board–specific
// I2C initialization code.
// ==========================================================================
fn init_i2c() -> Result<impl Write<Error = Infallible> + WriteRead<Error = Infallible>, ()> {
    // For demonstration, we use unimplemented!().
    // In a real application, you would configure your I2C peripheral and pins here.
    unimplemented!("Replace this with your board-specific I2C initialization")
}

// ==========================================================================
// Public API
//
// The init() function encapsulates all sensor initialization and configuration,
// returning a Sensors instance that can later be used to read sensor data.
// ==========================================================================
pub fn init() -> Result<Sensors<impl Write<Error = Infallible> + WriteRead<Error = Infallible>>, ()> {
    // Initialize the I2C bus.
    let i2c = init_i2c()?;
    // Create a QMI8658 instance with the appropriate slave address (0x6a or 0x6b).
    let mut sensor = QMI8658::new(i2c, 0x6a);
    // Perform basic sensor initialization.
    sensor.init_sensor().map_err(|_| ())?;
    // Create a default configuration matching the C defaults.
    // In the C code, QMI8658_Config_apply sets:
    //   - Accelerometer: 8g, 1000Hz
    //   - Gyroscope: 512dps, 1000Hz
    let config = SensorConfig {
        input_selection: 0x01 | 0x02, // Enable accelerometer and gyroscope.
        acc_range: AccRange::G8,
        acc_odr: AccOdr::Hz1000,
        gyr_range: GyrRange::Dps512,
        gyr_odr: GyrOdr::Hz1000,
    };
    let acc_div = sensor.config_acc(config.acc_range, config.acc_odr, LpfConfig::Enable, StConfig::Disable)
        .map_err(|_| ())?;
    let gyro_div = sensor.config_gyro(config.gyr_range, config.gyr_odr, LpfConfig::Enable, StConfig::Disable)
        .map_err(|_| ())?;
    // Enable sensors by writing to Ctrl7.
    sensor.write_reg(REG_CTRL7, config.input_selection & 0x0F)
        .map_err(|_| ())?;
    // Return the high-level Sensors instance.
    Ok(Sensors {
        sensor,
        acc_div,
        gyro_div,
    })
}
