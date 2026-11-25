use chrono::Duration;
use kallbet_ai::adapters::context::liquipedia::parse_revision_rosters;
use kallbet_ai::adapters::context::rss::{filter_match_news, parse_rss_items};

const SINGAPORE_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json");
const SINGAPORE_STAGE1_REV: &str =
    include_str!("fixtures/context/liquipedia/pgl-singapore-stage1-before-start.json");
const HLTV_RSS: &str = include_str!("fixtures/context/rss/hltv-news.xml");
const DUST2US_RSS: &str = include_str!("fixtures/context/rss/dust2us-news.xml");
const DUST2DK_RSS: &str = include_str!("fixtures/context/rss/dust2dk-news.xml");
const STEAM_RSS: &str = include_str!("fixtures/context/rss/steam-cs2-news.xml");

#[test]
fn liquipedia_parser_extracts_active_lineups_coaches_stand_ins_and_attribution() {
    let cutoff = "2026-11-24T00:00:00Z".parse().unwrap();
    let parsed = parse_revision_rosters(SINGAPORE_REV, cutoff)
        .unwrap()
        .expect("revision strictly before cutoff should parse");

    let rev_ts: kallbet_ai::Instant = "2026-11-23T19:49:30Z".parse().unwrap();
    assert_eq!(parsed.observed_at(), rev_ts);
    let event = parsed.at(cutoff).unwrap();
    assert_eq!(event.page_title, "PGL/2026/Singapore");
    assert_eq!(event.revid, 3473596);
    assert_eq!(event.revision_timestamp, rev_ts);
    assert_eq!(
        event.source_url,
        "https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473596"
    );
    assert_eq!(event.attribution.source, "Liquipedia");
    assert_eq!(event.attribution.license, "CC-BY-SA 3.0");
    assert_eq!(
        event.attribution.license_url,
        "https://creativecommons.org/licenses/by-sa/3.0/"
    );

    // PGL Major Singapore 2026 has 32 participating teams, each with 5 active players.
    assert_eq!(event.teams.len(), 32);
    for team in &event.teams {
        assert_eq!(
            team.active_players.len(),
            5,
            "expected 5 active players for {}, got {:?}",
            team.team,
            team.active_players
        );
    }

    // Team Vitality: jL benched, apEX active stand-in, XTQZZZ coach.
    let vitality = event
        .teams
        .iter()
        .find(|t| t.team == "Team Vitality")
        .unwrap();
    assert_eq!(
        vitality.active_players,
        ["ZywOo", "flameZ", "mezii", "ropz", "apEX"]
    );
    assert_eq!(vitality.benched_players, ["jL"]);
    assert_eq!(vitality.stand_ins.len(), 1);
    assert_eq!(vitality.stand_ins[0].player, "apEX");
    assert_eq!(vitality.stand_ins[0].from_team, None);
    assert_eq!(vitality.coaches.len(), 1);
    assert_eq!(vitality.coaches[0].player, "XTQZZZ");
    assert!(!vitality.coaches[0].emergency_sub);

    // Astralis: jabbi benched, Anlelele active stand-in from Sashi Esport, ryu with link=.
    let astralis = event.teams.iter().find(|t| t.team == "Astralis").unwrap();
    assert_eq!(
        astralis.active_players,
        ["Staehr", "phzy", "ryu", "siuhy", "Anlelele"]
    );
    assert_eq!(astralis.benched_players, ["jabbi"]);
    assert_eq!(astralis.stand_ins.len(), 1);
    assert_eq!(astralis.stand_ins[0].player, "Anlelele");
    assert_eq!(
        astralis.stand_ins[0].from_team.as_deref(),
        Some("Sashi Esport")
    );
    assert_eq!(astralis.coaches[0].player, "NEO");

    // MOUZ: 5 starters, sycrone coach, Nikodeon unused sub (not in active lineup).
    let mouz = event.teams.iter().find(|t| t.team == "MOUZ").unwrap();
    assert_eq!(
        mouz.active_players,
        ["torzsi", "xertioN", "Spinx", "xelex", "PR"]
    );
    assert!(mouz.stand_ins.is_empty());
    assert!(mouz.benched_players.is_empty());
    assert_eq!(mouz.unused_substitutes, ["Nikodeon"]);
    assert_eq!(mouz.coaches[0].player, "sycrone");

    // Wildcard: positional handle before role=coach, csub=true.
    let wildcard = event.teams.iter().find(|t| t.team == "Wildcard").unwrap();
    assert_eq!(wildcard.coaches.len(), 1);
    assert_eq!(wildcard.coaches[0].player, "splawik");
    assert!(wildcard.coaches[0].emergency_sub);

    // The One and GOSH: forced rename aliases and citation URLs extracted from notes.
    let the_one = event.teams.iter().find(|t| t.team == "The One").unwrap();
    assert!(
        the_one.aliases.iter().any(|a| a == "1win Team"),
        "aliases: {:?}",
        the_one.aliases
    );
    assert_eq!(
        the_one.notes[0].source_url.as_deref(),
        Some("https://www.hltv.org/news/45947/pgl-announces-name-changes-for-four-teams-at-singapore-major")
    );
    let gosh = event.teams.iter().find(|t| t.team == "GOSH").unwrap();
    assert!(
        gosh.aliases.iter().any(|a| a == "BC.Game Esports"),
        "aliases: {:?}",
        gosh.aliases
    );
}

