//! Versão N e foto `DOC.vN.html` de `ies documento DOC` (Refs #88, #207;
//! contracts/cli/documento-obter-versoes.md), depois da árvore conferida:
//! a barra do `DOCUMENTO` publica a lista de versões (link assinado) e a
//! edição; a lista (`documento_versao_listar`) publica, por linha, o
//! `documento_visualizar` assinado da versão. Nenhum link montado, nenhum
//! editor. `versao_atual` só do rodapé servido ou da lista lida (decisão
//! de custo: nenhum GET só para ela).
use super::super::EnvioDoProcesso;
use super::super::abertura::{Lida, Rede};
use super::{ErroDocumentoObter, fora_do_html};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::arvore_processo::{BarraDoDocumento, EnderecoDoConteudo, ler_barra_do_documento};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::{
    ErroVersao, HistoricoDocumento, VersaoDocumento, interpretar_historico,
};
use crate::documento_web::{identidade_do_titulo, texto_do_corpo};
use crate::instalacao::Instalacao;
use crate::rodape_do_documento::versao_do_rodape;

/// O que a leitura pede além do conteúdo atual.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PedidoDeVersoes {
    /// `--versao N`: lê a versão N em vez do conteúdo atual.
    pub(crate) versao: Option<VersaoDocumento>,
    /// `--saida DIR` (#207): a foto `DOC.vN.html` da versão lida, que
    /// exige saber N (rodapé ou lista).
    pub(crate) foto: bool,
    /// Preservar HTML no cache, sem GET adicional para versão desconhecida.
    pub(crate) cache: bool,
}

/// Por que `versao_atual` não foi publicada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemVersaoAtual {
    /// O conteúdo lido não serve o rodapé `versão N` e a lista não foi pedida.
    NaoServidaNoConteudo,
}

impl SemVersaoAtual {
    pub(crate) fn motivo(self) -> &'static str {
        match self {
            Self::NaoServidaNoConteudo => "nao_servida_no_conteudo",
        }
    }
}

/// O HTML que o SEI serviu da versão N, que a CLI grava como
/// `DOC.vN.html` (#207).
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FotoDaVersao {
    pub(crate) versao: VersaoDocumento,
    pub(crate) html: String,
}

impl std::fmt::Debug for FotoDaVersao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FotoDaVersao(v{}, <html>)", self.versao.numero())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VersoesDoDocumento {
    /// A versão do texto publicado: N de `--versao N` ou a do rodapé.
    pub(crate) lida: Option<VersaoDocumento>,
    pub(crate) atual: Result<VersaoDocumento, SemVersaoAtual>,
    /// Só com `--saida` e num documento interno.
    pub(crate) foto: Option<FotoDaVersao>,
    /// HTML lido para o cache, mesmo quando a versão é desconhecida.
    pub(crate) cache_html: Option<String>,
    pub(crate) crc_rodape: Option<String>,
}

/// O documento já conferido na árvore lida pelo caminho que a abriu
/// (pesquisa rápida, índice ou `--processo`): a barra vem desse HTML.
pub(super) struct NoConferido<'a> {
    pub(super) instalacao: &'a Instalacao,
    /// O número SEI pedido.
    pub(super) numero: &'a str,
    /// O HTML da árvore com o nó.
    pub(super) html: &'a str,
    pub(super) id_documento: &'a str,
    /// O `id_procedimento` do nó `PROCESSO` da árvore conferida.
    pub(super) id_procedimento: &'a str,
}

impl NoConferido<'_> {
    fn barra(&self) -> Result<BarraDoDocumento, ErroDocumentoObter> {
        ler_barra_do_documento(
            self.instalacao,
            self.html,
            self.id_documento,
            self.id_procedimento,
        )
        .map_err(ErroDocumentoObter::BarraDoDocumento)
    }
}

/// Documento externo (`documento_download_anexo`), antes do GET do anexo:
/// o arquivo não serve rodapé e não há HTML para fotografar. Com `--versao N`,
/// a barra (já no HTML, 0 GET) não publica a lista, observado no acervo:
/// recusa tipada; uma barra de externo com a lista não foi observada.
pub(super) fn do_externo(
    no: &NoConferido<'_>,
    pedido: PedidoDeVersoes,
) -> Result<VersoesDoDocumento, ErroDocumentoObter> {
    if pedido.versao.is_some() {
        return Err(match no.barra()?.versoes {
            None => ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore,
            Some(_) => ErroDocumentoObter::BarraDoDocumentoForaDaFormaObservada,
        });
    }
    Ok(VersoesDoDocumento {
        lida: None,
        atual: Err(SemVersaoAtual::NaoServidaNoConteudo),
        foto: None,
        cache_html: None,
        crc_rodape: None,
    })
}

/// Tipo, texto e versões do documento, com o mínimo de GETs do pedido.
pub(super) fn ler_versoes<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    conteudo: &EnderecoDoConteudo,
    pedido: PedidoDeVersoes,
) -> Result<(String, String, VersoesDoDocumento), ErroDocumentoObter> {
    match pedido.versao {
        None => ler_atual(rede, no, conteudo, pedido.foto, pedido.cache),
        Some(n) => ler_versao(rede, no, n, pedido.foto, pedido.cache),
    }
}

