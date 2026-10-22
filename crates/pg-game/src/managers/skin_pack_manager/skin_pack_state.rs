//! `SkinPackManager`: the pack table with its keychain-backed purchase and selection state,
//! applying skin values, the skin queries and the monster atlas channel.

use std::sync::OnceLock;

use pg_assets::data::skin_packs::{self, AnimAlias, AtlasImage, Remap, SkinPack, SkinPacksFile, SkinValue};
use pg_assets::data::textures::{self as texture_data, ImageKind, ImageRegion};
use pg_engine::library::Library;
use pg_engine::prefs::{self, KEY_CHAIN_APP_NAME, PURCHASED_SECRET};
use pg_engine::sequence::ns_int_value;
use pg_engine::texture::{Patch, Textures};

use crate::prelude::*;
use super::*;

/// `-[SkinPackManager id]`.
const CLASS_ID: &str = "SkinPackManager";

/// Entries of the pack table at `0x10037d620`.
pub const SKIN_PACK_COUNT: usize = 24;

/// `@"FreeBee"`: the product id of the pack that is never sold (Fishmas).
pub(super) const FREE_PRODUCT_ID: &str = "FreeBee";

/// `@"0"`: the default secret of an option's keychain item.
const DEFAULT_OPTION_SECRET: &str = "0";

/// `@"MonsterAtlas"`: the channel of the two monster slots.
const MONSTER_ATLAS_CHANNEL: &str = "MonsterAtlas";

/// `DAT_1004d8bf0`: page 0 of the monster channel.
const MONSTER_ATLAS_PAGE_0: &str = "atlasTRexGreen0";

/// `DAT_1004f0538`: page 1 of the monster channel.
const MONSTER_ATLAS_PAGE_1: &str = "atlasTRexGreen1";

/// What a `BCSkinPackDef *` that is NULL does in the original: every method dereferences it.
pub(super) const NULL_SKIN_PACK_DEF: &str = "SkinPackManager: NULL BCSkinPackDef";

/// The run-time fields of one `BCSkinOptionDef`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkinOptionState {
    /// `+0x08`: the keychain item holding the selected value index (`BCKeyChain`).
    pub key_chain: Option<Entity>,

    /// `+0x24`: the index of the option's current value.
    pub current_value: i32,
}

/// The run-time fields of one `BCSkinPackDef` and of its options.
#[derive(Clone, Debug, Default)]
pub struct SkinPackState {
    /// `+0x40`: bought, or free.
    pub purchased: bool,

    /// `+0x48`: the keychain item of the purchase (`BCKeyChain`).
    pub key_chain: Option<Entity>,

    /// `+0x54`: the index of the current object.
    pub current_object: i32,

    /// The pack's options, per object.
    pub options: Vec<Vec<SkinOptionState>>,
}

impl SkinPackState {
    /// Option `option_index` of object `object_index`.
    pub fn option(&self, object_index: usize, option_index: usize) -> &SkinOptionState {
        &self.options[object_index][option_index]
    }

    /// Option `option_index` of object `object_index`, mutably.
    pub fn option_mut(&mut self, object_index: usize, option_index: usize) -> &mut SkinOptionState {
        &mut self.options[object_index][option_index]
    }
}

/// `SkinPackManager`'s file-scope state: the list offset `DAT_1007d1e78` and the fields of the
/// pack table (`0x10037d620`) that change at run time.
#[derive(Resource, Clone, Debug, Default)]
pub struct SkinPackManagerStatics {
    /// `DAT_1007d1e78`: the table index of the first pack the list panel shows.
    pub list_offset: i32,

    /// The run-time fields of the packs, in table order (sized on first use).
    pub packs: Vec<SkinPackState>,
}

/// The pack table (`skin-packs.json`), parsed once.
pub fn skin_pack_file() -> &'static SkinPacksFile {
    static FILE: OnceLock<SkinPacksFile> = OnceLock::new();
    FILE.get_or_init(load_skin_pack_file)
}

fn load_skin_pack_file() -> SkinPacksFile {
    skin_packs::load().expect("assets/data/skin-packs.json matches its schema")
}

