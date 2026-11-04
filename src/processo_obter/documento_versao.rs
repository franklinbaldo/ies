//! Versão N e cópia de trabalho de `ies documento DOC` (Refs #88;
//! contracts/cli/documento-obter-versoes.md), depois da árvore conferida:
//! a barra do `DOCUMENTO` publica a lista de versões (link assinado) e a
//! edição; a lista (`documento_versao_listar`) publica, por linha, o
//! `documento_visualizar` assinado da versão. Nenhum link montado, nenhum
//! editor. `versao_atual` só do rodapé servido ou da lista lida (decisão
//! de custo: nenhum GET só para ela).
use super::super::EnvioDoProcesso;
use super::super::abertura::{Contexto, Lida, Rede};
use super::{E, fora_do_html};
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::arvore_processo::{EdicaoNaBarra, EnderecoDoConteudo, ler_barra_do_documento};
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
    /// `--saida` sem `--sem-copia`: preparar `DOC.vN.html` quando seguro.
    pub(crate) copia: bool,
}

/// Por que `versao_atual` não foi publicada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemVersaoAtual {
    /// O conteúdo lido não serve o rodapé `versão N` e a lista não foi pedida.
    NaoServidaNoConteudo,
    /// A cópia precisava da lista e a barra não a publica.
    ListaNaoPublicadaNaArvore,
}

impl SemVersaoAtual {
    pub(crate) fn motivo(self) -> &'static str {
        match self {
            Self::NaoServidaNoConteudo => "nao_servida_no_conteudo",
            Self::ListaNaoPublicadaNaArvore => "lista_de_versoes_nao_publicada_na_arvore",
        }
    }
}

/// Por que a cópia de trabalho não foi preparada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemCopia {
    NaoPedida,
    ListaNaoPublicadaNaArvore,
    DocumentoAssinado,
    EdicaoNaoPublicadaNaArvore,
    VersaoHistorica,
    ConteudoComLinkAssinado,
}

impl SemCopia {
    pub(crate) fn motivo(self) -> &'static str {
        match self {
            Self::NaoPedida => "nao_pedida",
            Self::ListaNaoPublicadaNaArvore => "lista_de_versoes_nao_publicada_na_arvore",
            Self::DocumentoAssinado => "documento_assinado",
            Self::EdicaoNaoPublicadaNaArvore => "edicao_nao_publicada_na_arvore",
            Self::VersaoHistorica => "versao_historica",
            Self::ConteudoComLinkAssinado => "conteudo_com_link_assinado",
        }
    }
}

/// O HTML servido da versão atual, que a CLI grava como `DOC.vN.html`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CopiaDeTrabalho {
    pub(crate) versao: VersaoDocumento,
    pub(crate) html: String,
}

impl std::fmt::Debug for CopiaDeTrabalho {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CopiaDeTrabalho(v{}, <html>)", self.versao.numero())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VersoesDoDocumento {
    /// A versão do texto publicado: N de `--versao N` ou a do rodapé.
    pub(crate) lida: Option<VersaoDocumento>,
    pub(crate) atual: Result<VersaoDocumento, SemVersaoAtual>,
    pub(crate) copia: Result<CopiaDeTrabalho, SemCopia>,
}

/// O documento já conferido na árvore: contexto (instalação e número
/// pedido), HTML da árvore, `id_documento` e `id_procedimento`.
pub(super) struct NoConferido<'a>(
    pub(super) &'a Contexto<'a>,
    pub(super) &'a str,
    pub(super) &'a str,
    pub(super) &'a str,
);

impl NoConferido<'_> {
    fn instalacao(&self) -> &Instalacao {
        &self.0.instalacao
    }
    fn numero(&self) -> &str {
        self.0.nup
    }
}

/// Tipo, texto e versões do documento, com o mínimo de GETs do pedido.
pub(super) fn ler_versoes<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    conteudo: &EnderecoDoConteudo,
    pedido: PedidoDeVersoes,
) -> Result<(String, String, VersoesDoDocumento), E> {
    match pedido.versao {
        None => ler_atual(rede, no, conteudo, pedido.copia),
        Some(n) => ler_versao(rede, no, n, pedido.copia),
    }
}

