//! `ies documento DOC` pela Web (Refs #60; contracts/cli/documento-obter-web.md).
//! Caminho mais barato: com `--processo NUP`, a abertura de `processo NUP`
//! e o documento na árvore; senão o índice (número SEI → processo e
//! documento: link da árvore, link de trabalho, entrada por id com
//! documento) e, sem ele ou com palpite desmentido, a pesquisa rápida do
//! número (o `action` guardado, senão o do Controle) → 302 com o
//! `id_protocolo` do documento → tela que seleciona esse `id_documento` →
//! árvore conferida (NUP do nó `PROCESSO`, documento de mesmo id e número
//! SEI **igual** ao pedido; pastas abertas se ele não estiver à vista) →
//! GET do `Nos[i].src` assinado → título e texto conferidos. Só a URL sem
//! assinatura da lista branca é montada (ADR 0003); sem retry nem redirect.
//! `--versao N` e a cópia de trabalho (#88) partem da árvore que o caminho
//! abriu (contracts/cli/documento-obter-versoes.md).
use super::abertura::{Contexto, Lida, Rede, concluir, fora_de_palpite, ler_arvore_aberta};
use super::fluxo::{ler_controle_colhido, pesquisar_no_controle, pesquisar_pela_guardada};
use super::{EnvioDoProcesso, ErroProcessoObter, ResolvidoPor, SessaoDoProcesso};
use crate::arvore_processo::{
    ConteudoNaArvore, DocumentoNaArvore, ErroArvore, NaturezaDoDocumento, ProcessoNaArvore,
    ler_conteudo_na_arvore, ler_tela_do_documento, nup_declarado_na_arvore,
};
use crate::documento::NumeroDocumento;
use crate::indice_local::{IndiceLocal, unidade_dos_links};
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::propriedades_documento::PropriedadesDocumento;
use crate::transporte::FalhaTransporte;

#[path = "documento_erro.rs"]
mod erro;
pub(crate) use erro::ErroDocumentoObter;

#[path = "documento_anexo.rs"]
mod anexo;
#[path = "documento_cadastro.rs"]
mod cadastro;
pub(crate) use anexo::PedidoDoAnexo;

#[path = "documento_versao.rs"]
mod versao;
#[cfg(test)]
pub(crate) use versao::{FotoDaVersao, SemVersaoAtual};
pub(crate) use versao::{PedidoDeVersoes, VersoesDoDocumento};

/// O que se lê do nó além do texto: o anexo do externo e as versões.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LeituraDoNo<'a> {
    pub(super) anexo: PedidoDoAnexo<'a>,
    pub(super) versoes: PedidoDeVersoes,
    pub(super) propriedades: bool,
}

/// O número SEI pedido, a `--unidade` (conferida na tela), o
/// `--processo NUP` e o limite e `--saida` do documento externo.
pub(crate) struct PedidoDoDocumento<'a> {
    pub(crate) numero: &'a NumeroDocumento,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) processo: Option<&'a NumeroProcesso>,
    /// Limite e `--saida` do documento externo (fatia 3).
    pub(crate) anexo: PedidoDoAnexo<'a>,
}

/// O documento lido: identidade da árvore e do título, processo, unidade
/// da tela (`None` pelo link da árvore, que não a publica), passo que o
/// abriu, texto visível e requisições feitas (inclusive as dos palpites).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DocumentoObtido {
    pub(crate) numero: String,
    pub(crate) id_documento: String,
    /// a5 do nó (o rótulo da árvore).
    pub(crate) rotulo: String,
    /// O tipo do título `SEI/… - DOC - tipo`.
    pub(crate) tipo: String,
    pub(crate) natureza: NaturezaDoDocumento,
    pub(crate) pasta: Option<String>,
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) unidade: Option<String>,
    pub(crate) resolvido_por: ResolvidoPor,
    pub(crate) texto: String,
    /// Documento externo (`documento_download_anexo`, fatia 3): o arquivo
    /// lido; `tipo` fica vazio e `texto`, o dos tipos de texto.
    pub(crate) anexo: Option<crate::anexo_documento::AnexoLido>,
    pub(crate) requisicoes: u8,
    /// Versão lida, atual e cópia de trabalho (Refs #88).
    pub(crate) versoes: VersoesDoDocumento,
    pub(crate) propriedades: PropriedadesDocumento,
}