/// The `BCSkinPackDef`s of the table at `0x10037d620`, in table order.
pub fn skin_pack_defs() -> &'static [SkinPack] {
    &skin_pack_file().packs
}

/// The statics, with the run-time fields of every pack, object and option (all zero at first).
pub(super) fn statics_mut(world: &mut World) -> Mut<'_, SkinPackManagerStatics> {
    let mut statics = world.get_resource_or_insert_with(SkinPackManagerStatics::default);
    if statics.packs.is_empty() {
        statics.packs = zeroed_pack_states();
    }
    statics
}

fn zeroed_pack_states() -> Vec<SkinPackState> {
    let mut states = Vec::new();
    for pack in skin_pack_defs() {
        let mut options = Vec::new();
        for object in &pack.objects {
            let object_options = vec![SkinOptionState::default(); object.options.len()];
            options.push(object_options);
        }

        let state = SkinPackState {
            options,
            ..SkinPackState::default()
        };
        states.push(state);
    }
    states
}

/// `pack->purchased` (`+0x40`).
pub(super) fn pack_purchased(world: &mut World, pack_index: usize) -> bool {
    statics_mut(world).packs[pack_index].purchased
}

/// `pack->keyChain` (`+0x48`).
pub(super) fn pack_key_chain(world: &mut World, pack_index: usize) -> Option<Entity> {
    statics_mut(world).packs[pack_index].key_chain
}

/// `pack->currentSkinObjectIndex` (`+0x54`).
pub(super) fn pack_current_object(world: &mut World, pack_index: usize) -> i32 {
    statics_mut(world).packs[pack_index].current_object
}

fn set_pack_current_object(world: &mut World, pack_index: usize, object_index: i32) {
    statics_mut(world).packs[pack_index].current_object = object_index;
}

/// The run-time fields of an option.
pub(super) fn option_state(world: &mut World, pack_index: usize, object_index: usize, option_index: usize) -> SkinOptionState {
    let statics = statics_mut(world);
    *statics.packs[pack_index].option(object_index, option_index)
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
pub(super) fn set_sequence(world: &mut World, this: Entity, id: &str) {
    let sequence = world.obj::<SkinPackManager>(this).sequence;
    let def = pg_engine::library::sequence_def(world, id);
    if let Some(sequence) = sequence {
        pg_engine::sequence::set_sequence_def(world, sequence, def);
    }
}

/// `[logic setBehavior:behavior]` (nothing for a nil logic).
pub(super) fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        pg_engine::logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `[[keyChain secret] intValue]` (0 for a nil keychain).
fn key_chain_int_value(world: &World, key_chain: Option<Entity>) -> i32 {
    let Some(key_chain) = key_chain else {
        return 0;
    };
    let secret = prefs::key_chain_secret(world, key_chain);
    ns_int_value(&secret)
}

/// `[keyChain reset]` (nothing for a nil keychain).
fn reset_key_chain(world: &mut World, key_chain: Option<Entity>) {
    if let Some(key_chain) = key_chain {
        prefs::key_chain_reset(world, key_chain);
    }
}

/// `FUN_100038d68(image, @"raw", slot)`: uploads `<image>.raw` into an atlas slot.
fn load_atlas_slot(world: &mut World, slot_id: &str, image: &str) {
    let atlas = world.resource::<Library>().atlas_id(slot_id);
    let Some(atlas) = atlas else {
        panic!("SkinPackManager: there is no atlas slot {slot_id}");
    };
    pg_engine::texture::load_atlas(world, atlas, image);
}

/// The slot a texture def samples now (`*def->atlas`); remaps can change it.
fn texture_def_atlas(world: &World, texture_def_id: &str) -> Option<AtlasId> {
    let texture_def = pg_engine::library::texture_def_id(world, texture_def_id)?;
    world.resource::<Textures>().def(texture_def).atlas
}

/// The sub-images of every bits file (`textures.json` `images[].regions`), parsed once.
fn bits_image_regions(image: &str) -> &'static [ImageRegion] {
    static REGIONS: OnceLock<HashMap<String, Vec<ImageRegion>>> = OnceLock::new();
    let regions = REGIONS.get_or_init(load_bits_image_regions);
    match regions.get(image) {
        Some(image_regions) => image_regions,
        None => &[],
    }
}

