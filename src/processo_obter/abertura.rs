//! Peças comuns aos passos: a rede (cookie lido uma vez, contagem de
//! requisições), a leitura de cada resposta sem seguir 302, a pesquisa
//! rápida e a abertura `procedimento_trabalhar` → árvore.
use super::login::{DestinoDoRedirecionamento, destino};
use super::{EnvioDoProcesso, ErroProcessoObter};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{
    AbertoEm, ArvoreLida, ConcluirNaBarra, EnderecoDeEscrita, EnderecoDoIncluirDocumento,
    ErroAcaoSobrestar, ErroAcompanharNaBarra, ErroAndamentoNaBarra, ErroAnexarNaBarra, ErroArvore,
    ErroAtribuirNaBarra, ErroConcluirNaBarra, ErroEnviarNaBarra, ErroLinkIncluirDocumento,
    ErroRelacionarNaBarra, MotivoAbertoEm, ReabrirNaBarra, TelaDoProcesso, ler_acompanhar_da_barra,
    ler_andamento_da_barra, ler_anexar_da_barra, ler_atribuir_da_barra, ler_concluir_da_barra,
    ler_destinos_dos_movidos, ler_enviar_da_barra, ler_link_das_propriedades,
    ler_link_dos_comentarios, ler_link_dos_marcadores, ler_reabrir_da_barra,
    ler_relacionar_da_barra, ler_retomar_da_barra, ler_sobrestar_da_barra, ler_vinculo_de_anexacao,
};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, SubmissaoDerivada,
    TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::historico_processo::{EnderecoDoHistorico, ErroLinkDoHistorico, ler_link_do_historico};
use crate::indice_local::{ChaveDoLink, IndiceLocal};
use crate::instalacao::Instalacao;
use crate::marcadores_processo::{EnderecoDosMarcadores, ErroMarcadores};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::redirecionamento::{
    OrigemDoTrabalho, ProcessoRedirecionado, conferir_trabalho,
};
use crate::processo_comentarios::{EnderecoDosComentarios, ErroLinkDosComentarios};
use crate::propriedades_processo::{EnderecoDasPropriedades, ErroPropriedades};
use crate::transporte::contagem::ContagemDeRequisicoes;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, decodificar_resposta};

/// `form#frmProtocoloPesquisaRapida` (`UnB`, GO, PEN1): POST para o `action`
/// assinado, um controle `txtPesquisaRapida`, enviado pelo `onclick` da
/// imagem (`submit()`), sem botão nem oculto. A resposta esperada é o 302.
pub(crate) const PESQUISA_RAPIDA: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProtocoloPesquisaRapida,
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

pub(crate) const CAMPO: &str = "txtPesquisaRapida";

/// O que o comando sabe antes da primeira requisição.
pub(super) struct Contexto<'a> {
    pub(super) instalacao: Instalacao,
    pub(super) nup: &'a str,
    pub(super) unidade_pedida: Option<&'a str>,
    pub(super) geracao: String,
    /// `infra_unidade_atual` da URL do Controle: a unidade dos links.
    pub(super) unidade_id: Option<String>,
    pub(super) indice: &'a IndiceLocal,
    pub(super) cookie: &'a CookieDaSessao,
    /// `documento DOC`: o número SEI procurado. Visível na árvore, as pastas
    /// fechadas não são abertas (a requisição a mais não é necessária).
    pub(super) documento_procurado: Option<&'a str>,
    pub(super) permitir_leitura_parcial: bool,
}