#[cfg(test)]
pub(crate) fn obter_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoObtido, ErroDocumentoObter> {
    obter_documento_com_versoes(sessao, pedido, PedidoDeVersoes::default(), indice, envio)
}

/// Como [`obter_documento`], com a versão N ou a cópia de trabalho
/// (contracts/cli/documento-obter-versoes.md), por qualquer caminho.
#[cfg(test)]
pub(crate) fn obter_documento_com_versoes<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    versoes: PedidoDeVersoes,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoObtido, ErroDocumentoObter> {
    obter_documento_com_propriedades(sessao, pedido, versoes, indice, envio, false)
}

/// Leitura Web com GET adicional do cadastro observado, quando solicitado.
pub(crate) fn obter_documento_com_propriedades<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    versoes: PedidoDeVersoes,
    indice: &IndiceLocal,
    envio: &mut V,
    propriedades: bool,
) -> Result<DocumentoObtido, ErroDocumentoObter> {
    let leitura = LeituraDoNo {
        anexo: pedido.anexo,
        versoes,
        propriedades,
    };
    abrir(sessao, pedido, leitura, indice, envio)
}

/// O documento aberto até a árvore conferida, sem o GET do conteúdo: o
/// começo de uma escrita no documento (`documento assinar`, Refs #174).
pub(crate) fn abrir_documento_na_arvore<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoNaArvoreAberto, ErroDocumentoObter> {
    abrir(sessao, pedido, SoArvore, indice, envio)
}

/// O documento conferido na árvore e o HTML dela (com URLs assinadas:
/// `Debug` redigido).
#[derive(PartialEq, Eq)]
pub(crate) struct DocumentoNaArvoreAberto {
    pub(crate) numero: String,
    pub(crate) id_documento: String,
    pub(crate) rotulo: String,
    pub(crate) natureza: NaturezaDoDocumento,
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) unidade: Option<String>,
    pub(crate) resolvido_por: ResolvidoPor,
    pub(crate) html: crate::tela_html::HtmlServido,
    pub(crate) requisicoes: u8,
}

impl std::fmt::Debug for DocumentoNaArvoreAberto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentoNaArvoreAberto")
            .field("numero", &self.numero)
            .field("id_documento", &self.id_documento)
            .field("processo", &self.processo)
            .field("html", &"<redigido>")
            .finish_non_exhaustive()
    }
}

/// O fim da leitura depois da árvore conferida, comum aos três caminhos.
pub(super) trait FimDaLeitura: Copy {
    type Lido;
    fn concluir<V: EnvioDoProcesso>(
        self,
        instalacao: &Instalacao,
        rede: &mut Rede<'_, V>,
        lido: NoLido<'_>,
        numero: &str,
    ) -> Result<Self::Lido, ErroDocumentoObter>;
}

impl FimDaLeitura for LeituraDoNo<'_> {
    type Lido = DocumentoObtido;
    fn concluir<V: EnvioDoProcesso>(
        self,
        instalacao: &Instalacao,
        rede: &mut Rede<'_, V>,
        lido: NoLido<'_>,
        numero: &str,
    ) -> Result<DocumentoObtido, ErroDocumentoObter> {
        ler_do_no(instalacao, rede, lido, self, numero)
    }
}

/// Só a árvore: nenhum GET do conteúdo.
#[derive(Clone, Copy)]
pub(super) struct SoArvore;

