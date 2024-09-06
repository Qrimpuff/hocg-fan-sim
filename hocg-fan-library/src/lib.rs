use hocg_fan_sim::cards::GlobalLibrary;

mod sets;

pub fn setup_library() -> GlobalLibrary {
    let mut library = GlobalLibrary::default();
    sets::append_sets(&mut library);
    library.pre_process();

    // TODO
    // // verify effect serialization consistency (ser -> de -> ser), check that tokens were used
    // for string in effects {
    //     let effect = string
    //         .parse_effect::<CardEffect>()
    //         .expect("should already be parsed above");
    //     let de_string = effect.serialize_effect();
    //     let default_string = CardEffect::default_effect().map(|e| e.serialize_effect());

    //     let string = string.replace(['(', ')', ' ', '\n', '\r'], "");
    //     let de_string = de_string.replace(['(', ')', ' ', '\n', '\r'], "");
    //     if Some(de_string.clone()) != default_string {
    //         assert_eq!(string, de_string);
    //     }
    // }
    // for string in conditions {
    //     let condition = string
    //         .parse_effect::<CardEffectCondition>()
    //         .expect("should already be parsed above");
    //     let de_string = condition.serialize_effect();
    //     let default_string =
    //         CardEffectCondition::default_effect().map(|e| e.serialize_effect());

    //     let string = string.replace(['(', ')', ' ', '\n', '\r'], "");
    //     let de_string = de_string.replace(['(', ')', ' ', '\n', '\r'], "");
    //     if Some(de_string.clone()) != default_string {
    //         assert_eq!(string, de_string);
    //     }
    // }

    library
}
