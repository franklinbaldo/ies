//! RED/GREEN da colheita nos fluxos de visão e página: cada tela do
//! Controle que um POST devolveu chega a `colher_controle` uma vez, na
//! ordem, sem requisição a mais (a tela do GET inicial a CLI colhe).
use super::envio::EnvioDoControle;
use super::tests::{EnvioRoteirizado, TelaD, TelaR, cookie, instalacao, tela_atual};
use super::*;
use crate::dominio::Categoria;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use crate::transporte::sessao::CookieDaSessao;
use std::io::Cursor;

/// O envio roteirizado que também anota cada tela entregue à colheita.
struct Vendo {
    envio: EnvioRoteirizado,
    vistas: Vec<String>,
}

impl EnvioDoControle for Vendo {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.envio.post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.envio.get(url, cookie)
    }
    fn colher_controle(&mut self, html: &HtmlServido, mime: Option<&str>) {
        assert_eq!(mime, Some("text/html; charset=iso-8859-1"));
        self.vistas.push(html.as_str().to_owned());
    }
}

fn vendo(respostas: Vec<(i32, String)>) -> Vendo {
    Vendo {
        envio: EnvioRoteirizado::novo(respostas),
        vistas: Vec::new(),
    }
}

#[test]
fn listar_da_r_com_pagina_n_colhe_as_duas_telas_d_recebidas() {
    let r = TelaR::nova((2, vec![1, 2]), (3, vec![3]));
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (_pasta, arquivo) = cookie();
    let mut envio = vendo(vec![(200, d1.html()), (200, d2.html())]);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(r.html()),
        Some("TESTE"),
        Some(2),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_ok());
    assert_eq!(envio.envio.registro.len(), 2);
    assert_eq!(envio.vistas, [d1.html(), d2.html()]);
}

#[test]
fn listar_ja_na_d_sem_pagina_nao_colhe_nada_aqui() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (_pasta, arquivo) = cookie();
    let mut envio = vendo(vec![]);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(d.html()),
        Some("TESTE"),
        None,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(resultado.is_ok());
    assert_eq!(envio.vistas.len(), 0);
}

#[test]
fn recebidos_da_d_com_pagina_n_colhe_a_r_e_a_pagina_n() {
    let d = TelaD::nova(4, 0, 10, vec![1, 2, 3, 9]);
    let r = TelaR::nova((3, vec![1, 2]), (1, vec![3]));
    let mut segunda = r.clone();
    segunda.pagina_r = "1";
    segunda.recebidos = (3, 3, vec![4]);
    let (_pasta, arquivo) = cookie();
    let mut envio = vendo(vec![(200, r.html()), (200, segunda.html())]);
    let resultado = obter_na_visao_resumida(
        &instalacao(),
        &tela_atual(d.html()),
        Some("TESTE"),
        Categoria::Recebidos,
        Some(2),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert_eq!(envio.envio.registro.len(), 2);
    assert_eq!(resultado.unwrap(), HtmlServido::de_teste(&segunda.html()));
    assert_eq!(envio.vistas, [r.html(), segunda.html()]);
}
