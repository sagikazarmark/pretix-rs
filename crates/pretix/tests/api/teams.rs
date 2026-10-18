use pretix::ListParams;
use pretix::resources::teams::{
    CreateTeam, CreateTeamInvite, CreateTeamToken, EventPermission, OrganizerPermission, Team,
    TeamInvite, TeamMember, TeamToken, UpdateTeam,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn team_models() {
    let team = assert_model::<Team>(&fixture("teams/team"), &[]);
    assert_eq!(
        team.limit_organizer_permissions,
        [OrganizerPermission::EventsCreate]
    );
    assert_model::<TeamMember>(&fixture("teams/team_member"), &[]);
    assert_model::<TeamInvite>(&fixture("teams/team_invite"), &[]);
    assert_model::<TeamToken>(&fixture("teams/team_token"), &[]);
    let token = assert_model::<TeamToken>(&fixture("teams/team_token_created"), &[]);
    assert_eq!(token.token.as_deref(), Some(""));
}

#[test]
fn invite_without_id() {
    let invite = assert_model::<TeamInvite>(&json!({"email": "mark@example.org"}), &[]);
    assert_eq!(invite.id, None);
}

#[tokio::test]
async fn team_crud() {
    let (server, client) = setup().await;
    let team = fixture("teams/team");
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![team.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&team))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/teams/"))
        .and(body_json(json!({
            "name": "Admin team",
            "all_events": true,
            "all_event_permissions": false,
            "limit_event_permissions": ["event.orders:read", "event:cancel"],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&team))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/teams/1/"))
        .and(body_json(json!({
            "all_organizer_permissions": false,
            "limit_organizer_permissions": ["organizer.events:create"],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&team))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/teams/1/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let teams = client.organizer("bigevents").teams();
    let page = teams
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].name, "Admin team");
    assert_eq!(teams.get(1).await.unwrap().id, 1);

    let body = CreateTeam::builder()
        .name("Admin team")
        .all_events(true)
        .all_event_permissions(false)
        .limit_event_permissions(vec![EventPermission::OrdersRead, EventPermission::Cancel])
        .build();
    teams.create(&body).await.unwrap();

    let update = UpdateTeam::builder()
        .all_organizer_permissions(false)
        .limit_organizer_permissions(vec![OrganizerPermission::EventsCreate])
        .build();
    teams.update(1, &update).await.unwrap();
    teams.delete(1).await.unwrap();
}

#[tokio::test]
async fn members() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/members/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("teams/team_member")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/members/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("teams/team_member")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/teams/1/members/1/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let members = client.organizer("bigevents").teams().members(1);
    let page = members.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].fullname.as_deref(), Some("John Doe"));
    assert_eq!(members.get(1).await.unwrap().email, "john@example.com");
    members.delete(1).await.unwrap();
}

#[tokio::test]
async fn invites() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/invites/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("teams/team_invite")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/invites/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("teams/team_invite")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/teams/1/invites/"))
        .and(body_json(json!({"email": "mark@example.org"})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"id": 1, "email": "mark@example.org"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/teams/1/invites/1/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let invites = client.organizer("bigevents").teams().invites(1);
    assert_eq!(invites.list(&ListParams::default()).await.unwrap().count, 1);
    assert_eq!(invites.get(1).await.unwrap().id, Some(1));
    let created = invites
        .create(
            &CreateTeamInvite::builder()
                .email("mark@example.org")
                .build(),
        )
        .await
        .unwrap();
    assert_eq!(created.email, "mark@example.org");
    invites.delete(1).await.unwrap();
}

#[tokio::test]
async fn tokens() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/tokens/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("teams/team_token")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/teams/1/tokens/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("teams/team_token")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/teams/1/tokens/"))
        .and(body_json(json!({"name": "New token"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("teams/team_token_created")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/teams/1/tokens/1/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"id": 1, "name": "My token", "active": false})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tokens = client.organizer("bigevents").teams().tokens(1);
    assert_eq!(
        tokens.list(&ListParams::default()).await.unwrap().results[0].name,
        "Test token"
    );
    assert!(tokens.get(1).await.unwrap().active);
    let created = tokens
        .create(&CreateTeamToken::builder().name("New token").build())
        .await
        .unwrap();
    assert_eq!(created.id, 2);
    let disabled = tokens.delete(1).await.unwrap();
    assert!(!disabled.active);
}
