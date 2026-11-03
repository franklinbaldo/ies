//! Peças comuns aos passos: a rede (cookie lido uma vez, contagem de
//! requisições), a leitura de cada resposta sem seguir 302, a pesquisa
//! rápida e a abertura `procedimento_trabalhar` → árvore.
use super::login::{DestinoDoRedirecionamento, destino};
use super::{EnvioDoProcesso, ErroProcessoObter as E};
use crate::arvore_processo::{ArvoreLida, ErroArvore, ler_arvore, ler_tela_do_processo};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, SubmissaoDerivada,
    TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::historico_processo::{EnderecoDoHistorico, ErroLinkDoHistorico, ler_link_do_historico};
use crate::indice_local::{ChaveDoLink, IndiceLocal};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::redirecionamento::{
    OrigemDoTrabalho, ProcessoRedirecionado, conferir_trabalho,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, decodificar_resposta};
use std::path::Path;

/// `form#frmProtocoloPesquisaRapida` (`UnB`, GO, PEN1): POST para o `action`
/// assinado, um controle `txtPesquisaRapida`, enviado pelo `onclick` da
/// imagem (`submit()`), sem botão nem oculto. A resposta esperada é o 302.
pub(super) const PESQUISA_RAPIDA: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisa_rapida",
    formulario: "frmProtocoloPesquisaRapida",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txtPesquisaRapida"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    // Não medido; a pesquisa pode ficar lembrada na sessão.
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

pub(super) const CAMPO: &str = "txtPesquisaRapida";

/// O que o comando sabe antes da primeira requisição.
pub(super) struct Contexto<'a> {
    pub(super) instalacao: Instalacao,
    pub(super) nup: &'a str,
    pub(super) unidade_pedida: Option<&'a str>,
    pub(super) geracao: String,
    /// `infra_unidade_atual` da URL do Controle: a unidade dos links.
    pub(super) unidade_id: Option<String>,
    pub(super) indice: &'a IndiceLocal,
    pub(super) cookie_file: &'a Path,
}

impl Contexto<'_> {
    pub(super) fn chave<'c>(&'c self, acao: &'c str, objeto: &'c str) -> Option<ChaveDoLink<'c>> {
        Some(ChaveDoLink {
            geracao_sessao: &self.geracao,
            unidade: self.unidade_id.as_deref()?,
            acao,
            objeto,
        })
    }

    /// Guarda o link sem editar; recusa do índice é ignorada (ADR 0003).
    pub(super) fn aprender_link(&self, acao: &str, objeto: &str, url: &str) {
        if let Some(chave) = self.chave(acao, objeto) {
            let _ = self.indice.registrar_link(&self.instalacao, chave, url);
        }
    }

    pub(super) fn esquecer_link(&self, acao: &str, objeto: &str) {
        if let Some(chave) = self.chave(acao, objeto) {
            self.indice.esquecer_link(&self.instalacao, chave);
        }
    }

    /// O `action` da pesquisa rápida publicado nesta tela, se houver.
    pub(super) fn aprender_pesquisa_rapida(
        &self,
        url: &str,
        html: &str,
        mime: &str,
        unidade: &str,
    ) {
        let Some(codificacao) = codificacao_da_tela(mime) else {
            return;
        };
        let tela = TelaServida {
            url,
            html,
            codificacao,
        };
        if let Ok(submissao) = derivar_submissao(
            &PESQUISA_RAPIDA,
            &self.instalacao,
            &tela,
            unidade,
            &[(CAMPO, "")],
        ) {
            self.aprender_link(PESQUISA_RAPIDA.acao, "", submissao.url());
        }
    }
}

/// A árvore conferida e o link do histórico lido da mesma página (a falha
/// do link só pesa se os andamentos forem pedidos).
pub(super) struct ArvoreAberta {
    pub(super) arvore: ArvoreLida,
    pub(super) historico: Result<EnderecoDoHistorico, ErroLinkDoHistorico>,
}

pub(super) fn ler_arvore_aberta(
    contexto: &Contexto<'_>,
    html: &str,
    id: &str,
) -> Result<ArvoreAberta, ErroArvore> {
    let arvore = ler_arvore(html, contexto.nup, id)?;
    let historico = ler_link_do_historico(&contexto.instalacao, html, id);
    Ok(ArvoreAberta { arvore, historico })
}

/// Cookie lido uma vez e as requisições feitas (inclusive as que falharam).
pub(super) struct Rede<'v, V> {
    pub(super) envio: &'v mut V,
    pub(super) cookie: String,
    pub(super) requisicoes: u8,
}

/// Uma resposta lida sem seguir 302.
pub(super) enum Lida {
    Html { html: String, mime: String },
    Redirecionamento(Option<String>),
    Login,
    Falha(FalhaTransporte),
}