#[test]
fn liquipedia_parser_excludes_revisions_at_or_after_cutoff() {
    let rev_ts: chrono::DateTime<chrono::Utc> = "2026-11-23T19:49:30Z".parse().unwrap();
    // Exact cutoff match must be excluded.
    assert!(
        parse_revision_rosters(SINGAPORE_REV, rev_ts)
            .unwrap()
            .is_none()
    );
    // Earlier cutoff must also exclude the revision.
    assert!(
        parse_revision_rosters(SINGAPORE_REV, rev_ts - Duration::seconds(1))
            .unwrap()
            .is_none()
    );
    // 1 second after revision timestamp succeeds.
    assert!(
        parse_revision_rosters(SINGAPORE_REV, rev_ts + Duration::seconds(1))
            .unwrap()
            .is_some()
    );
}

#[test]
fn liquipedia_parser_handles_stage_subpage_without_participants() {
    let cutoff = "2026-11-24T00:00:00Z".parse().unwrap();
    let parsed = parse_revision_rosters(SINGAPORE_STAGE1_REV, cutoff)
        .unwrap()
        .expect("stage subpage revision is valid JSON");
    assert!(parsed.at(cutoff).unwrap().teams.is_empty());
}

#[test]
fn rss_parser_extracts_dated_items_with_urls_across_real_cs2_feeds() {
    let cutoff: kallbet_ai::Instant = "2026-11-26T00:00:00Z".parse().unwrap();

    // HLTV: GMT timestamps, XML entities (&quot;), trailing whitespace trimmed.
    let hltv = parse_rss_items(HLTV_RSS, "https://www.hltv.org/rss/news", cutoff).unwrap();
    assert_eq!(hltv.len(), 10);
    let first = hltv[0].at(cutoff).unwrap();
    assert_eq!(first.source, "HLTV.org - The home of competitive Counter-Strike");
    assert_eq!(first.title, "Media: oSee joins Without a Roof");
    assert_eq!(
        first.url,
        "https://www.hltv.org/news/46055/media-osee-joins-without-a-roof"
    );
    assert_eq!(
        first.published_at,
        "2026-11-25T15:30:00Z".parse::<kallbet_ai::Instant>().unwrap()
    );
    let quote_item = hltv
        .iter()
        .map(|i| i.at(cutoff).unwrap())
        .find(|i| i.url.contains("/46049/"))
        .unwrap();
    assert!(quote_item.title.contains("\"history is another voice"));

    // Dust2.us: RFC 2822 -0500 normalized to UTC (18:14 -0500 -> 23:14Z).
    let dust2us = parse_rss_items(DUST2US_RSS, "https://www.dust2.us/rss", cutoff).unwrap();
    assert_eq!(dust2us.len(), 10);
    let d_first = dust2us[0].at(cutoff).unwrap();
    assert_eq!(
        d_first.published_at,
        "2026-11-24T23:14:00Z".parse::<kallbet_ai::Instant>().unwrap()
    );
    assert_eq!(
        d_first.url,
        "https://www.dust2.us/news/80253/have-you-ever-wanted-to-watch-a-cs2-major-in-minecraft-look-no-further"
    );

    // Dust2.dk: RFC 2822 +0100 normalized to UTC (15:45 +0100 -> 14:45Z), Danish UTF-8 preserved.
    let dust2dk = parse_rss_items(DUST2DK_RSS, "https://www.dust2.dk/rss", cutoff).unwrap();
    assert_eq!(dust2dk.len(), 10);
    let dk_first = dust2dk[0].at(cutoff).unwrap();
    assert_eq!(
        dk_first.published_at,
        "2026-11-25T14:45:00Z".parse::<kallbet_ai::Instant>().unwrap()
    );
    assert!(dk_first.title.contains("Når vi praccer"));

    // Steam CS2: CDATA links, +0000 dates, HTML-escaped descriptions cleaned to plain text.
    let steam = parse_rss_items(
        STEAM_RSS,
        "https://store.steampowered.com/feeds/news/app/730/",
        cutoff,
    )
    .unwrap();
    assert_eq!(steam.len(), 10);
    let s_first = steam[0].at(cutoff).unwrap();
    assert_eq!(s_first.title, "Counter-Strike 2 Update");
    assert_eq!(
        s_first.url,
        "https://store.steampowered.com/news/app/730/view/694278269818635488"
    );
    assert!(s_first.description.contains("Cobblestone"));
    assert!(!s_first.description.contains("<p class="));
}

