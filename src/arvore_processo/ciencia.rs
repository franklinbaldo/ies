//! Links de ciência no processo publicados pela árvore. A leitura da barra
//! usa o leitor compartilhado de atributos, e a URL da Lista de Ciências
//! vem exclusivamente da ação CIENCIAS da raiz. Nada executa JavaScript.
mod javascript;
#[cfg(test)]
mod tests_unb;
pub(crate) use javascript::niveis_js;

use super::barra_do_processo::ancoras_da_barra;
use super::concluir::{
    EnderecoDeEscrita, FuncaoForaDaForma, tela_do_no_da_raiz, urls_das_funcoes, urls_das_variaveis,
};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, ler_chamada};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{ConsultaDoControlador, ErroConsultaDoControlador};
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCienciaNaArvore {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    EnderecoDaCienciaDoDocumentoRecusado(
        #[source] crate::arvore_processo::endereco::EnderecoRecusado,
    ),
    EnderecoDaListaDeCienciasRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    /// A consulta do link de ciência não se deixa ler em pares.
    ConsultaDaCienciaIlegivel(#[source] ErroConsultaDoControlador),
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    CienciaRepetidaNaBarra,
    TelaDeCienciaNaoPublicada,
    FuncaoDeCienciaAusenteNaTela,
    FuncaoDeCienciaForaDaForma,
    FuncoesDeCienciaRepetidas,
    EnderecoDaCienciaNaoReconhecido,
    ListaDeCienciasRepetidaNaArvore,
    ListaDeCienciasForaDaForma,
    EnderecoDaListaDeCienciasNaoReconhecido,
    EnderecoDaCienciaDeOutroProcesso,
    FuncaoDeCienciaDoDocumentoForaDaForma,
    LinkDeCienciaDoDocumentoAusente,
    LinkDeCienciaDoDocumentoForaDaForma,
    LinksDeCienciaDoDocumentoRepetidos,
    ListaDeCienciasDoDocumentoComAlvoDivergente,
    ListaDeCienciasDoDocumentoForaDaForma,
    ListasDeCienciasDoDocumentoRepetidas,
    ScriptDaCienciaDoDocumentoForaDaForma,
}

/// A barra só oferece ciência quando existe exatamente um botão da função.
/// A URL é a tela de nó, jamais a URL que já grava a ciência.
pub(crate) fn ciencia_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroCienciaNaArvore> {
    let ancoras = ancoras_da_barra(html)?;
    let total = ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some("cienciaProcesso();"))
        .count();
    match total {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id)
            .map(Some)
            .ok_or(ErroCienciaNaArvore::TelaDeCienciaNaoPublicada),
        _ => Err(ErroCienciaNaArvore::CienciaRepetidaNaBarra),
    }
}

