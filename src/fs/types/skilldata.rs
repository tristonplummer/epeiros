use crate::fs::types::*;
use crate::io::{Deserialize, GameVersion, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SkillData(BTreeMap<usize, Vec<SkillRecord>>);

sdata_record!(SkillRecord {
    name                            String;
    description                     String;
    rank                            u8;
    icon                            u16;
    animation                       u16;
    effect                          u8          if(ep6_or_above);
    can_be_toggled                  bool;
    sound                           u16;
    min_level                       u16;
    permitted_races                 PermittedRace;
    usable_by_fighter               bool;
    usable_by_defender              bool;
    usable_by_ranger                bool;
    usable_by_archer                bool;
    usable_by_mage                  bool;
    usable_by_priest                bool;
    min_game_mode                   GameMode;
    skill_point_cost                u8;
    category                        SkillCategory;
    type_attack                     u8;
    type_effect                     u8;
    type_detail                     u16;
    usable_with_one_handed_sword    bool;
    usable_with_two_handed_sword    bool;
    usable_with_one_handed_axe      bool;
    usable_with_two_handed_axe      bool;
    usable_with_dual_swords         bool;
    usable_with_spear               bool;
    usable_with_one_handed_mace     bool;
    usable_with_two_handed_mace     bool;
    usable_with_reverse_sword       bool;
    usable_with_dagger              bool;
    usable_with_javelin             bool;
    usable_with_staff               bool;
    usable_with_bow                 bool;
    usable_with_crossbow            bool;
    usable_with_fist_weapon         bool;
    usable_with_shield              bool;
    sp_cost                         u16;
    mp_cost                         u16;
    cast_time                       u8;
    cooldown_duration               u16;
    attack_distance                 u8;
    state_type                      u8;
    element                         ElementType;
    disable                         u16;
    prerequisite_skill              u16;
    success_type                    u8;
    success_chance                  u8;
    target_type                     TargetType;
    radius                          u8;
    number_of_multi_hits            u8;
    duration                        u16;
    weapon1                         u8;
    weapon2                         u8;
    weapon_value                    u8;
    bag                             u8;
    arrow                           u16;
    damage_type                     DamageType;
    damage_hp                       u16;
    damage_sp                       u16;
    damage_mp                       u16;
    damage_over_time_type           DamageOverTimeType;
    damage_over_time_hp             u16;
    damage_over_time_sp             u16;
    damage_over_time_mp             u16;
    additional_damage_hp            u16;
    additional_damage_sp            u16;
    additional_damage_mp            u16;
    ability_effects                 Vec<AbilityRecord>  len(max_ability_types_for_version);
    heal_hp                         u16;
    heal_sp                         u16;
    heal_mp                         u16;
    heal_over_time_hp               u16;
    heal_over_time_sp               u16;
    heal_over_time_mp               u16;
    damage_avoid_type               u8;
    damage_avoid_value              u8;
    hp_trigger_threshold            u8;
    duration_type                   DurationType;
    change_type                     u16;
    change_value                    u16;
});

sdata_record!(AbilityRecord {
    ability_type    u8;
    ability_value   u16;
});

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum TargetType {
    #[default]
    CannotBeCasted,
    None,
    OnSelf,
    Enemy,
    Party,
    PartyExceptSelf,
    AroundCaster,
    AroundTarget,
    Raid,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum SkillCategory {
    #[default]
    None,
    Passive,
    Basic,
    Combat,
    Special,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum DamageType {
    #[default]
    NotUsed,
    Fixed,
    PlusAdditional,
    Coefficient,
    CasterCurrentHitpoints,
    TargetManaPercent,
    TargetHitpointsPercent,
    RecCoefficient,
    RecPlusAdditional,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum DamageOverTimeType {
    #[default]
    None,
    Percent,
    Exponential,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum DurationType {
    #[default]
    None,
    SecondsAndDisappearOnDeath,
    HoursAndPersistsOnDeath,
    SecondsAndPersistsOnDeath,
}

impl Deserialize for SkillData {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let mut decrypted = SData::deserialize(src)?;
        let mut src = Cursor::new(&mut decrypted.data);

        let max_skill_id = src.read_u32::<LE>()? as usize;
        let ranks_per_skill = ranks_per_skill_for_version(version);

        let mut map = BTreeMap::new();
        for skill_id in 1..=max_skill_id {
            for _rank in 1..=ranks_per_skill {
                let record = SkillRecord::versioned_deserialize(&mut src, version)?;
                map.entry(skill_id)
                    .or_insert_with(|| Vec::with_capacity(ranks_per_skill))
                    .push(record);
            }
        }

        Ok(Self(map))
    }
}

impl Serialize for SkillData {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let max_skill_id = *self.0.keys().max().unwrap();
        dst.write_u32::<LE>(max_skill_id as u32)?;

        let mut default_record = SkillRecord::default();
        let ranks_per_skill = ranks_per_skill_for_version(version);

        for skill_id in 1..=max_skill_id {
            let records = self
                .0
                .get(&skill_id)
                .expect("failed to get record for skill id");

            for rank in 1..=ranks_per_skill {
                if rank > records.len() {
                    default_record.rank = rank as u8;
                    default_record.versioned_serialize(dst, version)?;
                } else {
                    records[rank - 1].versioned_serialize(dst, version)?;
                }
            }
        }
        Ok(())
    }
}

impl Deserialize for TargetType {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let target_type = src.read_u8()?;
        return match target_type {
            0 => Ok(Self::CannotBeCasted),
            1 => Ok(Self::None),
            2 => Ok(Self::OnSelf),
            3 => Ok(Self::Enemy),
            4 => Ok(Self::Party),
            5 => Ok(Self::PartyExceptSelf),
            6 => Ok(Self::AroundCaster),
            7 => Ok(Self::AroundTarget),
            8 => Ok(Self::Raid),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid target type {target_type}"),
            )),
        };
    }
}

