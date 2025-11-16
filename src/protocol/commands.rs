//! IT8951 command definitions.
//!
//! This module defines all IT8951 built-in and user-defined commands.

/// Built-in IT8951 TCON commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Command {
    /// System run command
    SysRun = 0x0001,

    /// Standby mode
    Standby = 0x0002,

    /// Sleep mode
    Sleep = 0x0003,

    /// Register read
    RegRead = 0x0010,

    /// Register write
    RegWrite = 0x0011,

    /// Memory burst read trigger
    MemBurstReadTrigger = 0x0012,

    /// Memory burst read start
    MemBurstReadStart = 0x0013,

    /// Memory burst write
    MemBurstWrite = 0x0014,

    /// Memory burst end
    MemBurstEnd = 0x0015,

    /// Load image start
    LoadImage = 0x0020,

    /// Load image area
    LoadImageArea = 0x0021,

    /// Load image end
    LoadImageEnd = 0x0022,
}

impl Command {
    /// Converts the command to its u16 representation.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Creates a command from a u16 value.
    pub const fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(Command::SysRun),
            0x0002 => Some(Command::Standby),
            0x0003 => Some(Command::Sleep),
            0x0010 => Some(Command::RegRead),
            0x0011 => Some(Command::RegWrite),
            0x0012 => Some(Command::MemBurstReadTrigger),
            0x0013 => Some(Command::MemBurstReadStart),
            0x0014 => Some(Command::MemBurstWrite),
            0x0015 => Some(Command::MemBurstEnd),
            0x0020 => Some(Command::LoadImage),
            0x0021 => Some(Command::LoadImageArea),
            0x0022 => Some(Command::LoadImageEnd),
            _ => None,
        }
    }
}

/// User-defined I80 commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum UserCommand {
    /// Display area
    DisplayArea = 0x0034,

    /// Get device information
    GetDevInfo = 0x0302,

    /// Display buffer area
    DisplayBufArea = 0x0037,

    /// VCOM value get/set
    Vcom = 0x0039,
}

impl UserCommand {
    /// Converts the command to its u16 representation.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Creates a user command from a u16 value.
    pub const fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0034 => Some(UserCommand::DisplayArea),
            0x0302 => Some(UserCommand::GetDevInfo),
            0x0037 => Some(UserCommand::DisplayBufArea),
            0x0039 => Some(UserCommand::Vcom),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_values() {
        assert_eq!(Command::SysRun.as_u16(), 0x0001);
        assert_eq!(Command::RegRead.as_u16(), 0x0010);
        assert_eq!(Command::LoadImage.as_u16(), 0x0020);
    }

    #[test]
    fn test_command_from_u16() {
        assert_eq!(Command::from_u16(0x0001), Some(Command::SysRun));
        assert_eq!(Command::from_u16(0x0010), Some(Command::RegRead));
        assert_eq!(Command::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_user_command_values() {
        assert_eq!(UserCommand::DisplayArea.as_u16(), 0x0034);
        assert_eq!(UserCommand::GetDevInfo.as_u16(), 0x0302);
        assert_eq!(UserCommand::Vcom.as_u16(), 0x0039);
    }

    #[test]
    fn test_user_command_from_u16() {
        assert_eq!(
            UserCommand::from_u16(0x0302),
            Some(UserCommand::GetDevInfo)
        );
        assert_eq!(UserCommand::from_u16(0x0034), Some(UserCommand::DisplayArea));
        assert_eq!(UserCommand::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_command_roundtrip() {
        let commands = [
            Command::SysRun,
            Command::Standby,
            Command::Sleep,
            Command::RegRead,
            Command::MemBurstWrite,
            Command::LoadImage,
        ];

        for cmd in commands {
            let value = cmd.as_u16();
            let decoded = Command::from_u16(value);
            assert_eq!(decoded, Some(cmd));
        }
    }
}