/// Link assinado único da Lista de Ciências do PROCESSO na árvore novamente
/// aberta. A do documento (código CD<id>) não serve como prova do processo.
pub(crate) fn lista_do_processo(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroCienciaNaArvore> {
    let mut urls = Vec::new();
    for linha in html.lines() {
        if !linha.contains("new infraArvoreAcao(") || !linha.contains("CIENCIAS") {
            continue;
        }
        let argumentos = ler_chamada(linha, "NosAcoes[", "infraArvoreAcao")
            .ok_or(ErroCienciaNaArvore::ListaDeCienciasForaDaForma)?;
        match argumentos.as_slice() {
            [
                Literal::Texto(tipo),
                Literal::Texto(codigo),
                Literal::Texto(alvo),
                Literal::Texto(url),
                ..,
            ] if tipo == "CIENCIAS" && codigo == "C" && alvo == id => urls.push(url.clone()),
            [Literal::Texto(tipo), ..] if tipo == "CIENCIAS" => {
                // A ação CD... é do documento e não confere ciência no processo.
            }
            _ => return Err(ErroCienciaNaArvore::ListaDeCienciasForaDaForma),
        }
    }
    let url = match urls.as_slice() {
        [] => return Ok(None),
        [url] => url,
        _ => return Err(ErroCienciaNaArvore::ListaDeCienciasRepetidaNaArvore),
    };
    conferir_endereco_relativo(instalacao, url, AcaoDoSei::ProtocoloCienciaListar, id)
        .map(|url| Some(EnderecoDeEscrita::novo(url)))
        .map_err(|_: EnderecoRecusado| ErroCienciaNaArvore::EnderecoDaListaDeCienciasNaoReconhecido)
}

/// Lê só a URL assinada da função do SEI na tela do nó. Um GET nessa
/// URL já registra a ciência, portanto não pode ser usado na prévia.
pub(crate) fn url_da_ciencia(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDeEscrita, ErroCienciaNaArvore> {
    let funcoes = urls_das_funcoes(html, "cienciaProcesso")
        .map_err(|FuncaoForaDaForma| ErroCienciaNaArvore::FuncaoDeCienciaForaDaForma)?;
    // SEI 4.1 e 5.0 (UFRJ e GO, 10/11/2026): `var linkCienciaProcesso = '…';`.
    let variaveis = urls_das_variaveis(html, "linkCienciaProcesso")
        .map_err(|FuncaoForaDaForma| ErroCienciaNaArvore::FuncaoDeCienciaForaDaForma)?;
    let url = match (funcoes.as_slice(), variaveis.as_slice()) {
        ([], []) => return Err(ErroCienciaNaArvore::FuncaoDeCienciaAusenteNaTela),
        ([url], []) | ([], [url]) => *url,
        _ => return Err(ErroCienciaNaArvore::FuncoesDeCienciaRepetidas),
    };
    conferir_endereco_relativo(instalacao, url, AcaoDoSei::ProcedimentoCiencia, id)
        .map(EnderecoDeEscrita::novo)
        .map_err(|_: EnderecoRecusado| ErroCienciaNaArvore::EnderecoDaCienciaNaoReconhecido)
}

/// A ciência do documento é oferecida somente pela barra do próprio nó.
/// O documento sem assinatura pode não publicar o botão (`UnB` 4.0.9).
pub(crate) fn ciencia_do_documento_na_barra(
    html: &str,
    id_documento: &str,
) -> Result<bool, super::ErroBarraDoDocumento> {
    use crate::html_atributos::atributos;
    let Some(barra) = super::barra_do_documento::barra_do_no(html, id_documento)? else {
        return Ok(false);
    };
    let dom = crate::tela_html::interpretar(&barra);
    let mut total = 0;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(super::ErroBarraDoDocumento::AtributoDeLinkDaBarra)?;
        if let Some(onclick) = lidos.valor("onclick") {
            let onclick = crate::formulario_web::decodificar_atributo(onclick);
            if onclick == "cienciaDocumento();" {
                total += 1;
            }
        }
    }
    match total {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(super::ErroBarraDoDocumento::BotoesDeCienciaDoDocumentoRepetidos),
    }
}

/// O GET mutador vem da tela assinada do nó. Observados na `UnB`: função
/// ciênciaDocumento (4.0) e linkCienciaDocumento (4.1/5.0). Lê a fonte,
/// nunca executa JS. A URL às vezes não traz `id_procedimento`, mas a tela
/// do nó já foi vinculada ao processo na árvore.
pub(crate) fn url_da_ciencia_do_documento(
    instalacao: &Instalacao,
    html: &crate::tela_html::HtmlServido,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<EnderecoDeEscrita, ErroCienciaNaArvore> {
    // Só o conteúdo de scripts inline ativos comprova o link do GET
    // mutador. Texto em templates, campos e scripts de dados não é ação
    // publicada pelo SEI; nunca se interpreta nem executa JavaScript.
    // Conservador: uma declaração em <script> dentro de <template> nunca
    // autoriza o GET de escrita, mesmo se o seletor DOM listar o nó inerte.
    // Rejeita a tela inteira quando a região inerte contém script: não
    // precisa inferir qual ramo o navegador ativaria nem executar JS.
    let minusculas = html.to_ascii_lowercase();
    if minusculas.split("<template").skip(1).any(|resto| {
        resto
            .split("</template")
            .next()
            .is_some_and(|parte| parte.contains("<script"))
    }) {
        return Err(ErroCienciaNaArvore::ScriptDaCienciaDoDocumentoForaDaForma);
    }
    let dom = crate::tela_html::interpretar(html);
    let mut urls = Vec::<String>::new();
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(tag) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = crate::html_atributos::atributos(tag).map_err(ErroCienciaNaArvore::Atributo)?;
        if lidos.valor("src").is_some() {
            continue;
        }
        let tipo = lidos.valor("type");
        if tipo.is_some_and(|t| t != "text/javascript" && t != "application/javascript") {
            continue;
        }
        // Fonte bruta, não inner_text: a `UnB` 4.0.9 envolve o script em
        // `<!--//--><![CDATA[//><!--`, que inner_text omite (live 10/11/2026).
        let script = tag.raw().as_utf8_str();
        urls.extend(javascript::urls_ativas_da_ciencia_do_documento(
            script.as_ref(),
        )?);
    }
    let url = match urls.as_slice() {
        [] => {
            return Err(ErroCienciaNaArvore::LinkDeCienciaDoDocumentoAusente);
        }
        [url] => url.as_str(),
        _ => {
            return Err(ErroCienciaNaArvore::LinksDeCienciaDoDocumentoRepetidos);
        }
    };
    let absoluta = super::endereco::conferir_endereco_relativo_por(
        instalacao,
        url,
        AcaoDoSei::DocumentoCiencia,
        "id_documento",
        id_documento,
    )
    .map_err(ErroCienciaNaArvore::EnderecoDaCienciaDoDocumentoRecusado)?;
    // Se a instalação também publicou id_procedimento, ele tem de ser único
    // e corresponder ao processo; a ausência é a forma observada na UnB.
    let consulta = ConsultaDoControlador::ler(url.split_once('?').map_or("", |(_, q)| q))
        .map_err(ErroCienciaNaArvore::ConsultaDaCienciaIlegivel)?;
    let ids: Vec<&str> = consulta.valores("id_procedimento").collect();
    if !ids.is_empty() && ids.as_slice() != [id_procedimento] {
        return Err(ErroCienciaNaArvore::EnderecoDaCienciaDeOutroProcesso);
    }
    Ok(EnderecoDeEscrita::novo(absoluta))
}

/// Na árvore relida, só CD<id> é a Lista de Ciências do documento
/// procurado. A lista C da raiz e a de outro documento nunca provam.
pub(crate) fn lista_do_documento(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    id_documento: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroCienciaNaArvore> {
    let chave = format!("CD{id_documento}");
    let mut urls = Vec::new();
    for linha in html.lines() {
        if !linha.contains("new infraArvoreAcao(") || !linha.contains("CIENCIAS") {
            continue;
        }
        let argumentos = ler_chamada(linha, "NosAcoes[", "infraArvoreAcao")
            .ok_or(ErroCienciaNaArvore::ListaDeCienciasDoDocumentoForaDaForma)?;
        match argumentos.as_slice() {
            [
                Literal::Texto(tipo),
                Literal::Texto(codigo),
                Literal::Texto(alvo),
                Literal::Texto(url),
                ..,
            ] if tipo == "CIENCIAS" && codigo == &chave && alvo == id_documento => {
                urls.push(url.clone());
            }
            [Literal::Texto(tipo), Literal::Texto(codigo), ..]
                if tipo == "CIENCIAS" && codigo == &chave =>
            {
                return Err(ErroCienciaNaArvore::ListaDeCienciasDoDocumentoComAlvoDivergente);
            }
            [Literal::Texto(tipo), ..] if tipo == "CIENCIAS" => {}
            _ => {
                return Err(ErroCienciaNaArvore::ListaDeCienciasDoDocumentoForaDaForma);
            }
        }
    }
    let url = match urls.as_slice() {
        [] => return Ok(None),
        [url] => url,
        _ => {
            return Err(ErroCienciaNaArvore::ListasDeCienciasDoDocumentoRepetidas);
        }
    };
    let por = |chave, id| {
        super::endereco::conferir_endereco_relativo_por(
            instalacao,
            url,
            AcaoDoSei::ProtocoloCienciaListar,
            chave,
            id,
        )
        .map_err(ErroCienciaNaArvore::EnderecoDaListaDeCienciasRecusado)
    };
    por("id_procedimento", id_procedimento)?;
    por("id_documento", id_documento).map(|u| Some(EnderecoDeEscrita::novo(u)))
}
