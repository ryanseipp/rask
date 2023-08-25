bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CqeFlags: u32 {
        const Buffer = 1;
        const More = 1 << 1;
        const SockNonEmpty = 1 << 2;
        const Notification = 1 << 3;

        const _ = !0;
    }
}

impl CqeFlags {
    pub fn get_buffer_id(&self) -> Option<u16> {
        if self.contains(CqeFlags::Buffer) {
            Some((self.bits() >> 16) as u16)
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct IoUringCqe {
    user_data: u64,
    res: i32,
    flags: u32,
    // If the ring is initialized with IORING_SETUP_CQE32, then this field contains 16 bytes of
    // padding, double the size of the CQE.
    // big_cqe: [u64; 0],
}

impl IoUringCqe {
    pub fn get_data(&self) -> u64 {
        self.user_data
    }

    pub unsafe fn get_data_t<T>(&self) -> Option<&T> {
        (self.user_data as *const T).as_ref()
    }

    pub fn flags(&self) -> CqeFlags {
        CqeFlags::from_bits_retain(self.flags)
    }

    pub fn result(&self) -> i32 {
        self.res
    }
}

#[cfg(test)]
mod test {
    use super::CqeFlags;

    #[test]
    fn buffer_flag_equals_1() {
        assert_eq!(CqeFlags::Buffer.bits(), 1 << 0);
    }

    #[test]
    fn more_flag_equals_2() {
        assert_eq!(CqeFlags::More.bits(), 1 << 1);
    }

    #[test]
    fn sock_non_empty_flag_equals_4() {
        assert_eq!(CqeFlags::SockNonEmpty.bits(), 1 << 2);
    }

    #[test]
    fn notification_flag_equals_8() {
        assert_eq!(CqeFlags::Notification.bits(), 1 << 3);
    }
}
