// region:      --- Piece
pub struct Piece {
    index: u32,
    hash: [u8; 20],
    blocks: Box<[Block]>,
    begin: u64,
}

// region:      ---Constructors
impl Piece {
    pub fn new(index: u32, hash: [u8; 20], blocks: Box<[Block]>, begin: u64) -> Piece {
        Piece {
            index,
            hash,
            blocks,
            begin,
        }
    }
}
// endregion:   ---Constructors

// region:      ---Getters
impl Piece {
    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_hash(&self) -> &[u8; 20] {
        &self.hash
    }

    pub fn get_blocks(&self) -> &Box<[Block]> {
        &self.blocks
    }

    pub fn get_begin(&self) -> u64 {
        self.begin
    }

    pub fn get_length(&self) -> u32 {
        self.blocks.iter().map(|block| block.length).sum()
    }
}
// endregion:   ---Getters

// endregion:   --- Piece

// region:      --- Block
pub struct Block {
    begin: u32,
    length: u32,
}

// region:      ---Constructors
impl Block {
    pub fn new(begin: u32, length: u32) -> Block {
        Block { begin, length }
    }
}
// endregion:   ---Constructors

// region:      ---Getters
impl Block {
    pub fn get_begin(&self) -> u32 {
        self.begin
    }

    pub fn get_length(&self) -> u32 {
        self.length
    }
}
// endregion:   ---Getters
// endregion:   --- Block