impl Contexto<'_> {
    /// O mesmo contexto para o processo `nup` (o de `documento DOC`, que
    /// nasce com o número SEI, quando a árvore ou o índice dão o NUP).
    pub(super) fn com_nup<'c>(&'c self, nup: &'c str) -> Contexto<'c> {
        Contexto {
            instalacao: self.instalacao.clone(),
            nup,
            unidade_pedida: self.unidade_pedida,
            geracao: self.geracao.clone(),
            unidade_id: self.unidade_id.clone(),
            indice: self.indice,
            cookie: self.cookie,
            documento_procurado: self.documento_procurado,
            permitir_leitura_parcial: self.permitir_leitura_parcial,
        }
    }

    pub(super) fn chave<'c>(&'c self, acao: AcaoDoSei, objeto: &'c str) -> Option<ChaveDoLink<'c>> {
        Some(ChaveDoLink {
            geracao_sessao: &self.geracao,
            unidade: self.unidade_id.as_deref()?,
            acao,
            alvo: objeto,
        })
    }

    /// Guarda o link sem editar; a recusa do índice não para o comando e
    /// fica nos avisos (ADR 0003).
    pub(super) fn aprender_link(&self, acao: AcaoDoSei, objeto: &str, url: &str) {
        if let Some(chave) = self.chave(acao, objeto) {
            self.indice.guardar_link(&self.instalacao, chave, url);
        }
    }

    pub(super) fn esquecer_link(&self, acao: AcaoDoSei, objeto: &str) {
        if let Some(chave) = self.chave(acao, objeto) {
            self.indice.esquecer_link(&self.instalacao, chave);
        }
    }

    /// Cada `DOCUMENTO` da árvore conferida com número SEI: o índice aprende
    /// número ↔ (processo, documento), sem requisição (Refs #60, fatia 2).
    pub(super) fn aprender_documentos(&self, arvore: &ArvoreLida) {
        if !arvore.documentos_incluidos {
            return;
        }
        let documentos: Vec<(&str, &str)> = arvore
            .documentos
            .iter()
            .filter_map(|d| Some((d.numero_sei.as_deref()?, d.id.as_str())))
            .collect();
        self.indice
            .registrar_documentos(&self.instalacao, &arvore.processo.id, &documentos);
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
            Some(unidade),
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
    /// A página lida (a de "Abrir todas as Pastas", se aberta): o `.src` de
    /// cada documento sai dela.
    pub(super) html: String,
    pub(super) historico: Result<EnderecoDoHistorico, ErroLinkDoHistorico>,
    /// Link de inclusão na barra, observado somente para documento criar.
    pub(super) incluir: Result<Option<EnderecoDoIncluirDocumento>, ErroLinkIncluirDocumento>,
    /// O link das propriedades na barra (a falha só pesa se forem pedidas).
    pub(super) propriedades: Result<Option<EnderecoDasPropriedades>, ErroPropriedades>,
    /// O link dos marcadores na barra (a falha só pesa se forem pedidos).
    pub(super) marcadores: Result<Option<EnderecoDosMarcadores>, ErroMarcadores>,
    /// O link de prazo na barra; falha só pesa em `processo prazo`.
    pub(super) prazo: Result<Option<String>, crate::arvore_processo::ErroLinkPrazo>,
    /// A ação de concluir na barra (só pesa em `processo concluir`).
    pub(super) concluir: Result<Option<ConcluirNaBarra>, ErroConcluirNaBarra>,
    /// A ação de reabrir na barra (só pesa em `processo reabrir`).
    pub(super) reabrir: Result<Option<ReabrirNaBarra>, crate::arvore_processo::ErroReabrirNaBarra>,
    pub(super) sobrestar: Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar>,
    pub(super) retomar: Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar>,
    /// O link da tela de envio na barra (só pesa em `processo enviar`).
    pub(super) enviar: Result<Option<EnderecoDeEscrita>, ErroEnviarNaBarra>,
    pub(super) anotacoes: Result<Option<String>, crate::arvore_processo::ErroLinkDasAnotacoes>,
    /// Link da lista de comentários publicado na barra, nunca montado.
    pub(super) comentarios: Result<Option<EnderecoDosComentarios>, ErroLinkDosComentarios>,
    /// Ciência: botão de ação e URL da lista, cada um só pesa quando solicitado.
    pub(super) ciencia:
        Result<Option<EnderecoDeEscrita>, crate::arvore_processo::ErroCienciaNaArvore>,
    pub(super) lista_ciencias:
        Result<Option<EnderecoDeEscrita>, crate::arvore_processo::ErroCienciaNaArvore>,
    pub(super) anexar: Result<Option<EnderecoDeEscrita>, ErroAnexarNaBarra>,
    /// O link de atribuição, só examinado pelo comando atribuir.
    pub(super) atribuir: Result<Option<EnderecoDeEscrita>, ErroAtribuirNaBarra>,
    /// Link do formulário de Atualizar Andamento, quando publicado.
    pub(super) andamento: Result<Option<EnderecoDeEscrita>, ErroAndamentoNaBarra>,
    pub(super) relacionar: Result<Option<EnderecoDeEscrita>, ErroRelacionarNaBarra>,
    /// O link do acompanhamento, usado apenas por processo acompanhar.
    pub(super) acompanhar: Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra>,
}

pub(super) fn ler_arvore_aberta(
    contexto: &Contexto<'_>,
    html: &str,
    id: &str,
) -> Result<ArvoreAberta, ErroArvore> {
    ler_arvore_da_pagina_com_modo(
        &contexto.instalacao,
        contexto.nup,
        html,
        id,
        contexto.permitir_leitura_parcial,
    )
}

