//! `ies documento DOC` pela Web (Refs #60, fatia 1;
//! contracts/cli/documento-obter-web.md): pesquisa rápida do número SEI
//! (o `action` guardado, senão o do Controle) → 302 para
//! `procedimento_trabalhar` com o `id_protocolo` do documento → tela com a
//! árvore que seleciona esse `id_documento` → árvore conferida (NUP do nó
//! `PROCESSO`, documento de mesmo id e número SEI **igual** ao pedido) →
//! GET do `Nos[i].src` assinado (`documento_visualizar`) → título e texto
//! conferidos. Nenhuma URL é montada (ADR 0003); sem retry, sem redirect.
use super::abertura::{Contexto, Lida, Rede, fora_de_palpite};
use super::fluxo::{ler_controle_colhido, pesquisar_no_controle, pesquisar_pela_guardada};
use super::{EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso};
use crate::arvore_processo::{
    ConteudoNaArvore, EnderecoDoConteudo, ErroArvore, ErroConteudoNaArvore, NaturezaDoDocumento,
    ProcessoNaArvore, ler_arvore, ler_conteudo_na_arvore, ler_tela_do_documento,
    nup_declarado_na_arvore,
};
use crate::documento::NumeroDocumento;
use crate::documento_web::{identidade_do_titulo, texto_do_corpo};
use crate::indice_local::{IndiceLocal, unidade_dos_links};
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, arquivo_privado, validar_cookie};

#[path = "documento_erro.rs"]
mod erro;
pub(crate) use erro::ErroDocumentoObter;
use erro::ErroDocumentoObter as E;
#[path = "documento_versao.rs"]
mod versao;
pub(crate) use versao::{
    CopiaDeTrabalho, PedidoDeVersoes, SemCopia, SemVersaoAtual, VersoesDoDocumento,
};

/// O número SEI pedido e a `--unidade` (conferida na tela).
pub(crate) struct PedidoDoDocumento<'a> {
    pub(crate) numero: &'a NumeroDocumento,
    pub(crate) unidade: Option<&'a str>,
}

/// O documento lido: identidade da árvore e do título, processo, unidade
/// da tela, texto visível e requisições feitas.
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
    pub(crate) unidade: String,
    pub(crate) texto: String,
    pub(crate) requisicoes: u8,
    /// Versão lida, atual e cópia de trabalho (Refs #88).
    pub(crate) versoes: VersoesDoDocumento,
}

pub(crate) fn obter_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoObtido, E> {
    obter_documento_com_versoes(sessao, pedido, PedidoDeVersoes::default(), indice, envio)
}