fn load_bits_image_regions() -> HashMap<String, Vec<ImageRegion>> {
    let file = texture_data::load().expect("assets/data/textures.json matches its schema");
    let mut regions = HashMap::new();
    for image in file.images {
        if image.kind == ImageKind::Bits {
            regions.insert(image.name, image.regions);
        }
    }
    regions
}

/// `FUN_100038f24`: applies a skin option value (`docs/re/textures.md` §7.2): remaps, bits,
/// atlas images, animation aliases, then the atlas image.
fn activate_skin_option_value_def(world: &mut World, value: &SkinValue) {
    let file = skin_pack_file();

    if let Some(remap_set) = value.remap_set {
        remap_texture_defs(world, &file.remap_sets[remap_set]);
    }

    if let Some(bits_image) = &value.bits_image {
        activate_skin_texture_bits_batch_def(world, bits_image);
    }

    if !value.atlas_images.is_empty() {
        activate_skin_atlas_images(world, &value.atlas_images);
    }

    if let Some(anim_alias_set) = value.anim_alias_set {
        activate_anim_aliases(world, &file.anim_alias_sets[anim_alias_set]);
    }

    if let (Some(atlas_image), Some(atlas)) = (&value.atlas_image, &value.atlas) {
        load_atlas_slot(world, atlas, atlas_image);
    }
}

/// Step 1 of `FUN_100038f24`: copies bytes 0x10–0x7f of each source def over its target.
fn remap_texture_defs(world: &mut World, remap_set: &[Remap]) {
    for remap in remap_set {
        let target = pg_engine::library::texture_def_id(world, &remap.target);
        let source = pg_engine::library::texture_def_id(world, &remap.source);
        let (Some(target), Some(source)) = (target, source) else {
            continue;
        };

        // The copy starts at +0x10: the target keeps its id and its has_alpha flag (+0x08),
        // which `copy_texture_def` would take from the source.
        let target_blend = world.resource::<Textures>().def(target).blend;
        pg_engine::texture::copy_texture_def(world, target, source);
        world.resource_mut::<Textures>().def_mut(target).blend = target_blend;
    }
}

/// `FUN_1000384b0` (`activateSkinTextureBitsBatchDef`): uploads each sub-image of `<file>.raw`
/// into the rect of its record's def, in the slot that def samples now.
pub(crate) fn activate_skin_texture_bits_batch_def(world: &mut World, file: &str) {
    for region in bits_image_regions(file) {
        let destination = match &region.texture {
            Some(texture_def_id) => texture_def_atlas(world, texture_def_id),
            None => None,
        };
        let (Some(atlas), Some(x), Some(y)) = (destination, region.x, region.y) else {
            continue;
        };

        let patch = Patch {
            image: file,
            src_x: region.src_x,
            src_y: region.src_y,
            width: region.width,
            height: region.height,
            dst_x: x as u32,
            dst_y: y as u32,
        };
        pg_engine::texture::patch_atlas(world, atlas, patch);
    }
}

/// `FUN_100038a20`: loads each `{file, format, slotDef}` record's image into the slot its def
/// samples (RGBA4444 at the slot's size).
fn activate_skin_atlas_images(world: &mut World, atlas_images: &[AtlasImage]) {
    for record in atlas_images {
        let atlas = texture_def_atlas(world, &record.atlas);
        if let Some(atlas) = atlas {
            pg_engine::texture::load_atlas(world, atlas, &record.image);
        }
    }
}

/// `FUN_100038dc0`: points each alias at its animation and, when the record names a display
/// group, gives that object's logic the record's behavior.
fn activate_anim_aliases(world: &mut World, anim_aliases: &[AnimAlias]) {
    for record in anim_aliases {
        let animation = world.resource::<Library>().animation_def(&record.animation);
        world.resource_mut::<Library>().set_animation_def_alias(&record.alias, animation);

        if record.display_group.is_empty() {
            continue;
        }
        let display_group = pg_engine::library::display_group(world, &record.display_group);
        let display_object = match display_group {
            Some(display_group) => pg_engine::display::group::get_display_object(world, display_group, &record.display_object),
            None => None,
        };
        let logic = match display_object {
            Some(display_object) => pg_engine::display::object(world, display_object).logic,
            None => None,
        };
        set_logic_behavior(world, logic, &record.behavior);
    }
}