/// O conteúdo atual (`Nos[i].src`); a versão vem do rodapé servido. A
/// lista só é pedida para a cópia, quando o rodapé não declara a versão.
fn ler_atual<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    conteudo: &EnderecoDoConteudo,
    copia: bool,
) -> Result<(String, String, VersoesDoDocumento), E> {
    let (tipo, texto, html) = ler_pagina(rede, conteudo.as_str(), no.numero())?;
    let servida = versao_do_rodape(&html);
    let mut versoes = VersoesDoDocumento {
        lida: servida,
        atual: servida.ok_or(SemVersaoAtual::NaoServidaNoConteudo),
        copia: Err(SemCopia::NaoPedida),
    };
    if !copia {
        return Ok((tipo, texto, versoes));
    }
    let barra =
        ler_barra_do_documento(no.instalacao(), no.1, no.2, no.3).map_err(E::BarraDoDocumento)?;
    if let Some(sem) = inseguro(barra.edicao, &html) {
        versoes.copia = Err(sem);
        return Ok((tipo, texto, versoes));
    }
    let versao = match (servida, barra.versoes) {
        (Some(n), _) => n,
        (None, Some(lista)) => {
            let atual = ler_lista(rede, no, lista.as_str())?
                .atual()
                .ok_or(E::Versoes(ErroVersao::ListaIncompleta))?;
            versoes.atual = Ok(atual);
            atual
        }
        (None, None) => {
            versoes.atual = Err(SemVersaoAtual::ListaNaoPublicadaNaArvore);
            versoes.copia = Err(SemCopia::ListaNaoPublicadaNaArvore);
            return Ok((tipo, texto, versoes));
        }
    };
    versoes.copia = Ok(CopiaDeTrabalho { versao, html });
    Ok((tipo, texto, versoes))
}

/// `--versao N`: a lista publicada na barra e o `documento_visualizar` da
/// linha N; o conteúdo atual não é pedido.
fn ler_versao<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    n: VersaoDocumento,
    copia: bool,
) -> Result<(String, String, VersoesDoDocumento), E> {
    let barra =
        ler_barra_do_documento(no.instalacao(), no.1, no.2, no.3).map_err(E::BarraDoDocumento)?;
    let lista = barra.versoes.ok_or(E::ListaDeVersoesNaoPublicadaNaArvore)?;
    let historico = ler_lista(rede, no, lista.as_str())?;
    let atual = historico
        .atual()
        .ok_or(E::Versoes(ErroVersao::ListaIncompleta))?;
    let entrada = historico.selecionar(n).map_err(E::Versoes)?;
    let url = conferir_endereco_relativo(
        no.instalacao(),
        entrada.link_privado(),
        "documento_visualizar",
        no.3,
    )
    .map_err(|_| E::Versoes(ErroVersao::LinkInseguro))?;
    let (tipo, texto, html) = ler_pagina(rede, &url, no.numero())?;
    if versao_do_rodape(&html).is_some_and(|servida| servida != n) {
        return Err(E::VersaoServidaDiferenteDaPedida);
    }
    let copia = if !copia {
        Err(SemCopia::NaoPedida)
    } else if n != atual {
        Err(SemCopia::VersaoHistorica)
    } else if let Some(sem) = inseguro(barra.edicao, &html) {
        Err(sem)
    } else {
        Ok(CopiaDeTrabalho { versao: n, html })
    };
    let versoes = VersoesDoDocumento {
        lida: Some(n),
        atual: Ok(atual),
        copia,
    };
    Ok((tipo, texto, versoes))
}

/// Por que a cópia não é segura: a barra não declara edição não assinada,
/// ou o HTML traz `infra_hash` (credencial de sessão nunca vai a arquivo).
fn inseguro(edicao: EdicaoNaBarra, html: &str) -> Option<SemCopia> {
    match edicao {
        EdicaoNaBarra::NaoAssinado if html.contains("infra_hash") => {
            Some(SemCopia::ConteudoComLinkAssinado)
        }
        EdicaoNaBarra::NaoAssinado => None,
        EdicaoNaBarra::Assinado => Some(SemCopia::DocumentoAssinado),
        EdicaoNaBarra::NaoPublicada | EdicaoNaBarra::ForaDaForma => {
            Some(SemCopia::EdicaoNaoPublicadaNaArvore)
        }
    }
}

/// GET da lista publicada na barra, interpretada pelo leitor do histórico.
fn ler_lista<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    url: &str,
) -> Result<HistoricoDocumento, E> {
    let html = match rede.get(url).map_err(E::Processo)? {
        Lida::Html { html, .. } => html,
        outra => return Err(E::Processo(fora_do_html(outra))),
    };
    let numero =
        NumeroDocumento::novo(no.numero()).map_err(|_| E::NumeroSeiDaArvoreDiferenteDoPedido)?;
    interpretar_historico(&html, &numero, no.2, None).map_err(E::Versoes)
}

/// GET de um `documento_visualizar` assinado: tipo, texto e o HTML servido;
/// o título tem de ser do mesmo número SEI.
fn ler_pagina<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    url: &str,
    numero: &str,
) -> Result<(String, String, String), E> {
    let html = match rede.get(url).map_err(E::Processo)? {
        Lida::Html { html, .. } => html,
        outra => return Err(E::Processo(fora_do_html(outra))),
    };
    let identidade = identidade_do_titulo(&html).map_err(E::Titulo)?;
    if identidade.numero != numero {
        return Err(E::TituloDeOutroDocumento);
    }
    let texto = texto_do_corpo(&html).map_err(E::Corpo)?;
    Ok((identidade.tipo, texto, html))
}
