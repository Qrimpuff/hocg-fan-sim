use std::{collections::HashMap, sync::OnceLock};

use crate::{
    card_effects::{Action, Condition, Error, ParseEffect, ParseTokens, SerializeEffect, Trigger},
    cards::*,
};
use async_rwlock::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard};
use bincode::{config, Decode, Encode};
use flate2::read::GzDecoder;
use get_size::GetSize;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

fn global_library() -> &'static RwLock<GlobalLibrary> {
    static GLOBAL_LIBRARY: OnceLock<RwLock<GlobalLibrary>> = OnceLock::new();
    GLOBAL_LIBRARY.get_or_init(RwLock::default)
}

pub async fn library() -> RwLockReadGuard<'static, GlobalLibrary> {
    global_library().read().await
}

pub async fn update_library(lib: GlobalLibrary) {
    // only try to update the library when is was not already loaded
    // otherwise, try as best effort
    let guard = global_library().upgradable_read().await;
    if guard.cards.is_empty() {
        let mut guard = RwLockUpgradableReadGuard::upgrade(guard).await;
        *guard = lib;
    } else if let Ok(mut guard) = RwLockUpgradableReadGuard::try_upgrade(guard) {
        *guard = lib;
    } else {
        // mostly a problem when running tests
        warn!("trying to load library when it is in use")
    }
}

pub async fn load_library(bytes: &[u8]) {
    // load from bytes
    let mut decoder = GzDecoder::new(bytes);
    let config = config::standard();
    let library = bincode::decode_from_std_read(&mut decoder, config).unwrap();
    update_library(library).await;
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, Default, GetSize)]
pub struct GlobalLibrary {
    // TODO use a different key because rarity is not include in card number
    pub cards: HashMap<CardNumber, Card>,
}

impl GlobalLibrary {
    /// Any pre-processing of cards that could make my life easier later
    pub fn pre_process(&mut self) {
        // not sure if these are good ideas. might be better to be explicit
        // TODO oshi skill once turn
        // TODO special oshi skill once per game
        // TODO enough holo power to pay the cost for oshi skill
        // TODO enough cheers to perform art for members
        // TODO limited support
        // TODO if you can't select something, it should check that it's there first in condition

        // DON'T REMOVE YET. NOT BEFORE THE FILES ARE MADE
        // default condition to always
        let default_trigger = Trigger::Never;
        let default_condition = Condition::True;
        let default_action = Action::Noop;
        let default_url =
            "https://qrimpuff.github.io/hocg-fan-sim-assets/img/card-back.webp".to_string();
        // let default_damage_mod = DamageModifier::None;
        for card in self.cards.values_mut() {
            match card {
                Card::OshiHoloMember(o) => {
                    if o.illustration_url.is_empty() {
                        o.illustration_url.clone_from(&default_url);
                    };
                    o.skills.iter_mut().for_each(|s| {
                        if s.triggers.is_empty() {
                            s.triggers.push(default_trigger)
                        }
                        if s.condition.is_empty() {
                            s.condition.push(default_condition.clone())
                        }
                        if s.effect.is_empty() {
                            s.effect.push(default_action.clone())
                        }
                    });
                }
                Card::HoloMember(m) => {
                    if m.illustration_url.is_empty() {
                        m.illustration_url.clone_from(&default_url);
                    };
                    m.abilities.iter_mut().for_each(|a| {
                        if a.condition.is_empty() {
                            a.condition.push(default_condition.clone())
                        }
                        if a.effect.is_empty() {
                            a.effect.push(default_action.clone())
                        }
                    });
                    m.arts.iter_mut().for_each(|a| {
                        if a.condition.is_empty() {
                            a.condition.push(default_condition.clone())
                        }
                        if a.effect.is_empty() {
                            a.effect.push(default_action.clone())
                        }
                    })
                }
                Card::Support(s) => {
                    if s.illustration_url.is_empty() {
                        s.illustration_url.clone_from(&default_url);
                    };
                    s.effects.iter_mut().for_each(|s| {
                        if s.triggers.is_empty() {
                            s.triggers.push(default_trigger)
                        }
                        if s.condition.is_empty() {
                            s.condition.push(default_condition.clone())
                        }
                        if s.effect.is_empty() {
                            s.effect.push(default_action.clone())
                        }
                    });
                }
                Card::Cheer(c) => {
                    if c.illustration_url.is_empty() {
                        c.illustration_url.clone_from(&default_url);
                    };
                    // cheers do not have conditions
                }
            }
        }
        // end of: DON'T REMOVE YET. NOT BEFORE THE FILES ARE MADE

        // verify effect serialization consistency (de -> ser -> de)
        fn serialization_round_trip<T>(effect: T) -> crate::card_effects::Result<()>
        where
            T: SerializeEffect + ParseTokens + PartialEq + Clone,
        {
            let string = effect.clone().serialize_effect();
            let de_effect = string.parse_effect::<T>()?;

            if effect != de_effect && Some(de_effect) != T::default_effect() {
                Err(Error::Message(
                    "effect could not do serialization round trip".into(),
                ))
            } else {
                Ok(())
            }
        }
        let mut has_errors = false;
        for card in self.cards.values_mut() {
            match card {
                Card::OshiHoloMember(o) => o.skills.iter_mut().for_each(|s| {
                    if let Err(e) = serialization_round_trip(s.condition.clone()) {
                        error!("{}: {} - condition - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(s.effect.clone()) {
                        error!("{}: {} - effect - {}", o.card_number, s.name, e);
                        has_errors = true;
                    }
                }),
                Card::HoloMember(m) => {
                    m.abilities.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            error!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            error!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    });
                    m.arts.iter_mut().for_each(|a| {
                        if let Err(e) = serialization_round_trip(a.condition.clone()) {
                            error!("{}: {} - condition - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                        if let Err(e) = serialization_round_trip(a.effect.clone()) {
                            error!("{}: {} - effect - {}", m.card_number, a.name, e);
                            has_errors = true;
                        }
                    })
                }
                Card::Support(s) => s.effects.iter_mut().enumerate().for_each(|(i, e)| {
                    if let Err(e) = serialization_round_trip(e.condition.clone()) {
                        error!("{}: {} - condition - {}", s.card_number, i, e);
                        has_errors = true;
                    }
                    if let Err(e) = serialization_round_trip(e.effect.clone()) {
                        error!("{}: {} - effect - {}", s.card_number, i, e);
                        has_errors = true;
                    }
                }),
                Card::Cheer(_) => {} // cheers do not have effects
            }
        }
        if has_errors {
            panic!("effect serialization is not consistent")
        }
    }

