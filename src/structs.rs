use std::fmt::Error;
use std::collections::VecDeque;
use bitvec;

pub enum Prefixes {
    SerializedBocIdx = 0x68ff65f3,
    SerializedBocIdxCrc32c = 0xacc3a728,
    SerializedBoc = 0xb5ee9c72,
}

#[derive(Debug)]
pub struct Int257(pub bitvec::BitArr!(for 257, in u8));

#[derive(Debug, Clone)]
pub struct Slice{
    data: bitvec::vec::BitVec<u8>,
}
impl Slice {
    pub fn new(data: bitvec::vec::BitVec<u8>) -> Result<Slice, Error> {
        if data.len() <= 1023 {
            Ok(Slice {data})
        }
        else {
            Err(Error)
        }
    }
    pub fn from_bitslice<'a>(data: &'a bitvec::slice::BitSlice<u8>) -> Result<Slice, Error> {
        Slice::new(data.to_bitvec())
    }
    pub fn len(&self) -> u16 {
        self.data.len() as u16
    }
    pub fn data(&self) -> bitvec::vec::BitVec<u8> {
        self.data.clone()
    }
    pub fn to_bitslice(&self) -> &bitvec::slice::BitSlice<u8> {
        self.data.as_bitslice()
    }
    pub fn load_bits(&mut self, n: u16) -> Result<Slice, Error> {
        if self.len() < n {
            return Err(Error);
        }
        let res: Slice = Slice::from_bitslice(&self.data[0..n as usize])?;
        self.data = self.data[n as usize..].to_bitvec();
        Ok(res)
    }
    pub fn preload_bits(&self, n: u16) -> Result<Slice, Error> {
        if self.len() < n {
            return Err(Error);
        }
        Ok(Slice::from_bitslice(&self.data[0..n as usize])?)
    }
    pub fn load_int(&mut self) -> Result<Int257, Error> {
        Ok(Int257(self.load_bits(257)?.data()[..257].try_into().unwrap()))
    }
    pub fn preload_int(&self) -> Result<Int257, Error> {
        Ok(Int257(self.preload_bits(257)?.data()[..257].try_into().unwrap()))
    } 
    pub fn skip_bits(&mut self, n: u16) -> Result<(), Error> {
        self.data = self.data[n as usize..].to_bitvec();
        Ok(())
    }
    pub fn store_int(&mut self, a: Int257) -> Result<(), Error> {
        if self.len() + 257 > 1023 {
            return Err(Error);
        }
        self.data.extend(a.0.iter());
        Ok(())
    }
    pub fn store_slice(&mut self, a: Slice) -> Result<(), Error> {
        if self.len() + a.len() > 1023 {
            return Err(Error);
        }
        self.data.extend(a.data().iter());
        Ok(())
    }
    pub fn store_int_as_slice(&mut self, a: &[u8], n: u16) -> Result<(), Error> {
        if self.len() + n > 1023 {
            return Err(Error);
        }
        self.data.extend_from_bitslice(&bitvec::slice::BitSlice::<u8>::from_slice(a)[..n as usize]);
        Ok(())
    }
    pub fn store_bool(&mut self, a: bool) -> Result<(), Error> {
        if self.len() > 1022 {
            return Err(Error);
        }
        self.data.push(a);
        Ok(())
    }

    // pub fn first_bits(&self, n: u16) -> Result<Slice, Error> {
    //     todo!()
    // }
    // pub fn end_parse(&self) -> Result<(), Error> {
    //     todo!()
    // }
    // pub fn load_ref(&self) -> Result<Cell, Error> {
    //     todo!()
    // }
    // pub fn preload_ref(&self) -> Result<Cell, Error> {
    //     todo!()
    // }
}
pub struct Cell {
    pub data: Slice,
    refs: Vec<Cell>
}
impl Cell {
    pub fn new(data: Slice) -> Result<Cell, Error> {
        Ok(Cell{data, refs: Vec::new()})
    }
    pub fn ref_len(&self) -> u8 {
        self.refs.len() as u8
    }
    pub fn store_ref(&mut self, other: Cell) -> Result<(), Error> {
        if self.ref_len() >= 4 {
            return Err(Error);
        }
        self.refs.push(other);
        Ok(())
    }
    pub fn get_refs(&self) -> &Vec<Cell> {
        &self.refs
    }
    pub fn level(&self) -> usize {
        let mut max: usize = 0;
        let mut queue: VecDeque<(&Cell, usize)> = VecDeque::new();
        queue.push_back((self,1));
        while !queue.is_empty() {
            let item = queue.pop_front().unwrap();
            if max < item.1 {
                max = item.1;
            }
            for l in &item.0.refs {
                queue.push_back((l, item.1 + 1));
            }
        }
        max
    }
}