impl FimDaLeitura for SoArvore {
    type Lido = DocumentoNaArvoreAberto;
    fn concluir<V: EnvioDoProcesso>(
        self,
        _: &Instalacao,
        rede: &mut Rede<'_, V>,
        lido: NoLido<'_>,
        numero: &str,
    ) -> Result<DocumentoNaArvoreAberto, ErroDocumentoObter> {
        Ok(DocumentoNaArvoreAberto {
            numero: numero.to_owned(),
            id_documento: lido.no.id.clone(),
            rotulo: lido.no.rotulo.clone(),
            natureza: lido.no.natureza.clone(),
            processo: lido.processo.clone(),
            unidade: lido.unidade,
            resolvido_por: lido.resolvido_por,
            html: lido.html.to_owned(),
            requisicoes: rede.requisicoes.total_u8(),
        })
    }
}

/// Os três caminhos (processo informado, índice, pesquisa) até o nó.
fn abrir<V: EnvioDoProcesso, F: FimDaLeitura>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    leitura: F,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<F::Lido, ErroDocumentoObter> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroDocumentoObter::Processo(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let cookie = sessao
        .cookie
        .valor()
        .map_err(|f| ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(f)))?
        .to_owned();
    let numero = pedido.numero.as_str();
    let contexto = Contexto {
        instalacao,
        nup: numero,
        unidade_pedida: pedido.unidade,
        geracao: crate::sessao_login::geracao_da_sessao(&cookie),
        unidade_id: unidade_dos_links(sessao.url_do_controle),
        indice,
        cookie: sessao.cookie,
        documento_procurado: Some(numero),
        permitir_leitura_parcial: false,
    };
    let mut rede = Rede {
        envio,
        cookie,
        requisicoes: crate::transporte::contagem::ContagemDeRequisicoes::default(),
    };
    if let Some(nup) = pedido.processo {
        let contexto = contexto.com_nup(nup.as_str());
        return super::documento_indice::pelo_processo(
            &contexto, sessao, &mut rede, numero, leitura,
        );
    }
    if let Some(obtido) =
        super::documento_indice::pelo_indice(&contexto, &mut rede, numero, leitura)?
    {
        return Ok(obtido);
    }
    pela_pesquisa(&contexto, sessao, &mut rede, numero, leitura)
}

/// Fatia 1: pesquisa rápida, tela, árvore (e pastas, se o documento não
/// está à vista) e conteúdo.
fn pela_pesquisa<V: EnvioDoProcesso, F: FimDaLeitura>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
    numero: &str,
    leitura: F,
) -> Result<F::Lido, ErroDocumentoObter> {
    let processo = |erro| ErroDocumentoObter::Processo(erro);
    let redirecionado = match pesquisar_pela_guardada(contexto, sessao, rede, numero) {
        Ok(Some(redirecionado)) => redirecionado,
        Ok(None) => {
            let controle = ler_controle_colhido(contexto, sessao, rede).map_err(processo)?;
            pesquisar_no_controle(contexto, &controle, rede, numero).map_err(sem_trabalho)?
        }
        Err(erro) => return Err(sem_trabalho(erro)),
    };
    let id_documento = redirecionado.id.as_str().to_owned();
    let trabalho = redirecionado.trabalho.as_str();
    let (html, mime) = match rede.get(trabalho).map_err(processo)? {
        Lida::Html { html, mime } => (html, mime),
        outra @ (Lida::Redirecionamento(_) | Lida::Login | Lida::Falha(_)) => {
            return Err(ErroDocumentoObter::Processo(fora_do_html(outra)));
        }
    };
    let (tela, id_procedimento) = ler_tela_do_documento(
        &contexto.instalacao,
        &html,
        &id_documento,
        contexto.unidade_pedida,
    )
    .map_err(|e| ErroDocumentoObter::Processo(ErroProcessoObter::Tela(e)))?;
    contexto.aprender_pesquisa_rapida(trabalho, &html, &mime, &tela.unidade);
    let html = match rede.get(tela.arvore.as_str()).map_err(processo)? {
        Lida::Html { html, .. } => html,
        outra @ (Lida::Redirecionamento(_) | Lida::Login | Lida::Falha(_)) => {
            return Err(ErroDocumentoObter::Processo(fora_do_html(outra)));
        }
    };
    let arvore_erro = |e| ErroDocumentoObter::Processo(ErroProcessoObter::Arvore(e));
    let nup = nup_declarado_na_arvore(&html)
        .ok_or(arvore_erro(ErroArvore::NoDoProcessoAusenteNaArvore))?;
    let contexto = contexto.com_nup(&nup);
    let aberta = ler_arvore_aberta(&contexto, &html, &id_procedimento).map_err(arvore_erro)?;
    // A árvore confirmou o processo: o par e o link dela, como em processo NUP.
    concluir(&contexto, &aberta.arvore, None, tela.arvore.as_str());
    let aberta = super::pastas::completar(&contexto, rede, &html, aberta).map_err(processo)?;
    contexto.aprender_documentos(&aberta.arvore);
    let Some(no) = aberta
        .arvore
        .documentos
        .iter()
        .find(|d| d.id == id_documento)
    else {
        return Err(ErroDocumentoObter::DocumentoAusenteDaArvore);
    };
    if no.numero_sei.as_deref() != Some(numero) {
        return Err(ErroDocumentoObter::NumeroSeiDaArvoreDiferenteDoPedido);
    }
    localizar(&aberta.arvore.documentos, numero)?;
    let lido = NoLido {
        html: &aberta.html,
        no,
        processo: &aberta.arvore.processo,
        unidade: Some(tela.unidade),
        resolvido_por: ResolvidoPor::PesquisaRapida,
    };
    leitura.concluir(&contexto.instalacao, rede, lido, numero)
}

