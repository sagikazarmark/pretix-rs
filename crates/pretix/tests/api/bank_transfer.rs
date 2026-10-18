use pretix::resources::bank_transfer::{
    BankImportJob, BankImportJobState, BankTransaction, BankTransactionState, CreateBankImportJob,
    CreateBankTransaction, ListBankImportJobs,
};
use pretix::types::Decimal;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn bank_import_job_models() {
    let job = assert_model::<BankImportJob>(&fixture("bank_transfer/bank_import_job"), &[]);
    assert_eq!(job.state, BankImportJobState::Completed);
    assert_eq!(job.transactions[0].state, BankTransactionState::NoMatch);
    let job = assert_model::<BankImportJob>(&fixture("bank_transfer/bank_import_job_created"), &[]);
    assert_eq!(job.state, BankImportJobState::Pending);
}

#[tokio::test]
async fn list_get_create() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/bankimportjobs/"))
        .and(query_param("event", "sampleconf"))
        .and(query_param("state", "completed"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("bank_transfer/bank_import_job")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/bankimportjobs/1/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("bank_transfer/bank_import_job")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/bankimportjobs/"))
        .and(body_json(json!({
            "event": "sampleconf",
            "transactions": [{
                "payer": "Foo",
                "reference": "SAMPLECONF-173AS",
                "amount": "23.00",
                "date": "2017-06-26",
            }],
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(fixture("bank_transfer/bank_import_job_created")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let jobs = client.organizer("bigevents").bank_import_jobs();
    let params = ListBankImportJobs::builder()
        .event("sampleconf")
        .state(BankImportJobState::Completed)
        .build();
    let page = jobs.list(&params).await.unwrap();
    assert_eq!(page.results[0].transactions[0].amount.to_string(), "57.00");
    assert_eq!(
        jobs.get(1).await.unwrap().event.as_deref(),
        Some("sampleconf")
    );

    let body = CreateBankImportJob::builder()
        .event("sampleconf")
        .transactions(vec![
            CreateBankTransaction::builder()
                .payer("Foo")
                .reference("SAMPLECONF-173AS")
                .amount("23.00".parse::<Decimal>().unwrap())
                .date("2017-06-26")
                .build(),
        ])
        .build();
    let job = jobs.create(&body).await.unwrap();
    assert_eq!(job.transactions, Vec::<BankTransaction>::new());
}
