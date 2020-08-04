use actix_web::web::{Data, Json, Path, Query};
use actix_web::{HttpResponse, Responder, Scope};
use std::str::FromStr;
use std::sync::Arc;

use ya_client::model::market::{AgreementProposal, Demand, Proposal};
use ya_service_api_web::middleware::Identity;

use crate::market::MarketService;

use super::{
    PathAgreement, PathSubscription, PathSubscriptionProposal, ProposalId, QueryTimeout,
    QueryTimeoutMaxEvents,
};

// This file contains market REST endpoints. Responsibility of these functions
// is calling respective functions in market modules and mapping return values
// to http responses. No market logic is allowed here.

pub fn register_endpoints(scope: Scope) -> Scope {
    scope
        .service(subscribe)
        .service(get_demands)
        .service(unsubscribe)
        .service(collect)
        .service(counter_proposal)
        .service(get_proposal)
        .service(reject_proposal)
        .service(create_agreement)
        .service(get_agreement)
        .service(confirm_agreement)
        .service(wait_for_approval)
        .service(cancel_agreement)
        .service(terminate_agreement)
}

#[actix_web::post("/demands")]
async fn subscribe(
    market: Data<Arc<MarketService>>,
    body: Json<Demand>,
    id: Identity,
) -> impl Responder {
    market
        .subscribe_demand(&body.into_inner(), &id)
        .await
        .map(|id| HttpResponse::Created().json(id))
}

#[actix_web::get("/demands")]
async fn get_demands(_market: Data<Arc<MarketService>>, _id: Identity) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[actix_web::delete("/demands/{subscription_id}")]
async fn unsubscribe(
    market: Data<Arc<MarketService>>,
    path: Path<PathSubscription>,
    id: Identity,
) -> impl Responder {
    let subscription_id = path.into_inner().subscription_id;
    market
        .unsubscribe_demand(&subscription_id, &id)
        .await
        .map(|_| HttpResponse::Ok().json("Ok"))
}

#[actix_web::get("/demands/{subscription_id}/events")]
async fn collect(
    market: Data<Arc<MarketService>>,
    path: Path<PathSubscription>,
    query: Query<QueryTimeoutMaxEvents>,
    _id: Identity, // TODO: use it
) -> impl Responder {
    let subscription_id = path.into_inner().subscription_id;
    let timeout = query.timeout;
    let max_events = query.max_events;
    market
        .requestor_engine
        .query_events(&subscription_id, timeout, max_events)
        .await
        .map(|events| HttpResponse::Ok().json(events))
}

#[actix_web::post("/demands/{subscription_id}/proposals/{proposal_id}")]
async fn counter_proposal(
    market: Data<Arc<MarketService>>,
    path: Path<PathSubscriptionProposal>,
    body: Json<Proposal>,
    _id: Identity,
) -> impl Responder {
    let PathSubscriptionProposal {
        subscription_id,
        proposal_id,
    } = path.into_inner();
    let proposal = body.into_inner();
    market
        .requestor_engine
        .counter_proposal(&subscription_id, &proposal_id, &proposal)
        .await
        .map(|proposal_id| HttpResponse::Ok().json(proposal_id))
}

#[actix_web::get("/demands/{subscription_id}/proposals/{proposal_id}")]
async fn get_proposal(
    _market: Data<Arc<MarketService>>,
    _path: Path<PathSubscriptionProposal>,
    _id: Identity,
) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[actix_web::delete("/demands/{subscription_id}/proposals/{proposal_id}")]
async fn reject_proposal(
    _market: Data<Arc<MarketService>>,
    _path: Path<PathSubscriptionProposal>,
    _id: Identity,
) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[actix_web::post("/agreements")]
async fn create_agreement(
    market: Data<Arc<MarketService>>,
    body: Json<AgreementProposal>,
    id: Identity,
) -> impl Responder {
    let proposal_id = ProposalId::from_str(&body.proposal_id)?;
    let valid_to = body.valid_to;
    market
        .requestor_engine
        .create_agreement(id, &proposal_id, valid_to)
        .await
        .map(|agreement_id| HttpResponse::Ok().json(agreement_id))
}

#[actix_web::get("/agreements/{agreement_id}")]
async fn get_agreement(
    _market: Data<Arc<MarketService>>,
    _path: Path<PathAgreement>,
    _id: Identity,
) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[actix_web::post("/agreements/{agreement_id}/confirm")]
async fn confirm_agreement(
    market: Data<Arc<MarketService>>,
    path: Path<PathAgreement>,
    id: Identity,
) -> impl Responder {
    let agreement_id = path.into_inner().agreement_id;
    market
        .requestor_engine
        .confirm_agreement(id, &agreement_id)
        .await
        .map(|_| HttpResponse::NoContent().finish())
}

#[actix_web::post("/agreements/{agreement_id}/wait")]
async fn wait_for_approval(
    market: Data<Arc<MarketService>>,
    path: Path<PathAgreement>,
    query: Query<QueryTimeout>,
    _id: Identity,
) -> impl Responder {
    let agreement_id = path.into_inner().agreement_id;
    let timeout = query.timeout;
    market
        .requestor_engine
        .wait_for_approval(&agreement_id, timeout)
        .await
        .map(|status| HttpResponse::Ok().json(status.to_string()))
}

#[actix_web::delete("/agreements/{agreement_id}")]
async fn cancel_agreement(
    _market: Data<Arc<MarketService>>,
    _path: Path<PathAgreement>,
    _id: Identity,
) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[actix_web::post("/agreements/{agreement_id}/terminate")]
async fn terminate_agreement(
    _market: Data<Arc<MarketService>>,
    _path: Path<PathAgreement>,
    _id: Identity,
) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}