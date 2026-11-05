//! O link da troca no Controle e a consulta dos links assinados.
use super::{ACAO_DE_TROCAR, E};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;

/// URL absoluta do link de troca publicado por `a#lnkInfraUnidade` (todos
/// iguais) na tela servida em `url_da_tela`.
pub(crate) fn link_da_troca(
    instalacao: &Instalacao,
    url_da_tela: &str,
    html: &str,
) -> Result<String, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        E::Html(crate::erro_html::ErroHtml::ausente(
            "link_de_troca_de_unidade_ausente",
            "o Controle não publica o link de troca de unidade",
        ))
    })?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "lnkInfraUnidade").map_err(|e| {
        E::Html(crate::erro_html::ErroHtml::atributo(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
            e,
        ))
    })?;
    let mut onclicks = Vec::new();
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if let Some(bruto) = valor_do_atributo(tag, "onclick").map_err(|e| {
            E::Html(crate::erro_html::ErroHtml::atributo(
                "link_de_troca_de_unidade_fora_da_forma_observada",
                "o link de troca de unidade não tem a forma observada",
                e,
            ))
        })? {
            onclicks.push(decodificar_atributo(&bruto).ok_or(E::Html(
                crate::erro_html::ErroHtml::valor(
                    "link_de_troca_de_unidade_fora_da_forma_observada",
                    "o link de troca de unidade não tem a forma observada",
                ),
            ))?);
        }
    }
    let Some(primeiro) = onclicks.first() else {
        return Err(E::Html(crate::erro_html::ErroHtml::ausente(
            "link_de_troca_de_unidade_ausente",
            "o Controle não publica o link de troca de unidade",
        )));
    };
    if onclicks.iter().any(|o| o != primeiro) {
        return Err(E::LinksDeTrocaDivergentes);
    }
    let relativo = primeiro
        .trim()
        .strip_prefix("window.location.href='")
        .and_then(|r| r.strip_suffix("';"))
        .ok_or(E::Html(crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        )))?;
    let consulta = relativo.strip_prefix("controlador.php?").ok_or(E::Html(
        crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        ),
    ))?;
    if !relativo.bytes().all(|b| (33..=126).contains(&b)) || relativo.contains(['#', '\\', '\'']) {
        return Err(E::Html(crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        )));
    }
    conferir_consulta(consulta, ACAO_DE_TROCAR, |h| h.len() == 64).map_err(
        |ConsultaForaDaForma| {
            E::Html(crate::erro_html::ErroHtml::valor(
                "link_de_troca_de_unidade_fora_da_forma_observada",
                "o link de troca de unidade não tem a forma observada",
            ))
        },
    )?;
    let base = url_da_tela.split('?').next().unwrap_or_default();
    let (pasta, _) = base
        .rsplit_once('/')
        .ok_or(E::Html(crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        )))?;
    let url = format!("{pasta}/{relativo}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or(E::Html(crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        )))?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(E::Html(crate::erro_html::ErroHtml::valor(
            "link_de_troca_de_unidade_fora_da_forma_observada",
            "o link de troca de unidade não tem a forma observada",
        )));
    }
    Ok(url)
}

/// Confere a consulta de um link assinado: chaves canônicas, valores sem
/// `;`, `acao` única igual a `acao`, um `infra_hash` hexadecimal aceito por
/// `comprimento`, sem `acao_ajax`. Devolve o `infra_unidade_atual` (único),
/// se houver.
pub(crate) fn conferir_consulta(
    consulta: &str,
    acao: &str,
    comprimento: impl Fn(&str) -> bool,
) -> Result<Option<String>, ConsultaForaDaForma> {
    let mut unidades = conferir(consulta, acao, comprimento).ok_or(ConsultaForaDaForma)?;
    if unidades.len() > 1 {
        return Err(ConsultaForaDaForma);
    }
    Ok(unidades.pop())
}

/// A consulta não tem a forma conferida por [`conferir_consulta`].
#[derive(Debug)]
pub(crate) struct ConsultaForaDaForma;

fn conferir(consulta: &str, acao: &str, comprimento: impl Fn(&str) -> bool) -> Option<Vec<String>> {
    use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
    let (mut acoes, mut hashes) = (0, 0);
    let mut unidades = Vec::new();
    for par in consulta.split('&') {
        let (nome, valor) = par.split_once('=')?;
        conferir_chave_de_consulta(nome).ok()?;
        conferir_valor_de_consulta(valor).ok()?;
        match nome {
            "acao" if valor == acao => acoes += 1,
            "infra_hash" if comprimento(valor) && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "infra_unidade_atual" => unidades.push(valor.to_owned()),
            "acao" | "infra_hash" | "acao_ajax" => return None,
            _ => {}
        }
    }
    (acoes == 1 && hashes == 1).then_some(unidades)
}