    pub fn lookup_card(&self, card_number: &CardNumber) -> &Card {
        self.cards
            .get(card_number)
            .unwrap_or_else(|| panic!("should be in the library: {card_number}"))
    }

    pub fn is_oshi(&self, card_number: &CardNumber) -> bool {
        matches!(self.lookup_card(card_number), Card::OshiHoloMember(_))
    }

    pub fn lookup_oshi(&self, card_number: &CardNumber) -> Option<&OshiHoloMemberCard> {
        if let Card::OshiHoloMember(o) = self.lookup_card(card_number) {
            Some(o)
        } else {
            None
        }
    }

    pub fn is_holo_member(&self, card_number: &CardNumber) -> bool {
        matches!(self.lookup_card(card_number), Card::HoloMember(_))
    }

    pub fn lookup_holo_member(&self, card_number: &CardNumber) -> Option<&HoloMemberCard> {
        if let Card::HoloMember(m) = self.lookup_card(card_number) {
            Some(m)
        } else {
            None
        }
    }

    pub fn is_support(&self, card_number: &CardNumber) -> bool {
        matches!(self.lookup_card(card_number), Card::Support(_))
    }

    pub fn lookup_support(&self, card_number: &CardNumber) -> Option<&SupportCard> {
        if let Card::Support(s) = self.lookup_card(card_number) {
            Some(s)
        } else {
            None
        }
    }

    pub fn is_cheer(&self, card_number: &CardNumber) -> bool {
        matches!(self.lookup_card(card_number), Card::Cheer(_))
    }

    pub fn lookup_cheer(&self, card_number: &CardNumber) -> Option<&CheerCard> {
        if let Card::Cheer(c) = self.lookup_card(card_number) {
            Some(c)
        } else {
            None
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Set {
    number: String,
    name: String,
    // maybe preset decks
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Loadout {
    pub oshi: CardNumber,
    pub main_deck: Vec<CardNumber>,
    pub cheer_deck: Vec<CardNumber>,
    // cosmetic...
}
