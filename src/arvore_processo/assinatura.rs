//! A assinatura do `DOCUMENTO` na árvore (Refs #174;
//! contracts/cli/documento-assinar.md; observação em
//! docs/como-o-sei-funciona/assinatura-do-documento.md): a ação
//! `onclick="assinarDocumento();"` da barra, a caneta (`NosAcoes[j] = new
//! infraArvoreAcao("ASSINATURA",…)` com a dica "Assinado por:\nnome\ncargo\n
//! unidade"), o `href` assinado da tela do documento (a3 do nó) e, nessa
//! tela, o link `documento_assinar` nas duas formas observadas (4.0:
//! `function assinarDocumento(){` + `parent.infraAbrirJanelaModal('URL',
//! 600,450);` + `}`; 4.1 e 5.0: `var linkAssinarDocumento = 'URL';`).
//! Sem regex e sem JavaScript executado: só essas formas exatas.
use super::barra_do_documento::barra_do_no;
use super::concluir::{ACAO_DA_TELA_DO_NO, EnderecoDeEscrita};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use super::literais::{Literal, ler_chamada, ler_declaracao};
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

/// A `acao` da tela "Assinatura de Documento".
pub(crate) const ACAO_DE_ASSINAR: &str = "documento_assinar";
const ONCLICK: &str = "assinarDocumento();";
const CANETA: &str = "infraArvoreAcao(\"ASSINATURA\"";
const DICA: &str = "Assinado por:\n";

/// Um assinante da dica da caneta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Assinante {
    pub(crate) nome: String,
    pub(crate) cargo: String,
    pub(crate) unidade: String,
}

/// O que a árvore mostra da assinatura do documento.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssinaturaNaArvore {
    /// A barra publica `assinarDocumento()`.
    pub(crate) publicada: bool,
    /// Os assinantes da caneta; vazio sem caneta.
    pub(crate) assinantes: Vec<Assinante>,
}

/// A ação da barra e a caneta do `DOCUMENTO` de id `id_documento`.
pub(crate) fn ler_assinatura_na_arvore(
    html: &str,
    id_documento: &str,
) -> Result<AssinaturaNaArvore, ErroHtml> {
    use ErroHtml as E;
    let barra = barra_do_no(html, id_documento).map_err(|erro| match erro {
        super::ErroBarraDoDocumento::BarrasRepetidasNaArvore => {
            E::repetido("barras_do_documento_repetidas_na_arvore")
        }
        _ => E::valor("barra_do_documento_fora_da_forma_observada"),
    })?;
    let acoes = match barra {
        Some(texto) => acoes_de_assinar(&texto)?,
        None => 0,
    };
    if acoes > 1 {
        return Err(E::repetido("acoes_de_assinar_repetidas_na_barra"));
    }
    Ok(AssinaturaNaArvore {
        publicada: acoes == 1,
        assinantes: assinantes(html, id_documento)?,
    })
}

/// Quantas âncoras da barra têm exatamente `onclick="assinarDocumento();"`.
fn acoes_de_assinar(texto: &str) -> Result<usize, ErroHtml> {
    let fora = ErroHtml::valor("barra_do_documento_fora_da_forma_observada");
    let dom = tl::parse(texto, tl::ParserOptions::default()).map_err(|_| fora)?;
    let parser = dom.parser();
    let mut contagem = 0;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(|_| fora)?;
        if let Some(bruto) = lidos.valor("onclick").map_err(|_| fora)? {
            let onclick = decodificar_atributo(bruto).ok_or(fora)?;
            contagem += usize::from(onclick == ONCLICK);
        }
    }
    Ok(contagem)
}

/// Os assinantes da única caneta do documento; toda linha de caneta precisa
/// estar na forma observada (falhar fechado: a verificação depende dela).
fn assinantes(html: &str, id: &str) -> Result<Vec<Assinante>, ErroHtml> {
    use ErroHtml as E;
    let mut achada = None;
    for linha in html.split('\n').filter(|l| l.contains(CANETA)) {
        let argumentos = ler_chamada(linha, "NosAcoes[", "infraArvoreAcao")
            .ok_or(E::valor("assinaturas_do_documento_fora_da_forma_observada"))?;
        let [
            Literal::Texto(tipo),
            Literal::Texto(chave),
            Literal::Texto(no),
            Literal::Texto(_),
            Literal::Nulo | Literal::Texto(_),
            Literal::Texto(dica),
            Literal::Texto(_),
            Literal::Booleano(true),
        ] = argumentos.as_slice()
        else {
            return Err(E::valor("assinaturas_do_documento_fora_da_forma_observada"));
        };
        if tipo != "ASSINATURA" || *chave != format!("A{no}") {
            return Err(E::valor("assinaturas_do_documento_fora_da_forma_observada"));
        }
        if no != id {
            continue;
        }
        let lidos =
            ler_dica(dica).ok_or(E::valor("assinaturas_do_documento_fora_da_forma_observada"))?;
        if achada.replace(lidos).is_some() {
            return Err(E::repetido("assinaturas_do_documento_repetidas_na_arvore"));
        }
    }
    Ok(achada.unwrap_or_default())
}

