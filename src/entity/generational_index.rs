pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

pub struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

impl AllocatorEntry {
    pub fn update(&mut self) -> u64 {
        self.is_live = true;
        self.generation += 1;
        self.generation
    }
}

pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn allocate(&mut self) -> GenerationalIndex {
        match self.free.pop() {
            Some(i) => {
                let new_generation = self.entries[i].update();

                GenerationalIndex {
                    index: i,
                    generation: new_generation,
                }
            }

            None => {
                let index = self.entries.len();

                self.entries.push(AllocatorEntry {
                    is_live: true,
                    generation: 0,
                });

                GenerationalIndex { index, generation: 0 }
            }
        }
    }

    /// Returns true if the index was allocated, and is now
    /// deallocated.
    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        let entry = self.entries[index.index];
        if entry.is_live == false {
            false
        } else {
            entry.is_live = false;
            self.free.push(index.index);

            true
        }
    }

    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        self.entries[index.index].is_live
    }
}
