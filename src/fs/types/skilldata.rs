use crate::fs::types::SData;
use crate::io::{Deserialize, GameVersion, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SkillData(BTreeMap<usize, Vec<SkillRecord>>);

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillRecord {
    name: String,
    description: String,
    rank: u8,
    icon: u16,
    animation: u16,
    effect: u8,
    toggle_type: u8,
    sound: u16,
    min_level: u16,
    country: u8,
    usable_by_fighter: bool,
    usable_by_defender: bool,
    usable_by_ranger: bool,
    usable_by_archer: bool,
    usable_by_mage: bool,
    usable_by_priest: bool,
    min_game_mode: u8,
    skill_point_cost: u8,
    category: u8,
    type_attack: u8,
    type_effect: u8,
    type_detail: u16,
    usable_with_one_handed_sword: bool,
    usable_with_two_handed_sword: bool,
    usable_with_one_handed_axe: bool,
    usable_with_two_handed_axe: bool,
    usable_with_dual_swords: bool,
    usable_with_spear: bool,
    usable_with_one_handed_mace: bool,
    usable_with_two_handed_mace: bool,
    usable_with_reverse_sword: bool,
    usable_with_dagger: bool,
    usable_with_javelin: bool,
    usable_with_staff: bool,
    usable_with_bow: bool,
    usable_with_crossbow: bool,
    usable_with_fist_weapon: bool,
    usable_with_shield: bool,
    sp_cost: u16,
    mp_cost: u16,
    cast_time: u8,
    cooldown_duration: u16,
    attack_distance: u8,
    state_type: u8,
    element_type: u8,
    disable: u16,
    prerequisite_skill: u16,
    success_type: u8,
    success_chance: u8,
    target_type: u8,
    radius: u8,
    number_of_multi_hits: u8,
    duration: u16,
    weapon1: u8,
    weapon2: u8,
    weapon_value: u8,
    bag: u8,
    arrow: u16,
    damage_type: u8,
    fixed_damage_hp: u16,
    fixed_damage_sp: u16,
    fixed_damage_mp: u16,
    damage_over_time_type: u8,
    damage_over_time_hp: u16,
    damage_over_time_sp: u16,
    damage_over_time_mp: u16,
    additional_damage_hp: u16,
    additional_damage_sp: u16,
    additional_damage_mp: u16,
    ability_effects: Vec<AbilityEffect>,
    heal_hp: u16,
    heal_sp: u16,
    heal_mp: u16,
    heal_over_time_hp: u16,
    heal_over_time_sp: u16,
    heal_over_time_mp: u16,
    damage_absorb_type: u8,
    damage_absorb_value: u8,
    hp_threshold: u8,
    duration_counting_method: DurationMethod,
    change_type: u16,
    change_value: u16,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbilityEffect {
    ability_type: u8,
    ability_value: u16,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum DurationMethod {
    #[default]
    SecondsDisappearOnDeath = 0,
    HoursStayOnDeath = 1,
    SecondsStayOnDeath = 2,
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

impl Deserialize for SkillRecord {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let name = src.read_length_prefixed_string()?;
        let description = src.read_length_prefixed_string()?;
        let rank = src.read_u8()?;
        let icon = src.read_u16::<LE>()?;
        let animation = src.read_u16::<LE>()?;
        let effect = if version >= GameVersion::Ep6 {
            src.read_u8()?
        } else {
            0
        };
        let toggle_type = src.read_u8()?;
        let sound = src.read_u16::<LE>()?;

        let min_level = src.read_u16::<LE>()?;
        let country = src.read_u8()?;
        let usable_by_fighter = src.read_u8()? == 1;
        let usable_by_defender = src.read_u8()? == 1;
        let usable_by_ranger = src.read_u8()? == 1;
        let usable_by_archer = src.read_u8()? == 1;
        let usable_by_mage = src.read_u8()? == 1;
        let usable_by_priest = src.read_u8()? == 1;
        let min_game_mode = src.read_u8()?;
        let skill_point_cost = src.read_u8()?;
        let category = src.read_u8()?;
        let type_attack = src.read_u8()?; // Passive, Physical, Magic, Ranged
        let type_effect = src.read_u8()?;
        let type_detail = src.read_u16::<LE>()?;
        let usable_with_one_handed_sword = src.read_u8()? == 1;
        let usable_with_two_handed_sword = src.read_u8()? == 1;
        let usable_with_one_handed_axe = src.read_u8()? == 1;
        let usable_with_two_handed_axe = src.read_u8()? == 1;
        let usable_with_dual_swords = src.read_u8()? == 1;
        let usable_with_spear = src.read_u8()? == 1;
        let usable_with_one_handed_mace = src.read_u8()? == 1;
        let usable_with_two_handed_mace = src.read_u8()? == 1;
        let usable_with_reverse_sword = src.read_u8()? == 1;
        let usable_with_dagger = src.read_u8()? == 1;
        let usable_with_javelin = src.read_u8()? == 1;
        let usable_with_staff = src.read_u8()? == 1;
        let usable_with_bow = src.read_u8()? == 1;
        let usable_with_crossbow = src.read_u8()? == 1;
        let usable_with_fist_weapon = src.read_u8()? == 1;
        let usable_with_shield = src.read_u8()? == 1;
        let sp_cost = src.read_u16::<LE>()?;
        let mp_cost = src.read_u16::<LE>()?;
        let cast_time = src.read_u8()?;
        let cooldown_duration = src.read_u16::<LE>()?;
        let attack_distance = src.read_u8()?;
        let state_type = src.read_u8()?;
        let element_type = src.read_u8()?;
        let disable = src.read_u16::<LE>()?;
        let prerequisite_skill = src.read_u16::<LE>()?;
        let success_type = src.read_u8()?;
        let success_chance = src.read_u8()?;

        // 0 = CannotBeCasted
        // 1 = None
        // 2 = Self
        // 3 = Enemy
        // 4 = Party
        // 5 = PartyExceptSelf
        // 6 = AroundCaster
        // 7 = AroundTarget
        // 8 = Raid
        let target_type = src.read_u8()?;
        let radius = src.read_u8()?;
        let number_of_multi_hits = src.read_u8()?;
        let duration = src.read_u16::<LE>()?;
        let weapon1 = src.read_u8()?;
        let weapon2 = src.read_u8()?;
        let weapon_value = src.read_u8()?;
        let bag = src.read_u8()?; // ?
        let arrow = src.read_u16::<LE>()?; // ?
        let damage_type = src.read_u8()?;
        let fixed_damage_hp = src.read_u16::<LE>()?;
        let fixed_damage_sp = src.read_u16::<LE>()?;
        let fixed_damage_mp = src.read_u16::<LE>()?;
        let damage_over_time_type = src.read_u8()?;
        let damage_over_time_hp = src.read_u16::<LE>()?;
        let damage_over_time_sp = src.read_u16::<LE>()?;
        let damage_over_time_mp = src.read_u16::<LE>()?;
        let additional_damage_hp = src.read_u16::<LE>()?;
        let additional_damage_sp = src.read_u16::<LE>()?;
        let additional_damage_mp = src.read_u16::<LE>()?;

        let ability_effects = (0..max_ability_types_for_version(version))
            .map(|_| AbilityEffect::deserialize(src))
            .collect::<Result<Vec<AbilityEffect>, _>>()?;

        let heal_hp = src.read_u16::<LE>()?;
        let heal_sp = src.read_u16::<LE>()?;
        let heal_mp = src.read_u16::<LE>()?;

        let heal_over_time_hp = src.read_u16::<LE>()?;
        let heal_over_time_sp = src.read_u16::<LE>()?;
        let heal_over_time_mp = src.read_u16::<LE>()?;

        let damage_absorb_type = src.read_u8()?; // Eagle Eye = 2, Magic Protector = 3
        let damage_absorb_value = src.read_u8()?; // Fleet Foot = %, Magic Protector = number of attacks

        // used for potential skills
        let hp_threshold = src.read_u8()?;

        let duration_counting_method = src.read_u8()?;
        assert!(duration_counting_method <= 2);
        let duration_counting_method: DurationMethod =
            unsafe { std::mem::transmute(duration_counting_method) };

        let change_type = src.read_u16::<LE>()?;
        let change_value = src.read_u16::<LE>()?;

        Ok(Self {
            name,
            description,
            rank,
            animation,
            icon,
            effect,
            toggle_type,
            sound,
            min_level,
            country,
            usable_by_fighter,
            usable_by_defender,
            usable_by_ranger,
            usable_by_archer,
            usable_by_mage,
            usable_by_priest,
            min_game_mode,
            skill_point_cost,
            category,
            type_attack,
            type_effect,
            type_detail,
            usable_with_one_handed_sword,
            usable_with_two_handed_sword,
            usable_with_one_handed_axe,
            usable_with_two_handed_axe,
            usable_with_dual_swords,
            usable_with_spear,
            usable_with_one_handed_mace,
            usable_with_two_handed_mace,
            usable_with_reverse_sword,
            usable_with_dagger,
            usable_with_javelin,
            usable_with_staff,
            usable_with_bow,
            usable_with_crossbow,
            usable_with_fist_weapon,
            usable_with_shield,
            sp_cost,
            mp_cost,
            cast_time,
            cooldown_duration,
            attack_distance,
            state_type,
            element_type,
            disable,
            prerequisite_skill,
            success_type,
            success_chance,
            target_type,
            radius,
            number_of_multi_hits,
            duration,
            weapon1,
            weapon2,
            weapon_value,
            bag,
            arrow,
            damage_type,
            fixed_damage_hp,
            fixed_damage_sp,
            fixed_damage_mp,
            damage_over_time_type,
            damage_over_time_hp,
            damage_over_time_sp,
            damage_over_time_mp,
            additional_damage_hp,
            additional_damage_sp,
            additional_damage_mp,
            ability_effects,
            heal_hp,
            heal_sp,
            heal_mp,
            heal_over_time_hp,
            heal_over_time_sp,
            heal_over_time_mp,
            damage_absorb_type,
            damage_absorb_value,
            hp_threshold,
            duration_counting_method,
            change_type,
            change_value,
        })
    }
}

impl Deserialize for AbilityEffect {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let ability_type = src.read_u8()?;
        let ability_value = src.read_u16::<LE>()?;
        Ok(Self {
            ability_type,
            ability_value,
        })
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

impl Serialize for SkillRecord {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_length_prefixed_string(&self.name)?;
        dst.write_length_prefixed_string(&self.description)?;
        dst.write_u8(self.rank)?;
        dst.write_u16::<LE>(self.icon)?;
        dst.write_u16::<LE>(self.animation)?;
        if version >= GameVersion::Ep6 {
            dst.write_u8(self.effect)?;
        }
        dst.write_u8(self.toggle_type)?;
        dst.write_u16::<LE>(self.sound)?;
        dst.write_u16::<LE>(self.min_level)?;
        dst.write_u8(self.country)?;
        dst.write_bool(self.usable_by_fighter)?;
        dst.write_bool(self.usable_by_defender)?;
        dst.write_bool(self.usable_by_ranger)?;
        dst.write_bool(self.usable_by_archer)?;
        dst.write_bool(self.usable_by_mage)?;
        dst.write_bool(self.usable_by_priest)?;
        dst.write_u8(self.min_game_mode)?;
        dst.write_u8(self.skill_point_cost)?;
        dst.write_u8(self.category)?;
        dst.write_u8(self.type_attack)?;
        dst.write_u8(self.type_effect)?;
        dst.write_u16::<LE>(self.type_detail)?;
        dst.write_bool(self.usable_with_one_handed_sword)?;
        dst.write_bool(self.usable_with_two_handed_sword)?;
        dst.write_bool(self.usable_with_one_handed_axe)?;
        dst.write_bool(self.usable_with_two_handed_axe)?;
        dst.write_bool(self.usable_with_dual_swords)?;
        dst.write_bool(self.usable_with_spear)?;
        dst.write_bool(self.usable_with_one_handed_mace)?;
        dst.write_bool(self.usable_with_two_handed_mace)?;
        dst.write_bool(self.usable_with_reverse_sword)?;
        dst.write_bool(self.usable_with_dagger)?;
        dst.write_bool(self.usable_with_javelin)?;
        dst.write_bool(self.usable_with_staff)?;
        dst.write_bool(self.usable_with_bow)?;
        dst.write_bool(self.usable_with_crossbow)?;
        dst.write_bool(self.usable_with_fist_weapon)?;
        dst.write_bool(self.usable_with_shield)?;
        dst.write_u16::<LE>(self.sp_cost)?;
        dst.write_u16::<LE>(self.mp_cost)?;
        dst.write_u8(self.cast_time)?;
        dst.write_u16::<LE>(self.cooldown_duration)?;
        dst.write_u8(self.attack_distance)?;
        dst.write_u8(self.state_type)?;
        dst.write_u8(self.element_type)?;
        dst.write_u16::<LE>(self.disable)?;
        dst.write_u16::<LE>(self.prerequisite_skill)?;
        dst.write_u8(self.success_type)?;
        dst.write_u8(self.success_chance)?;
        dst.write_u8(self.target_type)?;
        dst.write_u8(self.radius)?;
        dst.write_u8(self.number_of_multi_hits)?;
        dst.write_u16::<LE>(self.duration)?;
        dst.write_u8(self.weapon1)?;
        dst.write_u8(self.weapon2)?;
        dst.write_u8(self.weapon_value)?;
        dst.write_u8(self.bag)?;
        dst.write_u16::<LE>(self.arrow)?;
        dst.write_u8(self.damage_type)?;
        dst.write_u16::<LE>(self.fixed_damage_hp)?;
        dst.write_u16::<LE>(self.fixed_damage_sp)?;
        dst.write_u16::<LE>(self.fixed_damage_mp)?;
        dst.write_u8(self.damage_over_time_type)?;
        dst.write_u16::<LE>(self.damage_over_time_hp)?;
        dst.write_u16::<LE>(self.damage_over_time_sp)?;
        dst.write_u16::<LE>(self.damage_over_time_mp)?;
        dst.write_u16::<LE>(self.additional_damage_hp)?;
        dst.write_u16::<LE>(self.additional_damage_sp)?;
        dst.write_u16::<LE>(self.additional_damage_mp)?;

        let default_ability = AbilityEffect::default();
        for effect_idx in 0..max_ability_types_for_version(version) {
            if effect_idx >= self.ability_effects.len() {
                default_ability.versioned_serialize(dst, version)?;
            } else {
                self.ability_effects[effect_idx].versioned_serialize(dst, version)?;
            }
        }

        dst.write_u16::<LE>(self.heal_hp)?;
        dst.write_u16::<LE>(self.heal_sp)?;
        dst.write_u16::<LE>(self.heal_mp)?;

        dst.write_u16::<LE>(self.heal_over_time_hp)?;
        dst.write_u16::<LE>(self.heal_over_time_sp)?;
        dst.write_u16::<LE>(self.heal_over_time_mp)?;
        dst.write_u8(self.damage_absorb_type)?;
        dst.write_u8(self.damage_absorb_value)?;
        dst.write_u8(self.hp_threshold)?;
        dst.write_u8(self.duration_counting_method.clone() as u8)?;
        dst.write_u16::<LE>(self.change_type)?;
        dst.write_u16::<LE>(self.change_value)?;

        Ok(())
    }
}

impl Serialize for AbilityEffect {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_u8(self.ability_type)?;
        dst.write_u16::<LE>(self.ability_value)?;
        Ok(())
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
        GameVersion::Ep6 => 15,
    }
}