/// O conteúdo atual (`Nos[i].src`); a versão vem do rodapé servido. A
/// lista só é pedida para a foto, quando o rodapé não declara a versão;
/// sem rodapé e sem lista publicada, a foto é recusa tipada.
fn ler_atual<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    conteudo: &EnderecoDoConteudo,
    foto: bool,
    cache: bool,
) -> Result<(String, String, VersoesDoDocumento), ErroDocumentoObter> {
    let (tipo, texto, html) = ler_pagina(rede, conteudo.as_str(), no.numero)?;
    let servida = versao_do_rodape(&html);
    // Um link assinado no corpo impede persistencia em cache, nao a
    // leitura da versao. A foto explicita continua recusada em fotografar.
    let cache_html = (cache && !html.contains("infra_hash")).then(|| html.clone());
    let crc_rodape = crate::rodape_do_documento::crc_do_rodape(&html);
    let mut versoes = VersoesDoDocumento {
        lida: servida,
        atual: servida.ok_or(SemVersaoAtual::NaoServidaNoConteudo),
        foto: None,
        cache_html,
        crc_rodape,
    };
    if !foto {
        return Ok((tipo, texto, versoes));
    }
    let versao = if let Some(n) = servida {
        n
    } else {
        let lista = no
            .barra()?
            .versoes
            .ok_or(ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore)?;
        let atual = ler_lista(rede, no, lista.as_str())?
            .atual()
            .ok_or(ErroDocumentoObter::Versoes(ErroVersao::ListaIncompleta))?;
        versoes.atual = Ok(atual);
        atual
    };
    versoes.foto = Some(fotografar(versao, html)?);
    Ok((tipo, texto, versoes))
}

/// `--versao N`: a lista publicada na barra e o `documento_visualizar` da
/// linha N; o conteúdo atual não é pedido. Com `--saida`, a foto é a da
/// versão N, histórica ou atual.
fn ler_versao<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    n: VersaoDocumento,
    foto: bool,
    cache: bool,
) -> Result<(String, String, VersoesDoDocumento), ErroDocumentoObter> {
    let barra = no.barra()?;
    let lista = barra
        .versoes
        .ok_or(ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore)?;
    let historico = ler_lista(rede, no, lista.as_str())?;
    let atual = historico
        .atual()
        .ok_or(ErroDocumentoObter::Versoes(ErroVersao::ListaIncompleta))?;
    let entrada = historico
        .selecionar(n)
        .map_err(ErroDocumentoObter::Versoes)?;
    let url = conferir_endereco_relativo(
        no.instalacao,
        entrada.link_privado(),
        AcaoDoSei::DocumentoVisualizar,
        no.id_procedimento,
    )
    .map_err(|_| ErroDocumentoObter::Versoes(ErroVersao::LinkInseguro))?;
    let (tipo, texto, html) = ler_pagina(rede, &url, no.numero)?;
    // Um link assinado no corpo impede persistencia em cache, nao a
    // leitura da versao. A foto explicita continua recusada em fotografar.
    let cache_html = (cache && !html.contains("infra_hash")).then(|| html.clone());
    let crc_rodape = crate::rodape_do_documento::crc_do_rodape(&html);
    if versao_do_rodape(&html).is_some_and(|servida| servida != n) {
        return Err(ErroDocumentoObter::VersaoServidaDiferenteDaPedida);
    }
    let foto = if foto {
        Some(fotografar(n, html)?)
    } else {
        None
    };
    let versoes = VersoesDoDocumento {
        crc_rodape,
        lida: Some(n),
        atual: Ok(atual),
        foto,
        cache_html,
    };
    Ok((tipo, texto, versoes))
}

/// A foto da versão N, se o HTML servido não traz `infra_hash`
/// (credencial de sessão nunca vai a arquivo).
fn fotografar(versao: VersaoDocumento, html: String) -> Result<FotoDaVersao, ErroDocumentoObter> {
    if html.contains("infra_hash") {
        return Err(ErroDocumentoObter::ConteudoComLinkAssinado);
    }
    Ok(FotoDaVersao { versao, html })
}

/// GET da lista publicada na barra, interpretada pelo leitor do histórico.
fn ler_lista<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    url: &str,
) -> Result<HistoricoDocumento, ErroDocumentoObter> {
    let html = match rede.get(url).map_err(ErroDocumentoObter::Processo)? {
        Lida::Html { html, .. } => html,
        outra @ (Lida::Redirecionamento(_) | Lida::Login | Lida::Falha(_)) => {
            return Err(ErroDocumentoObter::Processo(fora_do_html(outra)));
        }
    };
    let numero = NumeroDocumento::novo(no.numero)
        .map_err(|_| ErroDocumentoObter::NumeroSeiDaArvoreDiferenteDoPedido)?;
    interpretar_historico(&html, &numero, no.id_documento, None)
        .map_err(ErroDocumentoObter::Versoes)
}

/// GET de um `documento_visualizar` assinado: tipo, texto e o HTML servido;
/// o título tem de ser do mesmo número SEI.
fn ler_pagina<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    url: &str,
    numero: &str,
) -> Result<(String, String, String), ErroDocumentoObter> {
    let html = match rede.get(url).map_err(ErroDocumentoObter::Processo)? {
        Lida::Html { html, .. } => html,
        outra @ (Lida::Redirecionamento(_) | Lida::Login | Lida::Falha(_)) => {
            return Err(ErroDocumentoObter::Processo(fora_do_html(outra)));
        }
    };
    let identidade = identidade_do_titulo(&html).map_err(ErroDocumentoObter::Titulo)?;
    if identidade.numero != numero {
        return Err(ErroDocumentoObter::TituloDeOutroDocumento);
    }
    let texto = texto_do_corpo(&html).map_err(ErroDocumentoObter::Corpo)?;
    Ok((identidade.tipo, texto, html))
}