/// `-[SkinPackManager init]` @ 0x10005cad4
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(SkinPackManager::default());

    let mut singletons = world.get_resource_or_insert_with(Singletons::default);
    singletons.skin_pack_manager = Some(this);

    let sequence = pg_engine::sequence::spawn(world, None, Some(this));
    {
        let mut manager = world.obj_mut::<SkinPackManager>(this);
        manager.sequence = Some(sequence);
        manager.skin_option_state_dictionary = HashMap::with_capacity(5);
        manager.skin_option_channel_dictionary = HashMap::with_capacity(5);
        manager.logic_array = Vec::with_capacity(5);
        manager.skin_pack_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_list_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_customize_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_purchase_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_success_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_failure_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_error_ui_logic_array = Vec::with_capacity(5);
        manager.skin_pack_info_ui_logic_array = Vec::with_capacity(5);
    }

    init_skin_pack_defs(world, this);

    let mut all_skin_packs_are_purchased = true;
    for pack_index in 0..SKIN_PACK_COUNT {
        let purchased = pack_purchased(world, pack_index);
        if !purchased {
            all_skin_packs_are_purchased = false;
        }
    }
    world.obj_mut::<SkinPackManager>(this).all_skin_packs_are_purchased = all_skin_packs_are_purchased;

    // PORT: `[[SKPaymentQueue defaultQueue] addTransactionObserver:self]`. There is no store
    // (`docs/ARCHITECTURE.md`), so no transaction is ever reported.
}

/// `-[SkinPackManager getSkinPackDefByProductIdentifier:]` @ 0x10005cdd8
pub fn get_skin_pack_def_by_product_identifier(_world: &mut World, _this: Entity, product_identifier: &str) -> Option<usize> {
    let packs = &skin_pack_defs()[..SKIN_PACK_COUNT];
    packs.iter().position(|pack| pack.product_id == product_identifier)
}

/// `-[SkinPackManager getSkinPackDefById:]` @ 0x10005ce4c
pub fn get_skin_pack_def_by_id(_world: &mut World, _this: Entity, skin_pack_id: &str) -> Option<usize> {
    let packs = &skin_pack_defs()[..SKIN_PACK_COUNT];
    packs.iter().position(|pack| pack.id == skin_pack_id)
}

/// `-[SkinPackManager loadTextureAtlasChannel:]` @ 0x10005cec0
pub fn load_texture_atlas_channel(world: &mut World, this: Entity, channel: i32) {
    let (owner, page_0_image, page_1_image) = match channel {
        0 => ("IconPage", "IconPageTextureAtlas0", "IconPageTextureAtlas0"),
        1 => ("IdolPage", "IdolPageTextureAtlas0", "IdolPageTextureAtlas0"),
        2 => ("Disco", "DiscoTextureAtlas0", "DiscoTextureAtlas1"),
        3 => ("Xenu", "XenuTextureAtlas0", "XenuTextureAtlas1"),
        _ => return,
    };

    let manager = world.obj::<SkinPackManager>(this);
    let current_owner = manager.skin_option_channel_dictionary.get(MONSTER_ATLAS_CHANNEL);
    let already_loaded = current_owner.is_some_and(|current_owner| current_owner == owner);
    if already_loaded {
        return;
    }

    {
        let mut manager = world.obj_mut::<SkinPackManager>(this);
        let channels = &mut manager.skin_option_channel_dictionary;
        channels.insert(MONSTER_ATLAS_CHANNEL.to_owned(), owner.to_owned());
    }

    load_atlas_slot(world, MONSTER_ATLAS_PAGE_0, page_0_image);
    load_atlas_slot(world, MONSTER_ATLAS_PAGE_1, page_1_image);
}