/// "Assinado por:\nnome\ncargo\nunidade": a única forma observada.
fn ler_dica(dica: &str) -> Option<Vec<Assinante>> {
    let resto = dica.strip_prefix(DICA)?;
    let partes: Vec<&str> = resto.split('\n').collect();
    let [nome, cargo, unidade] = partes.as_slice() else {
        return None;
    };
    if [nome, cargo, unidade].iter().any(|p| p.trim().is_empty()) {
        return None;
    }
    Some(vec![Assinante {
        nome: (*nome).to_owned(),
        cargo: (*cargo).to_owned(),
        unidade: (*unidade).to_owned(),
    }])
}

/// O `href` (a3) do nó `DOCUMENTO` do id, conferido como `arvore_visualizar`
/// assinado do mesmo documento e processo.
pub(crate) fn tela_do_documento(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Option<EnderecoDeEscrita> {
    let mut achados = Vec::new();
    for linha in html.split('\n') {
        let Ok(Some((_, argumentos))) = ler_declaracao(linha) else {
            continue;
        };
        if let [
            Literal::Texto(tipo),
            Literal::Texto(no),
            _,
            Literal::Texto(href),
            ..,
        ] = argumentos.as_slice()
            && tipo == "DOCUMENTO"
            && no == id_documento
        {
            achados.push(href.clone());
        }
    }
    let [href] = achados.as_slice() else {
        return None;
    };
    let por = |chave, id| {
        conferir_endereco_relativo_por(instalacao, href, ACAO_DA_TELA_DO_NO, chave, id).ok()
    };
    por("id_procedimento", id_procedimento)?;
    por("id_documento", id_documento).map(EnderecoDeEscrita::novo)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDeAssinar {
    EnderecoDeAssinarDeOutroDocumento,
    Html(#[from] ErroHtml),
}

const VARIAVEL: &str = "var linkAssinarDocumento = '";
const FUNCAO: &str = "function assinarDocumento(){";
const JANELA: &str = "parent.infraAbrirJanelaModal('";
const JANELA_FIM: &str = "',600,450);";

/// O link `documento_assinar` da tela do documento, conferido.
pub(crate) fn ler_link_de_assinar(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<EnderecoDeEscrita, ErroLinkDeAssinar> {
    use ErroLinkDeAssinar as E;
    let linhas: Vec<&str> = html.lines().map(str::trim).collect();
    let mut achados = Vec::new();
    for (i, linha) in linhas.iter().enumerate() {
        if let Some(resto) = linha.strip_prefix(VARIAVEL) {
            achados.push(
                resto
                    .strip_suffix("';")
                    .ok_or(ErroHtml::valor("link_de_assinar_fora_da_forma_observada"))?,
            );
        } else if linha.contains("function assinarDocumento") {
            let url = (*linha == FUNCAO)
                .then(|| linhas.get(i + 1))
                .flatten()
                .and_then(|l| l.strip_prefix(JANELA))
                .and_then(|resto| resto.strip_suffix(JANELA_FIM))
                .filter(|_| linhas.get(i + 2) == Some(&"}"))
                .ok_or(ErroHtml::valor("link_de_assinar_fora_da_forma_observada"))?;
            achados.push(url);
        }
    }
    let url = match achados.as_slice() {
        [] => return Err(ErroHtml::ausente("link_de_assinar_ausente_na_tela_do_documento").into()),
        [url] => *url,
        _ => {
            return Err(
                ErroHtml::repetido("links_de_assinar_repetidos_na_tela_do_documento").into(),
            );
        }
    };
    let recusa = |r| match r {
        EnderecoRecusado::NaoReconhecido => {
            E::Html(ErroHtml::valor("endereco_de_assinar_nao_reconhecido"))
        }
        EnderecoRecusado::DeOutroProcesso => E::EnderecoDeAssinarDeOutroDocumento,
    };
    conferir_endereco_relativo_por(
        instalacao,
        url,
        ACAO_DE_ASSINAR,
        "id_procedimento",
        id_procedimento,
    )
    .map_err(recusa)?;
    conferir_endereco_relativo_por(
        instalacao,
        url,
        ACAO_DE_ASSINAR,
        "id_documento",
        id_documento,
    )
    .map(EnderecoDeEscrita::novo)
    .map_err(recusa)
}
