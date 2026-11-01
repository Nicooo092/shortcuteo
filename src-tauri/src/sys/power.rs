use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

pub fn on_battery() -> bool {
    let mut status = SYSTEM_POWER_STATUS::default();
    // ACLineStatus: 0 offline, 1 online, 255 unknown (desktops report 1)
    unsafe { GetSystemPowerStatus(&mut status) }.is_ok() && status.ACLineStatus == 0
}
