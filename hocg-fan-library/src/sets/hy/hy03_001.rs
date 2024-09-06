use hocg_fan_sim::cards::*;

pub fn card() -> Card {
    Card::Cheer(CheerCard {
        card_number: "hY03-001".into(),
        name: "Red Cheer".into(),
        color: Color::Red,
        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hY03/hY03-001.webp".into(),
        artist: "はずき".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn hy03_001() {
        // no need for testing: cheer card
    }
}