/// Como [`obter_documento`], com a versão N ou a cópia de trabalho
/// (contracts/cli/documento-obter-versoes.md).
pub(crate) fn obter_documento_com_versoes<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    versoes: PedidoDeVersoes,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoObtido, E> {
    let processo = |erro| E::Processo(erro);
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::Processo(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let texto = arquivo_privado(sessao.cookie_file, 8_192)
        .map_err(|f| E::Processo(ErroProcessoObter::Transporte(f)))?;
    let cookie = texto.trim().to_owned();
    validar_cookie(&cookie).map_err(|f| E::Processo(ErroProcessoObter::Transporte(f)))?;
    let numero = pedido.numero.as_str();
    let contexto = Contexto {
        instalacao,
        nup: numero,
        unidade_pedida: pedido.unidade,
        geracao: crate::sessao_login::geracao_da_sessao(&cookie),
        unidade_id: unidade_dos_links(sessao.url_do_controle),
        indice,
        cookie_file: sessao.cookie_file,
    };
    let mut rede = Rede {
        envio,
        cookie,
        requisicoes: 0,
    };
    let redirecionado = match pesquisar_pela_guardada(&contexto, sessao, &mut rede, numero) {
        Ok(Some(redirecionado)) => redirecionado,
        Ok(None) => {
            let controle = ler_controle_colhido(&contexto, sessao, &mut rede).map_err(processo)?;
            pesquisar_no_controle(&contexto, &controle, &mut rede, numero).map_err(sem_trabalho)?
        }
        Err(erro) => return Err(sem_trabalho(erro)),
    };
    let id_documento = redirecionado.id.as_str().to_owned();
    let trabalho = redirecionado.trabalho.as_str();
    let (html, mime) = match rede.get(trabalho).map_err(processo)? {
        Lida::Html { html, mime } => (html, mime),
        outra => return Err(E::Processo(fora_do_html(outra))),
    };
    let (tela, id_procedimento) =
        ler_tela_do_documento(&contexto.instalacao, &html, &id_documento, pedido.unidade)
            .map_err(|e| E::Processo(ErroProcessoObter::Tela(e)))?;
    contexto.aprender_pesquisa_rapida(trabalho, &html, &mime, &tela.unidade);
    let html = match rede.get(tela.arvore.as_str()).map_err(processo)? {
        Lida::Html { html, .. } => html,
        outra => return Err(E::Processo(fora_do_html(outra))),
    };
    let arvore_erro = |e| E::Processo(ErroProcessoObter::Arvore(e));
    let nup = nup_declarado_na_arvore(&html)
        .ok_or(arvore_erro(ErroArvore::NoDoProcessoAusenteNaArvore))?;
    let arvore = ler_arvore(&html, &nup, &id_procedimento).map_err(arvore_erro)?;
    // A árvore confirmou o processo: o par e o link dela, como em processo NUP.
    let _ = indice.registrar_processo(&contexto.instalacao, &nup, &id_procedimento);
    contexto.aprender_link(
        "procedimento_visualizar",
        &id_procedimento,
        tela.arvore.as_str(),
    );
    let Some(no) = arvore.documentos.iter().find(|d| d.id == id_documento) else {
        return Err(if arvore.pastas_nao_carregadas {
            E::DocumentoEmPastaNaoCarregada
        } else {
            E::Conteudo(ErroConteudoNaArvore::DocumentoAusenteDaArvore)
        });
    };
    if no.numero_sei.as_deref() != Some(numero) {
        return Err(E::NumeroSeiDaArvoreDiferenteDoPedido);
    }
    let repetidos = arvore
        .documentos
        .iter()
        .filter(|d| d.numero_sei.as_deref() == Some(numero));
    if repetidos.count() > 1 {
        return Err(E::NumeroSeiRepetidoNaArvore);
    }
    let endereco = match ler_conteudo_na_arvore(&contexto.instalacao, &html, &id_documento)
        .map_err(E::Conteudo)?
    {
        ConteudoNaArvore::Visualizar(endereco) => endereco,
        ConteudoNaArvore::DownloadDeAnexo => return Err(E::DocumentoExterno),
        ConteudoNaArvore::Vazio if no.natureza.cancelado == Some(true) => {
            return Err(E::DocumentoCancelado);
        }
        ConteudoNaArvore::Vazio => return Err(E::ConteudoNaoPublicadoNaArvore),
    };
    let no_conferido = versao::NoConferido {
        instalacao: &contexto.instalacao,
        arvore: &html,
        id_documento: &id_documento,
        id_procedimento: &id_procedimento,
        numero,
        conteudo: &endereco,
    };
    let (tipo, texto, versoes) = versao::ler_conteudo_e_versoes(&mut rede, &no_conferido, versoes)?;
    Ok(DocumentoObtido {
        numero: numero.to_owned(),
        id_documento,
        rotulo: no.rotulo.clone(),
        tipo,
        natureza: no.natureza.clone(),
        pasta: no.pasta.clone(),
        processo: arvore.processo,
        unidade: tela.unidade,
        texto,
        requisicoes: rede.requisicoes,
        versoes,
    })
}

/// GET do link assinado; o título tem de ser do mesmo número SEI.
fn ler_conteudo<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    endereco: &EnderecoDoConteudo,
    numero: &str,
) -> Result<(String, String), E> {
    let html = match rede.get(endereco.as_str()).map_err(E::Processo)? {
        Lida::Html { html, .. } => html,
        outra => return Err(E::Processo(fora_do_html(outra))),
    };
    let identidade = identidade_do_titulo(&html).map_err(E::Titulo)?;
    if identidade.numero != numero {
        return Err(E::TituloDeOutroDocumento);
    }
    let texto = texto_do_corpo(&html).map_err(E::Corpo)?;
    Ok((identidade.tipo, texto))
}

/// Resposta que não é HTML: 302, login ou falha de transporte.
fn fora_do_html(lida: Lida) -> ErroProcessoObter {
    match lida {
        Lida::Redirecionamento(local) => fora_de_palpite(local.as_deref()),
        Lida::Login => ErroProcessoObter::SessaoEncerrada,
        Lida::Falha(falha) => ErroProcessoObter::Transporte(falha),
        Lida::Html { .. } => {
            ErroProcessoObter::RedirecionamentoNaoReconhecido(StatusHttpRecusado::sem_mensagem(200))
        }
    }
}

/// A pesquisa rápida que não abriu tela de trabalho: o número não levou a
/// um documento (o resultado da pesquisa não é lido).
fn sem_trabalho(erro: ErroProcessoObter) -> E {
    match erro {
        ErroProcessoObter::PesquisaRapidaSemUrlDeTrabalho(status) => {
            E::PesquisaRapidaNaoAbriuODocumento(status)
        }
        outro => E::Processo(outro),
    }
}