/// `-[SkinPackManager getSkinPackOptionValueBySkinPackId:skinObjectId:skinOptionId:]` @ 0x10005d14c
pub fn get_skin_pack_option_value_by_skin_pack_id(
    world: &mut World,
    this: Entity,
    skin_pack_id: &str,
    skin_object_id: &str,
    skin_option_id: &str,
) -> Option<String> {
    let pack_index = get_skin_pack_def_by_id(world, this, skin_pack_id);
    let pack_index = pack_index.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    let object = pack.objects.iter().find(|object| object.id == skin_object_id)?;
    let option = object.options.iter().find(|option| option.id == skin_option_id)?;

    let manager = world.obj::<SkinPackManager>(this);
    let state = manager.skin_option_state_dictionary.get(&option.id);
    for (value_index, value) in option.values.iter().enumerate() {
        let key = value_index.to_string();
        if state == Some(&key) {
            return Some(value.id.clone());
        }
    }
    None
}

/// `-[SkinPackManager isSkinPackOptionActiveBySkinPackId:skinObjectId:skinOptionId:skinOptionValueId:]` @ 0x10005d2c4
pub fn is_skin_pack_option_active_by_skin_pack_id(
    world: &mut World,
    this: Entity,
    skin_pack_id: &str,
    skin_object_id: &str,
    skin_option_id: &str,
    skin_option_value_id: &str,
) -> bool {
    let pack_index = get_skin_pack_def_by_id(world, this, skin_pack_id);
    let pack_index = pack_index.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    let Some(object) = pack.objects.iter().find(|object| object.id == skin_object_id) else {
        return false;
    };
    let Some(option) = object.options.iter().find(|option| option.id == skin_option_id) else {
        return false;
    };
    let Some(value_index) = option.values.iter().position(|value| value.id == skin_option_value_id) else {
        return false;
    };

    let key = value_index.to_string();
    let manager = world.obj::<SkinPackManager>(this);
    let state = manager.skin_option_state_dictionary.get(&option.id);
    state == Some(&key)
}

/// `-[SkinPackManager refreshSkinPackOption:skinObjectId:skinOptionId:useFirstPurchased:]` @ 0x10005d450
///
/// The first purchased pack in table order that has the object and the option replaces
/// `skin_pack_id`; `use_first_purchased` is never read.
pub fn refresh_skin_pack_option_skin_object_id_skin_option_id_use_first_purchased(
    world: &mut World,
    this: Entity,
    skin_pack_id: &str,
    skin_object_id: &str,
    skin_option_id: &str,
    _use_first_purchased: bool,
) {
    let mut refresh_pack_id = skin_pack_id;
    for pack_index in 0..SKIN_PACK_COUNT {
        let purchased = pack_purchased(world, pack_index);
        if !purchased {
            continue;
        }

        let pack = &skin_pack_defs()[pack_index];
        let Some(object) = pack.objects.iter().find(|object| object.id == skin_object_id) else {
            continue;
        };
        let has_option = object.options.iter().any(|option| option.id == skin_option_id);
        if has_option {
            refresh_pack_id = &pack.id;
            break;
        }
    }

    refresh_skin_pack_option_skin_object_id_skin_option_id(world, this, refresh_pack_id, skin_object_id, skin_option_id);
}

/// `-[SkinPackManager refreshSkinPackOption:skinObjectId:skinOptionId:]` @ 0x10005d57c
pub fn refresh_skin_pack_option_skin_object_id_skin_option_id(
    world: &mut World,
    this: Entity,
    skin_pack_id: &str,
    skin_object_id: &str,
    skin_option_id: &str,
) {
    let pack_index = get_skin_pack_def_by_id(world, this, skin_pack_id);
    let pack_index = pack_index.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    let Some(object_index) = pack.objects.iter().position(|object| object.id == skin_object_id) else {
        return;
    };
    let object = &pack.objects[object_index];
    let Some(option_index) = object.options.iter().position(|option| option.id == skin_option_id) else {
        return;
    };

    set_pack_current_object(world, pack_index, object_index as i32);
    let current_value = option_state(world, pack_index, object_index, option_index).current_value;
    update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
        world,
        this,
        Some(pack_index),
        option_index as i32,
        current_value,
        true,
        false,
    );
}

