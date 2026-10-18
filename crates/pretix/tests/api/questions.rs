use pretix::query::Ordering;
use pretix::resources::questions::{
    CreateQuestion, CreateQuestionOption, ListQuestionOptions, ListQuestions, Question,
    QuestionOption, QuestionOrdering, QuestionType, UpdateQuestion, UpdateQuestionOption,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn question_models() {
    let question = assert_model::<Question>(&fixture("questions/question"), &[]);
    assert_eq!(question.r#type, QuestionType::Choice);
    assert_eq!(question.options.len(), 3);
    assert_model::<QuestionOption>(&fixture("questions/option"), &[]);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/questions/",
        ))
        .and(query_param("ordering", "-id"))
        .and(query_param("identifier", "WY3TP9SL"))
        .and(query_param("ask_during_checkin", "false"))
        .and(query_param("required", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("questions/question")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListQuestions::builder()
        .ordering(Ordering::desc(QuestionOrdering::Id))
        .identifier("WY3TP9SL")
        .ask_during_checkin(false)
        .required(true)
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .questions()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].identifier, "WY3TP9SL");
}

#[tokio::test]
async fn create_get_update_delete() {
    let (server, client) = setup().await;
    let question = fixture("questions/question");
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/questions/",
        ))
        .and(body_json(json!({
            "question": {"en": "T-Shirt size"},
            "type": "C",
            "required": false,
            "items": [1, 2],
            "identifier": null,
            "options": [
                {"answer": {"en": "S"}, "identifier": null},
                {"answer": {"en": "M"}, "identifier": "M"},
            ],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&question))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/questions/1/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&question))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/questions/1/",
        ))
        .and(body_json(
            json!({"position": 2, "help_text": null, "dependency_question": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&question))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/questions/1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let questions = client
        .organizer("bigevents")
        .event("sampleconf")
        .questions();
    let body = CreateQuestion::builder()
        .question(LocalizedString::from([("en", "T-Shirt size")]))
        .r#type(QuestionType::Choice)
        .required(false)
        .items(vec![1, 2])
        .options(vec![
            CreateQuestionOption::builder()
                .answer(LocalizedString::from([("en", "S")]))
                .build(),
            CreateQuestionOption::builder()
                .answer(LocalizedString::from([("en", "M")]))
                .identifier("M")
                .build(),
        ])
        .build();
    let created = questions.create(&body).await.unwrap();
    assert_eq!(created.id, 1);
    questions.get(1).await.unwrap();

    let update = UpdateQuestion::builder()
        .position(2)
        .help_text(None)
        .dependency_question(None)
        .build();
    questions.update(1, &update).await.unwrap();
    questions.delete(1).await.unwrap();
}

#[tokio::test]
async fn options() {
    let (server, client) = setup().await;
    let option = fixture("questions/option");
    let base = "/api/v1/organizers/bigevents/events/sampleconf/questions/11/options/";
    Mock::given(method("GET"))
        .and(path(base))
        .and(query_param("active", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![option.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&option))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "identifier": "LVETRWVU",
            "position": 1,
            "answer": {"en": "S"},
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&option))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({"position": 3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&option))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let options = client
        .organizer("bigevents")
        .event("sampleconf")
        .questions()
        .options(11);
    let page = options
        .list(&ListQuestionOptions::builder().active(true).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].answer.localize("en"), Some("S"));
    options.get(1).await.unwrap();
    let body = CreateQuestionOption::builder()
        .answer(LocalizedString::from([("en", "S")]))
        .identifier("LVETRWVU")
        .position(1)
        .build();
    options.create(&body).await.unwrap();
    options
        .update(1, &UpdateQuestionOption::builder().position(3).build())
        .await
        .unwrap();
    options.delete(1).await.unwrap();
}
