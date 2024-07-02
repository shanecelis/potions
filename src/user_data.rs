
bitflags::bitflags! {
    pub struct UserDataFlags: u8 {
        const OBJECT = 0b00000001;
        const WALL = 0b00000010;
    }
}

pub struct UserData {
    pub flags: UserDataFlags,
    pub id: u8,
}

impl UserData {
    pub fn wall(id: u8) -> Self {
        Self {
            flags: UserDataFlags::WALL,
            id
        }
    }

    pub fn object(id: u8) -> Self {
        Self {
            flags: UserDataFlags::OBJECT,
            id
        }
    }

    pub fn is_wall(&self) -> bool {
        self.flags.contains(UserDataFlags::WALL)
    }

    pub fn is_object(&self) -> bool {
        self.flags.contains(UserDataFlags::OBJECT)
    }
}

impl From<UserData> for u64 {
    fn from(user_data: UserData) -> u64 {
        (user_data.flags.bits() as u64) << 8 | user_data.id as u64
    }
}

impl From<u64> for UserData {
    fn from(n: u64) -> UserData {
        UserData {
            id: (n & 0xff) as u8,
            flags: UserDataFlags::from_bits((n >> 8 & 0xff) as u8).expect("flags")
        }
    }
}

impl From<UserData> for i64 {
    fn from(user_data: UserData) -> i64 {
        (user_data.flags.bits() as i64) << 8 | user_data.id as i64
    }
}

impl From<i64> for UserData {
    fn from(n: i64) -> UserData {
        UserData {
            id: (n & 0xff) as u8,
            flags: UserDataFlags::from_bits((n >> 8 & 0xff) as u8).expect("flags")
        }
    }
}

impl From<UserData> for u128 {
    fn from(user_data: UserData) -> u128 {
        (user_data.flags.bits() as u128) << 8 | user_data.id as u128
    }
}

impl From<u128> for UserData {
    fn from(n: u128) -> UserData {
        UserData {
            id: (n & 0xff) as u8,
            flags: UserDataFlags::from_bits(((n >> 8) & 0xff) as u8).expect("flags")
        }
    }
}
