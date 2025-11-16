//! IT8951 register definitions.
//!
//! This module defines all IT8951 register addresses and provides
//! type-safe access to them.

/// IT8951 register addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(u16);

impl Register {
    /// Creates a new register address.
    pub const fn new(addr: u16) -> Self {
        Self(addr)
    }

    /// Returns the register address.
    pub const fn addr(self) -> u16 {
        self.0
    }
}

// System Registers (Base: 0x0000)
impl Register {
    /// I80 Clock Divider Control Register
    pub const I80CPCR: Self = Self(0x0004);
}

// Display Registers (Base: 0x1000)
impl Register {
    /// LUT0 Engine Width Height Register
    pub const LUT0EWHR: Self = Self(0x1000);

    /// LUT0 XY Register
    pub const LUT0XYR: Self = Self(0x1040);

    /// LUT0 Base Address Register
    pub const LUT0BADDR: Self = Self(0x1080);

    /// LUT0 Mode and Frame Number Register
    pub const LUT0MFN: Self = Self(0x10C0);

    /// LUT0 and LUT1 Active Flag Register
    pub const LUT01AF: Self = Self(0x1114);

    /// Update Parameter 0 Setting Register
    pub const UP0SR: Self = Self(0x1134);

    /// Update Parameter 1 Setting Register
    pub const UP1SR: Self = Self(0x1138);

    /// LUT0 Alpha Blend and Fill Rectangle Value
    pub const LUT0ABFRV: Self = Self(0x113C);

    /// Update Buffer Base Address
    pub const UPBBADDR: Self = Self(0x117C);

    /// LUT0 Image Buffer X/Y Offset Register
    pub const LUT0IMXY: Self = Self(0x1180);

    /// LUT All Free Status Register (status of all LUT engines)
    pub const LUTAFSR: Self = Self(0x1224);

    /// Bitmap (1bpp) Image Color Table
    pub const BGVR: Self = Self(0x1250);
}

// Memory Converter Registers (Base: 0x0200)
impl Register {
    /// Memory Converter Status Register
    pub const MCSR: Self = Self(0x0200);

    /// Load Image Start Address Register
    pub const LISAR: Self = Self(0x0208);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_addresses() {
        assert_eq!(Register::I80CPCR.addr(), 0x0004);
        assert_eq!(Register::LUT0EWHR.addr(), 0x1000);
        assert_eq!(Register::LUTAFSR.addr(), 0x1224);
        assert_eq!(Register::LISAR.addr(), 0x0208);
    }

    #[test]
    fn test_register_creation() {
        let reg = Register::new(0x1234);
        assert_eq!(reg.addr(), 0x1234);
    }

    #[test]
    fn test_register_equality() {
        assert_eq!(Register::I80CPCR, Register::new(0x0004));
        assert_ne!(Register::I80CPCR, Register::LUT0EWHR);
    }
}
