use crate::fs::types::{
    ep6_or_above, ep6v4_or_above, sdata_record, user_type, user_type_readable, user_type_writeable,
    SData,
};
use crate::io::{Deserialize, GameVersion, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ItemData(BTreeMap<usize, Vec<ItemRecord>>);

sdata_record!(ItemRecord {
    name                String;
    description         String;
    item_type           u8;
    item_type_id        u8;
    model               u8;
    icon                u8;
    min_level           u16;
    country             u8;
    usable_by_fighter   bool;
    usable_by_defender  bool;
    usable_by_ranger    bool;
    usable_by_archer    bool;
    usable_by_mage      bool;
    usable_by_priest    bool;
    min_game_mode       u8;
    type2               u8;
    type3               u8;
    req_str             u16;
    req_dex             u16;
    req_rec             u16;
    req_int             u16;
    req_wis             u16;
    req_luc             u16;
    req_vg              u16;
    unknown             u16     if(ep6v4_or_above);
    req_og              u8;
    req_ig              u8;
    range               u16;
    attack_time         u8;
    attrib              u8;
    special             u8;
    slot                u8;
    quality             u16;
    attack              u16;
    attack_add          u16;
    def                 u16;
    resist              u16;
    hp                  u16;
    sp                  u16;
    mp                  u16;
    str                 u16;
    dex                 u16;
    rec                 u16;
    int                 u16;
    wis                 u16;
    luc                 u16;
    speed               u8;
    exp                 u8;
    buy_price           u32;
    sell_price          u32;
    grade               u16;
    drop                u16;
    server              u8;
    count               u8;
    duration            u32     if(ep6_or_above);
    ext_duration        u8      if(ep6_or_above);
    sec_option          u8      if(ep6_or_above);
    option_rate         u8      if(ep6_or_above);
    buy_method          u8      if(ep6_or_above);
    max_level           u8      if(ep6_or_above);
    arg1                u8      if(ep6_or_above);
    arg2                u8      if(ep6_or_above);
    arg3                u8      if(ep6_or_above);
    arg4                u8      if(ep6_or_above);
    arg5                u8      if(ep6_or_above);
    arg6                u32     if(ep6_or_above);
    arg7                u32     if(ep6_or_above);
    arg8                u32     if(ep6_or_above);
    arg9                u32     if(ep6_or_above);
    arg10               u32     if(ep6_or_above);
    arg11               u32     if(ep6_or_above);
    arg12               u32     if(ep6_or_above);
    arg13               u32     if(ep6_or_above);
    arg14               u32     if(ep6_or_above);
});

impl Deserialize for ItemData {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let mut decrypted = SData::deserialize(src)?;
        let mut src = Cursor::new(&mut decrypted.data);

        let mut map = BTreeMap::new();
        let max_item_type = src.read_u32::<LE>()? as usize;
        for item_type in 1..=max_item_type {
            let max_item_type_id = src.read_u32::<LE>()? as usize;
            for item_type_id in 1..=max_item_type_id {
                let record = ItemRecord::versioned_deserialize(&mut src, version)?;
                let mut records = map
                    .entry(item_type)
                    .or_insert_with(|| Vec::with_capacity(max_item_type_id));
                records.push(record);
            }
        }

        Ok(Self(map))
    }
}

impl Serialize for ItemData {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let max_item_type = *self.0.keys().max().unwrap();
        dst.write_u32::<LE>(max_item_type as u32)?;
        for item_type in 1..=max_item_type {
            match self.0.get(&item_type) {
                Some(records) => {
                    dst.write_u32::<LE>(records.len() as u32)?;
                    for record in records {
                        record.versioned_serialize(dst, version)?;
                    }
                }
                None => dst.write_u32::<LE>(0)?,
            }
        }
        Ok(())
    }
}
