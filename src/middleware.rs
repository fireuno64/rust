// src/middleware.rs
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use actix_session::SessionExt;
use futures_util::future::{LocalBoxFuture}; // Continua importando LocalBoxFuture
use actix_web::body::{BoxBody, MessageBody, EitherBody}; // Importa BoxBody e EitherBody

// IMPORTANTE: Importar a função `ready` do módulo `futures_util::future`
use futures_util::future::ready as futures_ready; // Renomeado para evitar conflito com std::future::ready se importado

use crate::auth_error::AuthError;

// Trait para o middleware que exige Admin
pub struct AuthAdmin;

impl<S, B> Transform<S, ServiceRequest> for AuthAdmin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    // O tipo de resposta do middleware será ServiceResponse<EitherBody<BoxBody>>
    // pois todas as respostas (originais ou geradas pelo middleware) serão encapsuladas em BoxBody
    type Response = ServiceResponse<EitherBody<BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthAdminMiddleware<S>;
    // O futuro de new_transform ainda usa Ready de futures_util
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Usa a função `futures_ready` que retorna o tipo correto `futures_util::future::Ready`
        futures_ready(Ok(AuthAdminMiddleware { service }))
    }
}

pub struct AuthAdminMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthAdminMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    // O tipo de resposta do middleware será ServiceResponse<EitherBody<BoxBody>>
    type Response = ServiceResponse<EitherBody<BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();

        let is_logged_in = session.get::<String>("user").unwrap_or(None).is_some();
        if !is_logged_in {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Found()
                        .append_header(("Location", "/login"))
                        .finish()
                        .map_into_right_body(),
                ))
            });
        }

        let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
        if !is_admin {
            eprintln!("Tentativa de acesso de não-administrador a rota restrita.");
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .body(AuthError::Forbidden.to_string())
                        .map_into_right_body(),
                ))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body().map_into_right_body())
        })
    }
}

// Middleware para verificar apenas se o usuário está logado (não precisa ser admin)
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    // O tipo de resposta do middleware será ServiceResponse<EitherBody<BoxBody>>
    type Response = ServiceResponse<EitherBody<BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareInternal<S>;
    // O futuro de new_transform ainda usa Ready de futures_util
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Usa a função `futures_ready` que retorna o tipo correto `futures_util::future::Ready`
        futures_ready(Ok(AuthMiddlewareInternal { service }))
    }
}

pub struct AuthMiddlewareInternal<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareInternal<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    // O tipo de resposta do middleware será ServiceResponse<EitherBody<BoxBody>>
    type Response = ServiceResponse<EitherBody<BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();
        let is_logged_in = session.get::<String>("user").unwrap_or(None).is_some();

        if !is_logged_in {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Found()
                        .append_header(("Location", "/login"))
                        .finish()
                        .map_into_right_body(),
                ))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body().map_into_right_body())
        })
    }
}