//! A ação "Concluir Processo" publicada na barra do nó `PROCESSO` e, na
//! forma do SEI 4.0, a URL do corpo de `concluirProcesso()` na tela do nó
//! (contracts/cli/processo-concluir.md; observação em
//! docs/como-o-sei-funciona/conclusao-do-processo.md). Duas formas:
//! - 4.1 e 5.0 (UFRJ, PEN2, GO, PEN3, PEN4): `<a href="controlador.php?
//!   acao=procedimento_concluir&…">`, link da tela "Conclusão de Processo";
//! - 4.0 (`UnB` 4.0.9, PEN1 4.0.12): `<a href="#"
//!   onclick="concluirProcesso();">`; a URL está na tela do nó
//!   (`arvore_visualizar`, o `href` a3 do `Nos[i]` `PROCESSO`), em três
//!   linhas: a da função, a do `confirm` comentado e a do `location.href`.
//!
//! Sem regex e sem JavaScript executado: só essas formas exatas.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, ler_declaracao};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

/// A `acao` da conclusão nas duas formas.
pub(crate) const ACAO_DE_CONCLUIR: &str = "procedimento_concluir";
/// A `acao` da tela do nó, onde a forma 4.0 publica a URL.
pub(crate) const ACAO_DA_TELA_DO_NO: &str = "arvore_visualizar";
const ONCLICK: &str = "concluirProcesso();";

/// URL absoluta e assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDeEscrita(String);

impl EnderecoDeEscrita {
    pub(super) fn novo(url: String) -> Self {
        Self(url)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDeEscrita {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDeEscrita(<redigido>)")
    }
}

/// Como a barra oferece concluir.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConcluirNaBarra {
    /// Link da tela "Conclusão de Processo" (forma `tela`).
    PelaTela(EnderecoDeEscrita),
    /// `concluirProcesso()` (forma `funcao`): a tela do nó, onde está a URL.
    PelaFuncao { tela_do_no: EnderecoDeEscrita },
}

/// Recusas da leitura da barra (só pesam quando se pede para concluir).
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroConcluirNaBarra {
    EnderecoDaConclusaoDeOutroProcesso,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl ErroConcluirNaBarra {}

/// A ação de concluir da barra do processo `id`; `Ok(None)` se a barra
/// não oferece concluir (nem link nem função).
pub(crate) fn ler_concluir_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<ConcluirNaBarra>, ErroConcluirNaBarra> {
    use ErroConcluirNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => E::Html(ErroHtml::valor("barra_do_processo_fora_da_forma_observada")),
        FalhaDaBarra::Repetidas => {
            E::Html(ErroHtml::repetido("barras_do_processo_repetidas_na_arvore"))
        }
    })?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_DE_CONCLUIR))
        .collect();
    let funcoes = ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some(ONCLICK))
        .count();
    match (links.as_slice(), funcoes) {
        ([], 0) => Ok(None),
        ([href], 0) => conferir_endereco_relativo(instalacao, href, ACAO_DE_CONCLUIR, id)
            .map(|url| Some(ConcluirNaBarra::PelaTela(EnderecoDeEscrita(url))))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    E::Html(ErroHtml::valor("endereco_da_conclusao_nao_reconhecido"))
                }
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDaConclusaoDeOutroProcesso,
            }),
        ([], 1) => tela_do_no_da_raiz(instalacao, html, id)
            .map(|tela_do_no| Some(ConcluirNaBarra::PelaFuncao { tela_do_no }))
            .ok_or(E::Html(ErroHtml::valor(
                "endereco_da_tela_do_no_nao_reconhecido",
            ))),
        _ => Err(ErroHtml::repetido("formas_de_concluir_repetidas_na_barra").into()),
    }
}

/// O `href` (a3) da declaração do nó `PROCESSO` do id, conferido como
/// `arvore_visualizar` assinado do mesmo processo (também a de reabrir).
pub(super) fn tela_do_no_da_raiz(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
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
            && tipo == "PROCESSO"
            && no == id
        {
            achados.push(href.clone());
        }
    }
    let [href] = achados.as_slice() else {
        return None;
    };
    conferir_endereco_relativo(instalacao, href, ACAO_DA_TELA_DO_NO, id)
        .ok()
        .map(EnderecoDeEscrita)
}