/// O único `DOCUMENTO` da árvore com o número SEI (a15) pedido.
pub(super) fn localizar<'a>(
    documentos: &'a [DocumentoNaArvore],
    numero: &str,
) -> Result<Option<&'a DocumentoNaArvore>, ErroDocumentoObter> {
    let mut com_numero = documentos
        .iter()
        .filter(|d| d.numero_sei.as_deref() == Some(numero));
    let primeiro = com_numero.next();
    if com_numero.next().is_some() {
        return Err(ErroDocumentoObter::NumeroSeiRepetidoNaArvore);
    }
    Ok(primeiro)
}

/// O nó já conferido na árvore lida e o que a abertura soube.
pub(super) struct NoLido<'a> {
    pub(super) html: &'a crate::tela_html::HtmlServido,
    pub(super) no: &'a DocumentoNaArvore,
    pub(super) processo: &'a ProcessoNaArvore,
    pub(super) unidade: Option<String>,
    pub(super) resolvido_por: ResolvidoPor,
}

/// O `.src` do nó e o GET do conteúdo (a versão N, se pedida, no lugar
/// do atual); a barra do nó vem da árvore lida.
pub(super) fn ler_do_no<V: EnvioDoProcesso>(
    instalacao: &Instalacao,
    rede: &mut Rede<'_, V>,
    lido: NoLido<'_>,
    pedido: LeituraDoNo<'_>,
    numero: &str,
) -> Result<DocumentoObtido, ErroDocumentoObter> {
    let no = lido.no;
    let conferido = versao::NoConferido {
        instalacao,
        numero,
        html: lido.html,
        id_documento: &no.id,
        id_procedimento: &lido.processo.id,
    };
    let mut cadastro_antecipado = None;
    let (tipo, texto, anexo, versoes) = match ler_conteudo_na_arvore(instalacao, lido.html, &no.id)?
    {
        ConteudoNaArvore::Visualizar(e) => {
            let (tipo, texto, versoes) = versao::ler_versoes(rede, &conferido, &e, pedido.versoes)?;
            (tipo, texto, None, versoes)
        }
        ConteudoNaArvore::DownloadDeAnexo(e) => {
            let versoes = versao::do_externo(&conferido, pedido.versoes)?;
            let (tipo, texto, anexo) = anexo::ler(rede, &e, numero, pedido.anexo)?;
            (tipo, texto, anexo, versoes)
        }
        ConteudoNaArvore::Vazio if no.natureza.cancelamento.cancelado() => {
            return Err(ErroDocumentoObter::DocumentoCancelado);
        }
        ConteudoNaArvore::Vazio => {
            let cadastro = cadastro::ler(
                instalacao,
                rede,
                lido.html,
                &no.id,
                &lido.processo.id,
                pedido.propriedades,
                true,
            )?;
            let Some(link) = cadastro.anexo.as_ref() else {
                return Err(ErroDocumentoObter::ConteudoNaoPublicadoNaArvore {
                    requisicoes: rede.requisicoes.total_u8(),
                });
            };
            let versoes = versao::do_externo(&conferido, pedido.versoes)?;
            let (tipo, texto, anexo) = anexo::ler(rede, link, numero, pedido.anexo)?;
            cadastro_antecipado = Some(cadastro);
            (tipo, texto, anexo, versoes)
        }
    };
    let propriedades = match cadastro_antecipado {
        Some(cadastro) => cadastro.propriedades,
        None => {
            cadastro::ler(
                instalacao,
                rede,
                lido.html,
                &no.id,
                &lido.processo.id,
                pedido.propriedades,
                false,
            )?
            .propriedades
        }
    };
    Ok(DocumentoObtido {
        numero: numero.to_owned(),
        id_documento: no.id.clone(),
        rotulo: no.rotulo.clone(),
        tipo,
        natureza: no.natureza.clone(),
        pasta: no.pasta.clone(),
        processo: lido.processo.clone(),
        unidade: lido.unidade,
        resolvido_por: lido.resolvido_por,
        texto,
        anexo,
        requisicoes: rede.requisicoes.total_u8(),
        versoes,
        propriedades,
    })
}

