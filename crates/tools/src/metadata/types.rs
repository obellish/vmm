macro_rules! impl_device_type {
	($type_name: ident as $type_enum: ident => { $($dev_name: ident => $dev_code: expr),* }) => {
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy)]
        pub enum $type_enum {
            $($dev_name),*
        }

        impl $type_enum {
            #[must_use]
            pub const fn decode(code: u32) -> ::std::option::Option<Self> {
                match code {
                    $($dev_code => Some(Self::$dev_name)),*
                    , _ => None,
                }
            }

            #[must_use]
            pub const fn code(self) -> u32 {
                match self {
                    $(Self::$dev_name => $dev_code),*
                }
            }

            #[must_use]
            pub const fn wrap(self) -> $crate::metadata::DeviceCategory {
                $crate::metadata::DeviceCategory::$type_name(self)
            }

            #[must_use]
            pub const fn encode(self) -> u64 {
                self.wrap().encode()
            }
        }

        impl ::std::convert::From<$type_enum> for $crate::metadata::DeviceCategory {
            fn from(typ: $type_enum) -> Self {
                typ.wrap()
            }
        }

        impl ::std::fmt::Display for $type_enum {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$dev_name => stringify!($dev_name)),*
                })
            }
        }
    };
}

impl_device_type!(Debug as DebugType => {
	Basic => 0x0000_0100
});

impl_device_type!(Clock as ClockType => {
	Realtime => 0x0000_0001
});

impl_device_type!(Display as DisplayType => {
	Number => 0x0000_0001,
	Character => 0x0000_0010,
	Buffered => 0x0000_0100
});

impl_device_type!(Keyboard as KeyboardType => {
	ReadCharSynchronous => 0x0000_0100,
	ReadLineSynchronous => 0x0000_1000
});

impl_device_type!(Memory as MemoryType => {
	Ram => 0x0000_0100
});

impl_device_type!(Storage as StorageType => {
	Readonly => 0x0000_0100,
	Flash => 0x0000_0011,
	Persistent => 0x0000_0021
});