/// Recusas da leitura de `concluirProcesso()` na tela do nó.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroFuncaoDeConcluir {
    EnderecoDaConclusaoDeOutroProcesso,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl ErroFuncaoDeConcluir {}

const CONFIRM: &str = "if (confirm(";
const LOCATION: &str = "location.href = '";

/// A definição de uma função da tela do nó fora da forma de três linhas.
pub(super) struct FuncaoForaDaForma;

/// As URLs de `location.href` de cada `function {nome}(){` da tela do nó,
/// na forma observada no SEI 4.0: a linha da função, a do `confirm`
/// (comentado ou ativo, como na retirada do sobrestamento da `UnB`) e a do
/// `location.href = '…';` (também a de reabrir).
pub(super) fn urls_das_funcoes<'h>(
    html: &'h str,
    nome: &str,
) -> Result<Vec<&'h str>, FuncaoForaDaForma> {
    let funcao = format!("function {nome}(){{");
    // Com o parêntese: `cienciaProcessoAnexado` não é `cienciaProcesso`.
    let prefixo = format!("function {nome}(");
    let linhas: Vec<&str> = html.lines().map(str::trim).collect();
    let mut achados = Vec::new();
    for (i, linha) in linhas.iter().enumerate() {
        if !linha.contains(&prefixo) {
            continue;
        }
        let confirm = linhas.get(i + 1).copied().unwrap_or_default();
        let confirm = confirm.strip_prefix("//").unwrap_or(confirm);
        let url = (*linha == funcao && confirm.starts_with(CONFIRM) && confirm.ends_with("){"))
            .then(|| linhas.get(i + 2))
            .flatten()
            .and_then(|l| l.strip_prefix(LOCATION))
            .and_then(|resto| resto.strip_suffix("';"))
            .ok_or(FuncaoForaDaForma)?;
        achados.push(url);
    }
    Ok(achados)
}

/// As URLs de `var {nome} = '…';` da tela do nó (SEI 4.1 e 5.0). Outra
/// variável com o mesmo prefixo (`linkCienciaProcessoAnexado`) não conta.
pub(super) fn urls_das_variaveis<'h>(
    html: &'h str,
    nome: &str,
) -> Result<Vec<&'h str>, FuncaoForaDaForma> {
    let declaracao = format!("var {nome}");
    let ate_a_url = format!("var {nome} = '");
    let mut achados = Vec::new();
    for linha in html.lines().map(str::trim) {
        let Some(resto) = linha.strip_prefix(&declaracao) else {
            continue;
        };
        if !resto.starts_with([' ', '=']) {
            continue;
        }
        let url = linha
            .strip_prefix(&ate_a_url)
            .and_then(|s| s.strip_suffix("';"))
            .ok_or(FuncaoForaDaForma)?;
        achados.push(url);
    }
    Ok(achados)
}

/// A URL de `concluirProcesso()` na tela do nó do processo `id`, conferida.
pub(crate) fn ler_funcao_de_concluir(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDeEscrita, ErroFuncaoDeConcluir> {
    use ErroFuncaoDeConcluir as E;
    let achados = urls_das_funcoes(html, "concluirProcesso").map_err(|FuncaoForaDaForma| {
        ErroHtml::valor("funcao_de_concluir_fora_da_forma_observada")
    })?;
    let url = match achados.as_slice() {
        [] => return Err(ErroHtml::ausente("funcao_de_concluir_ausente_na_tela_do_no").into()),
        [url] => *url,
        _ => return Err(ErroHtml::repetido("funcoes_de_concluir_repetidas_na_tela_do_no").into()),
    };
    conferir_endereco_relativo(instalacao, url, ACAO_DE_CONCLUIR, id)
        .map(EnderecoDeEscrita)
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                E::Html(ErroHtml::valor("endereco_da_conclusao_nao_reconhecido"))
            }
            EnderecoRecusado::DeOutroProcesso => E::EnderecoDaConclusaoDeOutroProcesso,
        })
}
