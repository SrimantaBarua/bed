// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;
use std::marker::PhantomData;

use crate::error::*;
use crate::rcbuffer::RcBuf;
use crate::types::get_u16;

pub(crate) struct LookupList<T: LookupSubtable>(RcBuf, PhantomData<T>);

impl<T: LookupSubtable> LookupList<T> {
    pub(crate) fn load(data: RcBuf) -> Result<LookupList<T>> {
        Ok(LookupList(data, PhantomData))
    }
}

impl<T: LookupSubtable> fmt::Debug for LookupList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let count = get_u16(slice, 0).unwrap() as usize;
        f.debug_struct("LookupList")
            .field("lookupCount", &count)
            .field(
                "lookups",
                &(2..2 + count * 2)
                    .step_by(2)
                    .map(|off| {
                        let offset = get_u16(slice, off).unwrap() as usize;
                        let data = self.0.slice(offset..);
                        LookupTable::<T>(data, PhantomData)
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

struct LookupTable<T: LookupSubtable>(RcBuf, PhantomData<T>);

impl<T: LookupSubtable> fmt::Debug for LookupTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice = &self.0;
        let lookup_type = get_u16(slice, 0).unwrap();
        let count = get_u16(slice, 4).unwrap() as usize;
        let flags = LookupFlag::from_bits_truncate(slice[3]);

        let mut tmp = f.debug_struct("LookupTable");
        tmp.field("lookupType", &lookup_type)
            .field("lookupFlag", &flags)
            .field("markAttachmentTypeMask", &slice[2])
            .field("subtableCount", &count)
            .field(
                "subtableOffsets",
                &(6..6 + count * 2)
                    .step_by(2)
                    .map(|off| {
                        let offset = get_u16(slice, off).unwrap() as usize;
                        T::load(self.0.slice(offset..), lookup_type).unwrap()
                    })
                    .collect::<Vec<_>>(),
            );
        if flags.contains(LookupFlag::USE_MARK_FILTERING_SET) {
            tmp.field("markFilteringSet", &get_u16(slice, 6 + count * 2).unwrap());
        }
        tmp.finish()
    }
}

bitflags! {
    struct LookupFlag : u8 {
        const RIGHT_TO_LEFT          = 0x01;
        const IGNORE_BASE_GLYPHS     = 0x02;
        const IGNORE_LIGATURES       = 0x04;
        const IGNORE_MARKS           = 0x08;
        const USE_MARK_FILTERING_SET = 0x10;
    }
}

pub(crate) trait LookupSubtable: Sized + fmt::Debug {
    fn load(data: RcBuf, lookup_type: u16) -> Result<Self>;
}
