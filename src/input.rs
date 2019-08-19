use tcod::input::KeyPressFlags;

/// Wrapper for KeyPressFlags to make things neat
pub struct KeyFlags;
impl KeyFlags {
	
	pub fn key_pressed () -> KeyPressFlags {
		return KeyPressFlags::from_bits(1 as u32).unwrap();
	}

	pub fn key_released () -> KeyPressFlags {	
		return KeyPressFlags::from_bits(2 as u32).unwrap();
	}
}