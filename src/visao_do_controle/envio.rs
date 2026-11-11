//! Envio injetável das requisições da troca de visão: cada chamada de
//! `post`/`get` é uma requisição real única (sem retry); os testes contam
//! e conferem a sequência.
use crate::formulario_web::{
    OperacaoDeFormulario, RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::cell::RefCell;
use std::io::Read;

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
    /// Uma tela do Controle que o fluxo acabou de receber, para a colheita
    /// do índice local (nenhuma requisição). Padrão: nada.
    fn colher_controle(&mut self, _html: &str, _mime: Option<&str>) {}
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

/// Decorador que registra uma tentativa antes de cada POST/GET injetado.
/// Falhas de transporte e respostas 302 também consomem uma tentativa;
/// a colheita local não consome nenhuma.
pub(crate) struct EnvioContado<V> {
    pub(crate) interno: V,
    pub(crate) pedidos: crate::transporte::contagem::ContagemDeRequisicoes,
}

impl<V> EnvioContado<V> {
    pub(crate) fn novo(interno: V) -> Self {
        Self {
            interno,
            pedidos: crate::transporte::contagem::ContagemDeRequisicoes::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> usize {
        self.pedidos.total()
    }
}

impl<V: EnvioDoControle> EnvioDoControle for EnvioContado<V> {
    type Corpo = V::Corpo;

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.pedidos.registrar();
        self.interno.post(submissao, cookie)
    }

    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.pedidos.registrar();
        self.interno.get(url, cookie)
    }

    fn colher_controle(&mut self, html: &str, mime: Option<&str>) {
        self.interno.colher_controle(html, mime);
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
    pub(super) cookie: &'a CookieDaSessao,
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
    ) -> Result<TelaAtual, super::ErroVisaoDoControle> {
        let codificacao = codificacao_da_tela(&tela.mime)
            .ok_or(super::ErroVisaoDoControle::CharsetDaTelaNaoSuportado)?;
        let servida = TelaServida {
            url: &tela.url,
            html: &tela.html,
            codificacao,
        };
        let submissao =
            derivar_submissao(operacao, self.instalacao, &servida, self.unidade, campos)?;
        let url = submissao.url().to_owned();
        let mime = RefCell::new(String::new());
        let envio = &mut *self.envio;
        let resposta = enviar_submissao_via(submissao, self.cookie, |s, c| {
            let resposta = envio.post(s, c)?;
            mime.replace(resposta.resposta.mime.clone());
            Ok(resposta)
        })?;
        match resposta {
            RespostaDoEnvio::Html(html) => {
                let mime = mime.into_inner();
                // Só os envios de `frmProcedimentoControlar` devolvem o Controle.
                if operacao.acao == crate::acao_do_sei::AcaoDoSei::ProcedimentoControlar {
                    self.envio.colher_controle(&html, Some(&mime));
                }
                Ok(TelaAtual { url, html, mime })
            }
            RespostaDoEnvio::Redirecionamento(_) => Err(match redirecionado {
                Redirecionado::AoTrocarVisao => {
                    super::ErroVisaoDoControle::RedirecionamentoAoTrocarVisao
                }
                Redirecionado::AoPaginar => super::ErroVisaoDoControle::RedirecionamentoAposPaginar,
            }),
        }
    }
}