/// A leitura da página da árvore, sem rede nem índice (também a do replay
/// do acervo, docs/decisoes/processo-obter/replay-do-acervo.md).
pub(super) fn ler_arvore_da_pagina(
    instalacao: &Instalacao,
    nup: &str,
    html: &str,
    id: &str,
) -> Result<ArvoreAberta, ErroArvore> {
    ler_arvore_da_pagina_com_modo(instalacao, nup, html, id, false)
}

pub(super) fn ler_arvore_da_pagina_com_modo(
    instalacao: &Instalacao,
    nup: &str,
    html: &str,
    id: &str,
    permitir_parcial: bool,
) -> Result<ArvoreAberta, ErroArvore> {
    let mut arvore = crate::arvore_processo::ler_arvore_com_modo(html, nup, id, permitir_parcial)?;
    arvore.anexado_a = ler_vinculo_de_anexacao(instalacao, html, id)?;
    if arvore.anexado_a.is_some() {
        // O `.html` da raiz é o aviso do vínculo, não a situação.
        arvore.aberto_em = AbertoEm::NaoLido(MotivoAbertoEm::RaizPublicaOVinculoDeAnexacao);
    }
    ler_destinos_dos_movidos(instalacao, html, &mut arvore)?;
    let historico = ler_link_do_historico(instalacao, html, id);
    let incluir = crate::arvore_processo::ler_link_de_incluir_documento(instalacao, html, id);
    let propriedades = ler_link_das_propriedades(instalacao, html, id);
    let marcadores = ler_link_dos_marcadores(instalacao, html, id);
    let prazo = crate::arvore_processo::ler_link_do_prazo(instalacao, html, id);
    let concluir = ler_concluir_da_barra(instalacao, html, id);
    let reabrir = ler_reabrir_da_barra(instalacao, html, id);
    let sobrestar = ler_sobrestar_da_barra(instalacao, html, id);
    let retomar = ler_retomar_da_barra(instalacao, html, id);
    let enviar = ler_enviar_da_barra(instalacao, html, id);
    let anotacoes = crate::arvore_processo::ler_link_das_anotacoes(instalacao, html, id);
    let comentarios = ler_link_dos_comentarios(instalacao, html, id);
    let ciencia = crate::arvore_processo::ciencia_da_barra(instalacao, html, id);
    let lista_ciencias = crate::arvore_processo::lista_do_processo(instalacao, html, id);
    let anexar = ler_anexar_da_barra(instalacao, html, id);
    let atribuir = ler_atribuir_da_barra(instalacao, html, id);
    let andamento = ler_andamento_da_barra(instalacao, html, id);
    let relacionar = ler_relacionar_da_barra(instalacao, html, id);
    let acompanhar = ler_acompanhar_da_barra(instalacao, html, id);
    Ok(ArvoreAberta {
        arvore,
        html: html.to_owned(),
        historico,
        incluir,
        propriedades,
        marcadores,
        prazo,
        concluir,
        reabrir,
        sobrestar,
        retomar,
        enviar,
        anotacoes,
        comentarios,
        ciencia,
        lista_ciencias,
        acompanhar,
        relacionar,
        andamento,
        atribuir,
        anexar,
    })
}

/// Cookie lido uma vez e as requisições feitas (inclusive as que falharam).
pub(super) struct Rede<'v, V> {
    pub(super) envio: &'v mut V,
    pub(super) cookie: String,
    pub(super) requisicoes: ContagemDeRequisicoes,
}

/// Uma resposta lida sem seguir 302.
pub(super) enum Lida {
    Html { html: String, mime: String },
    Redirecionamento(Option<String>),
    Login,
    Falha(FalhaTransporte),
}