#[test]
fn rss_parser_strictly_excludes_items_at_or_after_cutoff_and_rejects_html() {
    // 2026-11-25T12:43:00Z is the exact pubDate of "NIP beat Liquid to reach Stage 2 in Singapore".
    let cutoff: kallbet_ai::Instant = "2026-11-25T12:43:00Z".parse().unwrap();
    let items = parse_rss_items(HLTV_RSS, "https://www.hltv.org/rss/news", cutoff).unwrap();
    // Only the 4 items strictly before 12:43:00Z (11:18, 10:55, 09:56, 09:36) may remain.
    assert_eq!(items.len(), 4);
    for item in &items {
        assert!(item.observed_at() < cutoff);
        assert!(!item.at(cutoff).unwrap().url.contains("/46052/"));
    }

    // Cloudflare HTML challenge page is rejected as a parse error.
    let html = "<!DOCTYPE html><html><head><title>Just a moment...</title></head><body></body></html>";
    assert!(parse_rss_items(html, "https://www.hltv.org/rss/news", cutoff).is_err());
}

#[test]
fn rss_relevance_filter_matches_teams_and_aliases_with_word_boundaries() {
    let cutoff: kallbet_ai::Instant = "2026-11-26T00:00:00Z".parse().unwrap();
    let mut items = parse_rss_items(HLTV_RSS, "https://www.hltv.org/rss/news", cutoff).unwrap();
    items.extend(parse_rss_items(DUST2US_RSS, "https://www.dust2.us/rss", cutoff).unwrap());

    // Ninjas in Pyjamas (alias NIP) vs Team Liquid (core Liquid): matches NIP/Liquid articles,
    // excludes unrelated articles like TYLOO, Without a Roof, FlyQuest/Imperial.
    let matched = filter_match_news(
        &items,
        "Ninjas in Pyjamas",
        &[],
        "Team Liquid",
        &[],
        cutoff,
        Duration::days(14),
        10,
    )
    .unwrap();
    let urls: Vec<&str> = matched
        .iter()
        .map(|f| f.at(cutoff).unwrap().url.as_str())
        .collect();
    assert!(urls.contains(&"https://www.hltv.org/news/46052/nip-beat-liquid-to-reach-stage-2-in-singapore"));
    assert!(urls.contains(&"https://www.dust2.us/news/80247/liquid-go-up-2-0-in-pgl-major-singapore-stage-1-after-a-confident-day-one"));
    assert!(!urls.iter().any(|u| u.contains("tyloo-bench-moseyuh")));
    assert!(!urls.iter().any(|u| u.contains("without-a-roof")));

    // GOSH with Liquipedia-discovered alias "BC.Game Esports" matches BC.Game news items.
    let gosh_matched = filter_match_news(
        &items,
        "GOSH",
        &["BC.Game Esports".to_string()],
        "MOUZ",
        &[],
        cutoff,
        Duration::days(14),
        10,
    )
    .unwrap();
    assert!(
        gosh_matched
            .iter()
            .any(|f| f.at(cutoff).unwrap().url.contains("chinggis-warriors-condemn-bcgame"))
    );
}