/// Resposta que não é HTML: 302, login ou falha de transporte.
fn fora_do_html(lida: Lida) -> ErroProcessoObter {
    match lida {
        Lida::Redirecionamento(local) => fora_de_palpite(local.as_deref()),
        Lida::Login => ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada),
        Lida::Falha(falha) => ErroProcessoObter::Transporte(falha),
        Lida::Html { .. } => ErroProcessoObter::RedirecionamentoNaoReconhecido,
    }
}

/// A pesquisa rápida que não abriu tela de trabalho: o número não levou a
/// um documento (o resultado da pesquisa não é lido).
fn sem_trabalho(erro: ErroProcessoObter) -> ErroDocumentoObter {
    match erro {
        ErroProcessoObter::PesquisaRapidaSemUrlDeTrabalho => {
            ErroDocumentoObter::PesquisaRapidaNaoAbriuODocumento
        }
        outro @ (ErroProcessoObter::Transporte(_)
        | ErroProcessoObter::Controle(_)
        | ErroProcessoObter::EntradaPorIdRecusada
        | ErroProcessoObter::SessaoEncerrada(_)
        | ErroProcessoObter::RespostaDaEntradaPorIdNaoReconhecida
        | ErroProcessoObter::RedirecionamentoNaoReconhecido
        | ErroProcessoObter::Redirecionamento(_)
        | ErroProcessoObter::LinkDoProcessoNoControleNaoReconhecido(_)
        | ErroProcessoObter::CharsetDoControleNaoSuportado
        | ErroProcessoObter::Formulario(_)
        | ErroProcessoObter::ValidacaoDeAcesso(_)
        | ErroProcessoObter::Tela(_)
        | ErroProcessoObter::Arvore(_)
        | ErroProcessoObter::LinkDoHistorico(_)
        | ErroProcessoObter::Andamentos(_)
        | ErroProcessoObter::FormularioDoHistorico(_)
        | ErroProcessoObter::Propriedades(_)
        | ErroProcessoObter::Marcadores(_)
        | ErroProcessoObter::PropriedadesNaoPublicadasNaBarra
        | ErroProcessoObter::MarcadoresNaoPublicadosNaBarra
        | ErroProcessoObter::TabelaDeMarcadoresNaoPublicadaNaTela
        | ErroProcessoObter::LinhaDoControleIlegivel
        | ErroProcessoObter::LinksDoProcessoRepetidosNoControle
        | ErroProcessoObter::ProcessoRepetidoNoControle) => ErroDocumentoObter::Processo(outro),
    }
}
