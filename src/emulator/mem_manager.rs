

pub struct MemoryManager {
    ram: Vec<u8>,
}

impl MemoryManager {
    pub fn new() -> MemoryManager {
        MemoryManager { ram: vec![0; 0x2000] }
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn load_init_buff(&mut self, buff: Vec<u8>) {
        self.ram = buff;
    }

    pub fn load_8(&self, addr: u32) -> u8 {
        if self.ram.len() < addr as usize {
            self.ram[addr as usize]
        } else {
            0
        }
    }

    pub fn load_32(&self, addr: u32) -> u32 {
        if self.ram.len() > addr as usize {
            self.ram[addr as usize] as u32 +
                ((self.ram[(addr + 1) as usize] as u32) << 8) +
                ((self.ram[(addr + 2) as usize] as u32) << 16) +
                ((self.ram[(addr + 3) as usize] as u32) << 24)
        } else {
            0
        }
    }

    pub fn store_8(&mut self, addr: u32, data: u8) {
        if self.ram.len() < addr as usize {
            self.ram.reserve( addr as usize + 100 );
        }

        self.ram[addr as usize] = data;
    }

    pub fn store_32(&mut self, addr: u32, value: u32) {
        if self.ram.len() < addr as usize + 3 {
            self.ram.reserve(addr as usize + 100 - self.ram.len());
        }

        self.ram[addr as usize] = value as u8;
        self.ram[(addr + 1) as usize] = (value >> 8) as u8;
        self.ram[(addr + 2) as usize] = (value >> 16) as u8;
        self.ram[(addr + 3) as usize] = (value >> 24) as u8;
    }
}