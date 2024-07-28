use crate::{
    board::zobrist::ZobristKey, movegen::defs::ShortMove, search::defs::CHECKMATE_THRESHOLD,
};

const MEGABYTE: usize = 1024 * 1024;
const ENTRIES_PER_BUCKET: usize = 4;
const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

pub trait IHashData {
    fn new() -> Self;
    fn depth(&self) -> i8;
}

#[derive(Copy, Clone)]
pub enum HashFlag {
    Nothing,
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone)]
pub struct SearchData {
    depth: i8,
    flag: HashFlag,
    value: i16,
    best_move: ShortMove,
}

impl IHashData for SearchData {
    fn new() -> Self {
        Self {
            depth: 0,
            flag: HashFlag::Nothing,
            value: 0,
            best_move: ShortMove::new(0),
        }
    }

    fn depth(&self) -> i8 {
        self.depth
    }
}

impl SearchData {
    pub fn create(depth: i8, ply: i8, flag: HashFlag, value: i16, best_move: ShortMove) -> Self {
        // Val that we store in the TT
        let mut v = value;

        if v > CHECKMATE_THRESHOLD {
            v += ply as i16;
        }
        if v < CHECKMATE_THRESHOLD {
            v -= ply as i16;
        }

        Self {
            depth,
            flag,
            value: v,
            best_move,
        }
    }

    pub fn get(&self, depth: i8, ply: i8, alpha: i16, beta: i16) -> (Option<i16>, ShortMove) {
        // We have or don't have a value to return
        let mut value: Option<i16> = None;

        if self.depth >= depth {
            match self.flag {
                HashFlag::Exact => {
                    // Get the val from the data.
                    let mut v = self.value;

                    if v > CHECKMATE_THRESHOLD {
                        v += ply as i16;
                    }
                    if v < CHECKMATE_THRESHOLD {
                        v -= ply as i16;
                    }

                    value = Some(v);
                }
                HashFlag::Alpha => {
                    if self.value <= alpha {
                        value = Some(alpha);
                    }
                }
                HashFlag::Beta => {
                    if self.value >= beta {
                        value = Some(beta);
                    }
                }
                _ => {}
            };
        }

        (value, self.best_move)
    }
}

#[derive(Copy, Clone)]
pub struct Entry<D> {
    verification: u32,
    data: D,
}

impl<D: IHashData> Entry<D> {
    pub fn new() -> Self {
        Self {
            verification: 0,
            data: D::new(),
        }
    }
}

#[derive(Clone)]
struct Bucket<D> {
    bucket: [Entry<D>; ENTRIES_PER_BUCKET],
}

impl<D: IHashData + Copy> Bucket<D> {
    pub fn new() -> Self {
        Self {
            bucket: [Entry::new(); ENTRIES_PER_BUCKET],
        }
    }

    // Store a position in the bucket. Replace the position with the stored
    // lowest depth, as positions with higher depth are more valuable.
    pub fn store(&mut self, verification: u32, data: D, used_entries: &mut usize) {
        let mut idx_lowest_depth = 0;

        // Find the index of the entry with the lowest depth.
        for entry in 1..ENTRIES_PER_BUCKET {
            if self.bucket[entry].data.depth() < data.depth() {
                idx_lowest_depth = entry
            }
        }

        // If the verifiaction was 0, this entry in the bucket was never
        // used before. Count the use of this entry.
        if self.bucket[idx_lowest_depth].verification == 0 {
            *used_entries += 1;
        }

        // Store.
        self.bucket[idx_lowest_depth] = Entry { verification, data }
    }

    // Find a position in the bucket, where both the stored verification and
    // depth match the requested verification and depth.
    pub fn find(&self, verification: u32) -> Option<&D> {
        for e in self.bucket.iter() {
            if e.verification == verification {
                return Some(&e.data);
            }
        }
        None
    }
}

pub struct TT<D> {
    tt: Vec<Bucket<D>>,   // Vector of buckets to store the transposition table entries
    megabytes: usize,     // Size of the transposition table in megabytes
    used_entries: usize,  // Number of entries currently used in the transposition table
    total_entries: usize, // Total number of entries the table can hold
    total_buckets: usize, // Total number of buckets in the transposition table
}

impl<D: IHashData + Copy + Clone> TT<D> {
    // Create a new transposition table (TT) with the specified size in megabytes.
    // The data type D must implement IHashData, and be clonable and copyable.
    pub fn new(megabytes: usize) -> Self {
        let (total_buckets, total_entries) = Self::calculate_init_values(megabytes);

        Self {
            tt: vec![Bucket::<D>::new(); total_buckets], // Initialize the TT with empty buckets
            megabytes,
            used_entries: 0,
            total_buckets,
            total_entries,
        }
    }

    // Resize the transposition table to a new size in megabytes.
    pub fn resize(&mut self, megabytes: usize) {
        let (total_buckets, total_entries) = TT::<D>::calculate_init_values(megabytes);

        self.tt = vec![Bucket::<D>::new(); total_buckets]; // Reinitialize the TT with new size
        self.megabytes = megabytes;
        self.used_entries = 0;
        self.total_buckets = total_buckets;
        self.total_entries = total_entries;
    }

    // Insert a new entry into the transposition table.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key); // Calculate bucket index
            let verification = self.calculate_verification(zobrist_key); // Calculate verification key
            self.tt[index].store(verification, data, &mut self.used_entries); // Store the entry
        }
    }

    // Probe the transposition table for an entry corresponding to the given zobrist key.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key); // Calculate bucket index
            let verification = self.calculate_verification(zobrist_key); // Calculate verification key

            self.tt[index].find(verification) // Find the entry
        } else {
            None
        }
    }

    // Clear the transposition table by resizing it to its current size.
    pub fn clear(&mut self) {
        self.resize(self.megabytes);
    }
}

impl<D: IHashData + Copy + Clone> TT<D> {
    // Calculate the index for the given zobrist key.
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        let key = (zobrist_key & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER; // Extract high four bytes
        let total = self.total_buckets as u64;

        (key % total) as usize // Modulo to get the bucket index
    }

    // Calculate the verification key for the given zobrist key.
    fn calculate_verification(&self, zobrist_key: ZobristKey) -> u32 {
        (zobrist_key & LOW_FOUR_BYTES) as u32 // Extract low four bytes
    }

    // Calculate the initial values for the number of buckets and entries based on the size in megabytes.
    fn calculate_init_values(megabytes: usize) -> (usize, usize) {
        let entry_size = std::mem::size_of::<Entry<D>>(); // Size of one entry
        let bucket_size = entry_size * ENTRIES_PER_BUCKET; // Size of one bucket
        let total_buckets = MEGABYTE / bucket_size * megabytes; // Total number of buckets
        let total_entries = total_buckets * ENTRIES_PER_BUCKET; // Total number of entries

        (total_buckets, total_entries)
    }
}
