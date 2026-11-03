//! Envio injetável das requisições da troca de visão: cada chamada de
//! `post`/`get` é uma requisição real única (sem retry); os testes contam
//! e conferem a sequência.
use super::ErroVisaoDoControle as E;
use crate::formulario_web::{
    OperacaoDeFormulario, RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::cell::RefCell;
use std::io::Read;
use std::path::Path;

/// HTML, URL e `Content-Type` da mesma resposta, para derivar o envio seguinte.
pub(crate) struct TelaAtual {
    pub(crate) url: String,
    pub(crate) html: String,
    pub(crate) mime: String,
}

impl std::fmt::Debug for TelaAtual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelaAtual")
            .field("url", &"<redigido>")
            .field("html", &"<redigido>")
            .field("mime", &self.mime)
            .finish()
    }
}

/// Uma requisição por chamada, sem redirect nem retry.
pub(crate) trait EnvioDoControle {
    type Corpo: Read;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
    fn get(&mut self, url: &str, cookie: &str)
    -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte>;
}

/// Transporte real: os mesmos POST e GET HTTPS das demais leituras.
pub(crate) struct EnvioHttps;

impl EnvioDoControle for EnvioHttps {
    type Corpo = crate::transporte::cliente_https::CorpoHttps;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::enviar_post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        crate::transporte::get_https(url, cookie)
    }
}

/// O que um POST da troca de visão produz em caso de 302.
#[derive(Clone, Copy)]
pub(super) enum Redirecionado {
    AoTrocarVisao,
    AoPaginar,
}

/// Contexto fixo de um fluxo: instalação, unidade e cookie.
pub(super) struct Sessao<'a, V: EnvioDoControle> {
    pub(super) instalacao: &'a Instalacao,
    pub(super) unidade: &'a str,
    pub(super) cookie_file: &'a Path,
    pub(super) envio: &'a mut V,
}

impl<V: EnvioDoControle> Sessao<'_, V> {
    /// Deriva da `tela` e envia uma vez; a resposta vira a nova tela.
    pub(super) fn enviar(
        &mut self,
        tela: &TelaAtual,
        operacao: &OperacaoDeFormulario,
        campos: &[(&str, &str)],
        redirecionado: Redirecionado,
    ) -> Result<TelaAtual, E> {
        let codificacao = codificacao_da_tela(&tela.mime).ok_or(E::CharsetDaTelaNaoSuportado)?;
        let servida = TelaServida {
            url: &tela.url,
            html: &tela.html,
            codificacao,
        };
        let submissao =
            derivar_submissao(operacao, self.instalacao, &servida, self.unidade, campos)
                .map_err(E::Submissao)?;
        let url = submissao.url().to_owned();
        let mime = RefCell::new(String::new());
        let envio = &mut *self.envio;
        let resposta = enviar_submissao_via(submissao, self.cookie_file, |s, c| {
            let resposta = envio.post(s, c)?;
            mime.replace(resposta.resposta.mime.clone());
            Ok(resposta)
        })
        .map_err(E::Submissao)?;
        match resposta {
            RespostaDoEnvio::Html(html) => Ok(TelaAtual {
                url,
                html,
                mime: mime.into_inner(),
            }),
            RespostaDoEnvio::Redirecionamento(_) => Err(match redirecionado {
                Redirecionado::AoTrocarVisao => E::RedirecionamentoAoTrocarVisao,
                Redirecionado::AoPaginar => E::RedirecionamentoAposPaginar,
            }),
        }
    }
}