/// `-[SkinPackManager initSkinPackDefs]` @ 0x10005d694
pub fn init_skin_pack_defs(world: &mut World, this: Entity) {
    for pack_index in 0..SKIN_PACK_COUNT {
        let pack = &skin_pack_defs()[pack_index];
        statics_mut(world).packs[pack_index].purchased = false;

        let key_chain = prefs::key_chain_with_key_using_app_name(world, KEY_CHAIN_APP_NAME, &pack.keychain_key);
        statics_mut(world).packs[pack_index].key_chain = Some(key_chain);

        let purchased = if pack.product_id == FREE_PRODUCT_ID {
            true
        } else {
            let secret = prefs::key_chain_secret(world, key_chain);
            secret == PURCHASED_SECRET
        };
        if purchased {
            statics_mut(world).packs[pack_index].purchased = true;
        }
    }

    for pack_index in 0..SKIN_PACK_COUNT {
        init_skin_pack_def(world, this, Some(pack_index));
    }
}

/// `-[SkinPackManager initSkinPackDef:]` @ 0x10005d7c8
pub fn init_skin_pack_def(world: &mut World, this: Entity, skin_pack_def: Option<usize>) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    for (object_index, object) in pack.objects.iter().enumerate() {
        set_pack_current_object(world, pack_index, object_index as i32);

        for (option_index, option) in object.options.iter().enumerate() {
            let key_chain = prefs::key_chain_with_key_using_app_name_default_secret(
                world,
                KEY_CHAIN_APP_NAME,
                &option.id,
                DEFAULT_OPTION_SECRET,
            );
            {
                let mut statics = statics_mut(world);
                let stored_option = statics.packs[pack_index].option_mut(object_index, option_index);
                stored_option.key_chain = Some(key_chain);
            }

            let value_index = key_chain_int_value(world, Some(key_chain));
            update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
                world,
                this,
                Some(pack_index),
                option_index as i32,
                value_index,
                true,
                true,
            );
        }
    }

    set_pack_current_object(world, pack_index, 0);
}

/// `-[SkinPackManager clearUnpurchasedSkinOptionsFromSkinPackDef:]` @ 0x10005d94c
pub fn clear_unpurchased_skin_options_from_skin_pack_def(world: &mut World, this: Entity, skin_pack_def: Option<usize>) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    for (object_index, object) in pack.objects.iter().enumerate() {
        set_pack_current_object(world, pack_index, object_index as i32);
        for option_index in 0..object.options.len() {
            clear_unpurchased_skin_pack_def(world, this, Some(pack_index), option_index as i32);
        }
    }

    set_pack_current_object(world, pack_index, 0);
}

/// `-[SkinPackManager resetKeyChainsFromSkinPackDef:]` @ 0x10005d9ec
pub fn reset_key_chains_from_skin_pack_def(world: &mut World, _this: Entity, skin_pack_def: Option<usize>) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    let pack_key_chain = pack_key_chain(world, pack_index);
    reset_key_chain(world, pack_key_chain);

    for (object_index, object) in pack.objects.iter().enumerate() {
        for option_index in 0..object.options.len() {
            let key_chain = option_state(world, pack_index, object_index, option_index).key_chain;
            reset_key_chain(world, key_chain);
        }
    }
}

/// `-[SkinPackManager resetKeyChainsForAllSkinPackDefs]` @ 0x10005da8c
pub fn reset_key_chains_for_all_skin_pack_defs(world: &mut World, this: Entity) {
    for pack_index in 0..SKIN_PACK_COUNT {
        reset_key_chains_from_skin_pack_def(world, this, Some(pack_index));
    }
}

/// `-[SkinPackManager releaseKeyChainsFromSkinPackDef:]` @ 0x10005dae4
pub fn release_key_chains_from_skin_pack_def(world: &mut World, _this: Entity, skin_pack_def: Option<usize>) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);

    let mut statics = statics_mut(world);
    let pack = &mut statics.packs[pack_index];
    pack.key_chain = None;
    for object_options in &mut pack.options {
        for option in object_options {
            option.key_chain = None;
        }
    }
}