impl Serialize for TargetType {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::CannotBeCasted => 0,
            Self::None => 1,
            Self::OnSelf => 2,
            Self::Enemy => 3,
            Self::Party => 4,
            Self::PartyExceptSelf => 5,
            Self::AroundCaster => 6,
            Self::AroundTarget => 7,
            Self::Raid => 8,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for SkillCategory {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let category = src.read_u8()?;
        return match category {
            0 => Ok(Self::None),
            1 => Ok(Self::Passive),
            2 => Ok(Self::Basic),
            3 => Ok(Self::Combat),
            4 => Ok(Self::Special),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid category {category}"),
            )),
        };
    }
}

impl Serialize for SkillCategory {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::None => 0,
            Self::Passive => 1,
            Self::Basic => 2,
            Self::Combat => 3,
            Self::Special => 4,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for DamageType {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let damage_type = src.read_u8()?;
        return match damage_type {
            0 => Ok(Self::Fixed),
            1 => Ok(Self::PlusAdditional),
            2 => Ok(Self::Coefficient),
            3 => Ok(Self::CasterCurrentHitpoints),
            4 => Ok(Self::TargetManaPercent),
            5 => Ok(Self::TargetHitpointsPercent),
            6 => Ok(Self::RecCoefficient),
            7 => Ok(Self::RecPlusAdditional),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid damage type {damage_type}"),
            )),
        };
    }
}

impl Serialize for DamageType {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::NotUsed | Self::Fixed => 0,
            Self::PlusAdditional => 1,
            Self::Coefficient => 2,
            Self::CasterCurrentHitpoints => 3,
            Self::TargetManaPercent => 4,
            Self::TargetHitpointsPercent => 5,
            Self::RecCoefficient => 6,
            Self::RecPlusAdditional => 7,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for DamageOverTimeType {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let dot_type = src.read_u8()?;
        return match dot_type {
            0 => Ok(Self::None),
            4 => Ok(Self::Percent),
            12 => Ok(Self::Exponential),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid damage over time type {dot_type}"),
            )),
        };
    }
}

impl Serialize for DamageOverTimeType {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::None => 0,
            Self::Percent => 4,
            Self::Exponential => 8,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for DurationType {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let duration_type = src.read_u8()?;
        return match duration_type {
            0 => Ok(Self::SecondsAndDisappearOnDeath),
            1 => Ok(Self::HoursAndPersistsOnDeath),
            2 => Ok(Self::SecondsAndPersistsOnDeath),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid duration type {duration_type}"),
            )),
        };
    }
}

impl Serialize for DurationType {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::None | Self::SecondsAndDisappearOnDeath => 0,
            Self::HoursAndPersistsOnDeath => 1,
            Self::SecondsAndPersistsOnDeath => 2,
        };
        dst.write_u8(id)
    }
}

fn max_ability_types_for_version(version: GameVersion) -> usize {
    if version >= GameVersion::Ep6 {
        10
    } else {
        3
    }
}

fn ranks_per_skill_for_version(version: GameVersion) -> usize {
    match version {
        GameVersion::Ep4 => 3,
        GameVersion::Ep5 => 5,
        GameVersion::Ep6 | GameVersion::Ep6v4 => 15,
    }
}
