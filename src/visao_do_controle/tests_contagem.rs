//! Testes sintéticos das tentativas HTTP da visão de unidade, sem rede.
//! Os totais são os envios injetados; o GET inicial do Controle fica fora
//! deste recorte e já é contado no medidor real do processo.
use super::tests::{TelaD, TelaR, cookie, instalacao, tela_atual, tela_nivel};
use super::*;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::testes_http::{atender, resposta, resposta_post};
use std::collections::VecDeque;
use std::io::Cursor;

enum Passo {
    Http(i32, String),
    Falha,
}

struct Roteiro {
    respostas: VecDeque<Passo>,
    pedidos: Vec<&'static str>,
    colheitas: usize,
}

impl Roteiro {
    fn novo(passos: Vec<Passo>) -> Self {
        Self {
            respostas: passos.into(),
            pedidos: Vec::new(),
            colheitas: 0,
        }
    }

    fn proximo(&mut self, metodo: &'static str) -> Result<(i32, Vec<u8>), FalhaTransporte> {
        match atender(&mut self.pedidos, &mut self.respostas, metodo) {
            Passo::Http(status, html) => Ok((status, html.into_bytes())),
            Passo::Falha => Err(FalhaTransporte::Rede),
        }
    }
}

impl EnvioDoControle for Roteiro {
    type Corpo = Cursor<Vec<u8>>;

    fn post(
        &mut self,
        _submissao: SubmissaoDerivada,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        let (status, bytes) = self.proximo("POST")?;
        Ok(resposta_post(
            status,
            "text/html; charset=utf-8",
            bytes,
            None,
            None,
        ))
    }

    fn get(
        &mut self,
        _url: &str,
        _cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        let (status, bytes) = self.proximo("GET")?;
        Ok(resposta(status, "text/html; charset=utf-8", bytes, None))
    }

    fn colher_controle(&mut self, _html: &HtmlServido, _mime: Option<&str>) {
        self.colheitas += 1;
    }
}

fn contado(passos: Vec<Passo>) -> EnvioContado<Roteiro> {
    EnvioContado::novo(Roteiro::novo(passos))
}

#[test]
fn get_200_do_nivel_conta_uma_tentativa_sem_contar_parsing() {
    let (_pasta, arquivo) = cookie();
    let d = TelaD::nova(2, 0, 10, vec![1, 2]);
    let mut envio = contado(vec![Passo::Http(200, tela_nivel(&[]))]);
    let resultado = ler_colunas(
        &instalacao(),
        &tela_atual(d.html()),
        Some("TESTE"),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_ok(), "{resultado:?}");
    assert_eq!(envio.total(), 1);
    assert_eq!(envio.interno.pedidos, ["GET"]);
    assert_eq!(envio.interno.colheitas, 0);
}

#[test]
fn get_302_e_dado_sem_redirect_ou_retry_e_conta_uma() {
    let (_pasta, arquivo) = cookie();
    let d = TelaD::nova(2, 0, 10, vec![1, 2]);
    let mut envio = contado(vec![Passo::Http(302, String::new())]);
    let resultado = ler_colunas(
        &instalacao(),
        &tela_atual(d.html()),
        Some("TESTE"),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_err());
    assert_eq!(envio.total(), 1);
    assert_eq!(envio.interno.pedidos, ["GET"]);
}

#[test]
fn erro_na_segunda_requisicao_preserva_duas_tentativas() {
    let (_pasta, arquivo) = cookie();
    let r = TelaR::nova((2, vec![1, 2]), (3, vec![3]));
    let d = TelaD::nova(5, 0, 2, vec![1, 2]);
    let mut envio = contado(vec![Passo::Http(200, d.html()), Passo::Falha]);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(r.html()),
        Some("TESTE"),
        Some(2),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_err());
    assert_eq!(envio.total(), 2);
    assert_eq!(envio.interno.pedidos, ["POST", "POST"]);
    assert_eq!(envio.interno.colheitas, 1);
}

#[test]
fn post_recusado_conta_a_tentativa_sem_repeticao() {
    let (_pasta, arquivo) = cookie();
    let r = TelaR::nova((2, vec![1, 2]), (3, vec![3]));
    let mut envio = contado(vec![Passo::Http(403, String::new())]);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(r.html()),
        Some("TESTE"),
        None,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_err());
    assert_eq!(envio.total(), 1);
    assert_eq!(envio.interno.pedidos, ["POST"]);
    assert_eq!(envio.interno.colheitas, 0);
}