impl<V: EnvioDoProcesso> Rede<'_, V> {
    pub(super) fn get(&mut self, url: &str) -> Result<Lida, E> {
        self.requisicoes = self.requisicoes.saturating_add(1);
        let resposta = self
            .envio
            .get_sem_seguir(url, &self.cookie)
            .map_err(E::Transporte)?;
        if resposta.resposta.status_code == 302 {
            return Ok(Lida::Redirecionamento(resposta.localizacao));
        }
        let mime = resposta.resposta.mime.clone();
        Ok(match decodificar_resposta(resposta.resposta) {
            Ok(html) => Lida::Html { html, mime },
            Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => Lida::Login,
            Err(falha) => Lida::Falha(falha),
        })
    }

    /// POST da pesquisa rápida. `Ok(None)`: a resposta foi a tela de login.
    pub(super) fn pesquisar(
        &mut self,
        contexto: &Contexto<'_>,
        submissao: SubmissaoDerivada,
    ) -> Result<Option<ProcessoRedirecionado>, E> {
        self.requisicoes = self.requisicoes.saturating_add(1);
        let envio = &mut *self.envio;
        let resposta =
            enviar_submissao_via(submissao, contexto.cookie_file, |s, c| envio.post(s, c));
        let sem_trabalho = |status| E::PesquisaRapidaSemUrlDeTrabalho(status);
        match resposta {
            Ok(RespostaDoEnvio::Redirecionamento(r)) => match destino(r.localizacao()) {
                DestinoDoRedirecionamento::Outro => conferir_trabalho(
                    r.localizacao(),
                    r.url_do_envio(),
                    &contexto.instalacao,
                    OrigemDoTrabalho::PesquisaRapida,
                )
                .map(Some)
                .map_err(E::Redirecionamento),
                _ => Ok(None),
            },
            Ok(RespostaDoEnvio::Html(_)) => {
                Err(sem_trabalho(StatusHttpRecusado::sem_mensagem(200)))
            }
            Err(ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio) => Ok(None),
            Err(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(status)) => {
                Err(sem_trabalho(status))
            }
            Err(ErroSubmissaoFormulario::Transporte(falha)) => match falha.status_http() {
                Some(status) => Err(sem_trabalho(status.clone())),
                None => Err(E::Transporte(falha)),
            },
            Err(outro) => Err(E::Formulario(outro)),
        }
    }

    /// GET da tela do processo e GET da árvore; registra o que leu.
    pub(super) fn abrir(
        &mut self,
        contexto: &Contexto<'_>,
        trabalho: &str,
        id: &str,
    ) -> Result<(ArvoreAberta, String), E> {
        let (html, mime) = match self.get(trabalho)? {
            Lida::Html { html, mime } => (html, mime),
            Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
            Lida::Login => return Err(E::SessaoEncerrada),
            Lida::Falha(falha) => return Err(E::Transporte(falha)),
        };
        let tela = ler_tela_do_processo(&contexto.instalacao, &html, id, contexto.unidade_pedida)
            .map_err(E::Tela)?;
        contexto.aprender_pesquisa_rapida(trabalho, &html, &mime, &tela.unidade);
        let arvore = match self.get(tela.arvore.as_str())? {
            Lida::Html { html, .. } => ler_arvore_aberta(contexto, &html, id).map_err(E::Arvore)?,
            Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
            Lida::Login => return Err(E::SessaoEncerrada),
            Lida::Falha(falha) => return Err(E::Transporte(falha)),
        };
        concluir(
            contexto,
            &arvore.arvore,
            Some(trabalho),
            tela.arvore.as_str(),
        );
        Ok((arvore, tela.unidade))
    }
}

/// 302 fora de palpite: login é sessão encerrada; o resto, não observado.
pub(super) fn fora_de_palpite(localizacao: Option<&str>) -> E {
    match destino(localizacao) {
        DestinoDoRedirecionamento::Outro => {
            E::RedirecionamentoNaoReconhecido(StatusHttpRecusado::sem_mensagem(302))
        }
        _ => E::SessaoEncerrada,
    }
}

/// A árvore confirmou a identidade: o índice aprende o par e o link dela.
pub(super) fn concluir(
    contexto: &Contexto<'_>,
    arvore: &ArvoreLida,
    trabalho: Option<&str>,
    url_da_arvore: &str,
) {
    let id = &arvore.processo.id;
    let _ = contexto
        .indice
        .registrar_processo(&contexto.instalacao, contexto.nup, id);
    if let Some(trabalho) = trabalho {
        contexto.aprender_link("procedimento_trabalhar", id, trabalho);
    }
    contexto.aprender_link("procedimento_visualizar", id, url_da_arvore);
}
