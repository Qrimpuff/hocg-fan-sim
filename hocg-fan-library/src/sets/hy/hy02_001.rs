use hocg_fan_sim::cards::*;

pub fn card() -> Card {
    Card::Cheer(CheerCard {
        card_number: "hY02-001".into(),
        name: "Green Cheer".into(),
        color: Color::Green,
        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
        rarity: Rarity::Common,
        illustration_url: "/hocg-fan-sim-assets/img/hY02/hY02-001.webp".into(),
        artist: "はずき".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn hy02_001() {
        // no need for testing: cheer card
    }
}