impl<V: EnvioDoProcesso> Rede<'_, V> {
    pub(super) fn get(&mut self, url: &str) -> Result<Lida, ErroProcessoObter> {
        self.requisicoes.registrar();
        let resposta = self.envio.get_sem_seguir(url, &self.cookie)?;
        if resposta.resposta.status_code == 302 {
            return Ok(Lida::Redirecionamento(resposta.localizacao));
        }
        let mime = resposta.resposta.mime.clone();
        Ok(match decodificar_resposta(resposta.resposta) {
            Ok(html) => Lida::Html { html, mime },
            Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => Lida::Login,
            // Um 5xx é uma recusa do servidor, não evidência de link
            // envelhecido. Não abrir outro caminho/pedido em seguida:
            // após uma escrita, a releitura de prova deve ser única.
            Err(falha) if falha.status_http().is_some_and(|http| http.status >= 500) => {
                return Err(ErroProcessoObter::Transporte(falha));
            }
            Err(falha) => Lida::Falha(falha),
        })
    }

    /// POST da pesquisa rápida. `Ok(None)`: a resposta foi a tela de login.
    pub(super) fn pesquisar(
        &mut self,
        contexto: &Contexto<'_>,
        submissao: SubmissaoDerivada,
        valor_pesquisado: &str,
    ) -> Result<Option<ProcessoRedirecionado>, ErroProcessoObter> {
        // Só conta a tentativa se a validação do cookie permitir o POST.
        let envio = &mut *self.envio;
        let requisicoes = &mut self.requisicoes;
        let resposta = enviar_submissao_via(submissao, contexto.cookie, |s, c| {
            requisicoes.registrar();
            envio.post(s, c)
        });
        match resposta {
            Ok(RespostaDoEnvio::Redirecionamento(r)) => match destino(r.localizacao()) {
                DestinoDoRedirecionamento::Outro => {
                    let origem = if contexto.documento_procurado == Some(valor_pesquisado) {
                        OrigemDoTrabalho::PesquisaRapidaDocumento
                    } else {
                        OrigemDoTrabalho::PesquisaRapida
                    };
                    conferir_trabalho(
                        r.localizacao(),
                        r.url_do_envio(),
                        &contexto.instalacao,
                        origem,
                    )
                    .map(Some)
                    .map_err(ErroProcessoObter::Redirecionamento)
                }
                DestinoDoRedirecionamento::RecusaSemAssinatura
                | DestinoDoRedirecionamento::Login => Ok(None),
            },
            Ok(RespostaDoEnvio::Html(_)) => Err(ErroProcessoObter::PesquisaRapidaSemUrlDeTrabalho),
            Err(ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio) => Ok(None),
            Err(ErroSubmissaoFormulario::Transporte(falha)) => {
                Err(ErroProcessoObter::Transporte(falha))
            }
            Err(outro) => Err(ErroProcessoObter::Formulario(outro)),
        }
    }

    /// GET da tela do processo e GET da árvore (e o de abrir as pastas, se
    /// alguma veio fechada); registra o que leu.
    pub(super) fn abrir(
        &mut self,
        contexto: &Contexto<'_>,
        trabalho: &str,
        id: &str,
    ) -> Result<(ArvoreAberta, String), ErroProcessoObter> {
        let (html, mime) = match self.get(trabalho)? {
            Lida::Html { html, mime } => (html, mime),
            Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
            Lida::Login => {
                return Err(ErroProcessoObter::SessaoEncerrada(
                    crate::erro_publico::SessaoEncerrada,
                ));
            }
            Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
        };
        super::validacao_acesso::abrir_trabalho(self, contexto, trabalho, id, (&html, &mime))
    }

    /// A partir da tela do processo já lida e conferida: GET da árvore (e o
    /// de abrir as pastas, se alguma veio fechada); registra o que leu.
    pub(super) fn abrir_da_tela(
        &mut self,
        contexto: &Contexto<'_>,
        trabalho: &str,
        (html, mime): (&str, &str),
        tela: TelaDoProcesso,
        id: &str,
    ) -> Result<(ArvoreAberta, String), ErroProcessoObter> {
        contexto.aprender_pesquisa_rapida(trabalho, html, mime, &tela.unidade);
        let html = match self.get(tela.arvore.as_str())? {
            Lida::Html { html, .. } => html,
            Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
            Lida::Login => {
                return Err(ErroProcessoObter::SessaoEncerrada(
                    crate::erro_publico::SessaoEncerrada,
                ));
            }
            Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
        };
        let arvore = ler_arvore_aberta(contexto, &html, id)?;
        concluir(
            contexto,
            &arvore.arvore,
            Some(trabalho),
            tela.arvore.as_str(),
        );
        let arvore = super::pastas::completar(contexto, self, &html, arvore)?;
        Ok((arvore, tela.unidade))
    }
}

/// 302 fora de palpite: login é sessão encerrada; o resto, não observado.
pub(super) fn fora_de_palpite(localizacao: Option<&str>) -> ErroProcessoObter {
    match destino(localizacao) {
        DestinoDoRedirecionamento::Outro => ErroProcessoObter::RedirecionamentoNaoReconhecido,
        DestinoDoRedirecionamento::RecusaSemAssinatura | DestinoDoRedirecionamento::Login => {
            ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
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
    contexto
        .indice
        .registrar_processo(&contexto.instalacao, contexto.nup, id);
    if let Some(trabalho) = trabalho {
        contexto.aprender_link(AcaoDoSei::ProcedimentoTrabalhar, id, trabalho);
    }
    contexto.aprender_link(AcaoDoSei::ProcedimentoVisualizar, id, url_da_arvore);
}