/// `-[SkinPackManager id]` @ 0x10005db8c
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[SkinPackManager setSkinPackManagerListener:]` @ 0x10005db98
pub fn set_skin_pack_manager_listener(world: &mut World, this: Entity, skin_pack_manager_listener: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_manager_listener = Some(skin_pack_manager_listener);
}

/// `-[SkinPackManager dealloc]` @ 0x10005dbf4
pub fn dealloc(world: &mut World, this: Entity) {
    for pack_index in 0..SKIN_PACK_COUNT {
        release_key_chains_from_skin_pack_def(world, this, Some(pack_index));
    }

    let sequence = world.obj::<SkinPackManager>(this).sequence;
    if let Some(sequence) = sequence {
        pg_engine::sequence::stop(world, sequence);
        world.despawn(sequence);
    }

    world.despawn(this);
}

/// `-[SkinPackManager skinOptionValueIndex:]` @ 0x100060068
pub fn skin_option_value_index(world: &mut World, this: Entity, skin_option_index: i32) -> i32 {
    let active_skin_pack_def = world.obj::<SkinPackManager>(this).active_skin_pack_def;
    let pack_index = active_skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let object_index = pack_current_object(world, pack_index) as usize;
    let option_index = skin_option_index as usize;
    option_state(world, pack_index, object_index, option_index).current_value
}

/// `-[SkinPackManager updateSkinPackDef:skinObjectIndex:silent:]` @ 0x100060090
pub fn update_skin_pack_def_skin_object_index_silent(
    world: &mut World,
    this: Entity,
    skin_pack_def: Option<usize>,
    skin_object_index: i32,
    silent: bool,
) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];
    let object_count = pack.objects.len() as i32;

    let mut object_index = skin_object_index;
    if object_count <= object_index {
        object_index = 0;
    }
    if object_index < 0 {
        object_index = object_count - 1;
    }

    let current_object = pack_current_object(world, pack_index);
    if object_index == current_object && !silent {
        return;
    }
    set_pack_current_object(world, pack_index, object_index);

    let object_index = object_index as usize;
    let object = &pack.objects[object_index];
    let mut value_behavior_set = false;
    for (option_index, option) in object.options.iter().enumerate() {
        let current_value = option_state(world, pack_index, object_index, option_index).current_value;
        let value = &option.values[current_value as usize];
        if let Some(behavior) = &value.behavior {
            let customize_item = world.obj::<SkinPackManager>(this).skin_pack_customize_item_logic;
            set_logic_behavior(world, customize_item, behavior);
            value_behavior_set = true;
        }
    }
    if !value_behavior_set {
        let customize_item = world.obj::<SkinPackManager>(this).skin_pack_customize_item_logic;
        set_logic_behavior(world, customize_item, &object.behavior);
    }

    if silent {
        return;
    }
    set_sequence(world, this, "UpdateSkinObject");
}

/// `-[SkinPackManager updateSkinPackDef:skinOptionIndex:skinOptionValueIndex:silent:init:]` @ 0x1000601e4
pub fn update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
    world: &mut World,
    this: Entity,
    skin_pack_def: Option<usize>,
    skin_option_index: i32,
    skin_option_value_index: i32,
    silent: bool,
    init: bool,
) -> bool {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];
    let object_index = pack_current_object(world, pack_index) as usize;
    let option_index = skin_option_index as usize;
    let option = &pack.objects[object_index].options[option_index];

    // `sdiv` (0 for a count of 0), then a negative remainder wraps once.
    let value_count = option.values.len() as i32;
    let quotient = match value_count {
        0 => 0,
        _ => skin_option_value_index / value_count,
    };
    let remainder = skin_option_value_index - quotient * value_count;
    let value_index = if remainder < 0 { remainder + value_count } else { remainder };

    let current_value = option_state(world, pack_index, object_index, option_index).current_value;
    if value_index == current_value && !silent {
        return true;
    }

    let value = &option.values[value_index as usize];
    if let Some(required_pack_id) = &value.required_pack {
        let required_pack = get_skin_pack_def_by_id(world, this, required_pack_id);
        let required_pack = required_pack.expect(NULL_SKIN_PACK_DEF);
        let required_pack_purchased = pack_purchased(world, required_pack);
        let purchased = pack_purchased(world, pack_index);
        if !required_pack_purchased || !purchased {
            return false;
        }
    }

    {
        let mut statics = statics_mut(world);
        let stored_option = statics.packs[pack_index].option_mut(object_index, option_index);
        stored_option.current_value = value_index;
    }
    let key = value_index.to_string();

    let manager = world.obj::<SkinPackManager>(this);
    let state = manager.skin_option_state_dictionary.get(&option.id);
    let channel_owner = match &option.channel {
        Some(channel) => manager.skin_option_channel_dictionary.get(channel),
        None => None,
    };
    let already_applied = state == Some(&key) && channel_owner == Some(&option.id);

    if !already_applied && (!init || option.activate_at_init) {
        {
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_option_state_dictionary.insert(option.id.clone(), key.clone());
            if let Some(channel) = &option.channel {
                manager.skin_option_channel_dictionary.insert(channel.clone(), option.id.clone());
            }
        }

        activate_skin_option_value_def(world, value);

        if let Some(behavior) = &value.behavior {
            let customize_item = world.obj::<SkinPackManager>(this).skin_pack_customize_item_logic;
            set_logic_behavior(world, customize_item, behavior);
        }
    }

    if !silent {
        let purchased = pack_purchased(world, pack_index);
        if purchased {
            let key_chain = option_state(world, pack_index, object_index, option_index).key_chain;
            if let Some(key_chain) = key_chain {
                prefs::key_chain_set_secret(world, key_chain, &key);
            }
        }
        set_sequence(world, this, "UpdateSkinOptionValue");
    }
    true
}

/// `-[SkinPackManager clearUnpurchasedSkinPackDef:skinOptionIndex:]` @ 0x10006042c
pub fn clear_unpurchased_skin_pack_def(world: &mut World, this: Entity, skin_pack_def: Option<usize>, skin_option_index: i32) {
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];
    let object_index = pack_current_object(world, pack_index) as usize;
    let option_index = skin_option_index as usize;
    let option = &pack.objects[object_index].options[option_index];

    let key_chain = option_state(world, pack_index, object_index, option_index).key_chain;
    let value_index = key_chain_int_value(world, key_chain);
    {
        let mut statics = statics_mut(world);
        let stored_option = statics.packs[pack_index].option_mut(object_index, option_index);
        stored_option.current_value = value_index;
    }

    let purchased = pack_purchased(world, pack_index);
    if purchased {
        return;
    }

    let key = value_index.to_string();
    let manager = world.obj::<SkinPackManager>(this);
    let state = manager.skin_option_state_dictionary.get(&option.id);
    if state == Some(&key) {
        return;
    }
    if let Some(channel) = &option.channel {
        let channel_owner = manager.skin_option_channel_dictionary.get(channel);
        if channel_owner != Some(&option.id) {
            return;
        }
    }

    {
        let mut manager = world.obj_mut::<SkinPackManager>(this);
        manager.skin_option_state_dictionary.insert(option.id.clone(), key);
    }

    let value = &option.values[value_index as usize];
    activate_skin_option_value_def(world, value);

    if let Some(behavior) = &value.behavior {
        let customize_item = world.obj::<SkinPackManager>(this).skin_pack_customize_item_logic;
        set_logic_behavior(world, customize_item, behavior);
    }
}

/// `-[SkinPackManager activateSkinPackDef:]` @ 0x1000605b0
pub fn activate_skin_pack_def(world: &mut World, this: Entity, skin_pack_def: Option<usize>) {
    world.obj_mut::<SkinPackManager>(this).active_skin_pack_def = skin_pack_def;
    let pack_index = skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let pack = &skin_pack_defs()[pack_index];

    for (object_index, object) in pack.objects.iter().enumerate() {
        set_pack_current_object(world, pack_index, object_index as i32);

        for option_index in 0..object.options.len() {
            let purchased = pack_purchased(world, pack_index);
            let value_index = if purchased {
                let key_chain = option_state(world, pack_index, object_index, option_index).key_chain;
                key_chain_int_value(world, key_chain)
            } else {
                0
            };
            update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
                world,
                this,
                Some(pack_index),
                option_index as i32,
                value_index,
                true,
                false,
            );
        }
    }

    set_pack_current_object(world, pack_index, 0);
    let current_object = pack_current_object(world, pack_index);
    update_skin_pack_def_skin_object_index_silent(world, this, Some(pack_index), current_object, true);
    set_sequence(world, this, "OpenSkinPackScreenCustomize");
}

/// `+[SkinPackManager instance]` @ 0x10005cac4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().skin_pack_manager
}
