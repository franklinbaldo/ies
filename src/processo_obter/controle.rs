//! A linha do Controle cujo link do processo tem o texto **igual** ao NUP
//! (visões R e D: `tr#P{id}` com `a[href^=controlador.php?acao=procedimento_trabalhar]`,
//! observado em `UnB`, GO, PEN1, PEN3, PEN4 e UFRJ). Os leitores da caixa não
//! guardam o `href`; esta leitura usa o mesmo tokenizador de atributos.
use super::ErroProcessoObter;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;

fn ilegivel<T>(_: T) -> ErroProcessoObter {
    ErroProcessoObter::LinhaDoControleIlegivel
}

/// `href` relativo de `procedimento_trabalhar` para o id da linha.
fn eh_link_do_processo(href: &str, id: &str) -> bool {
    let Some(consulta) = href.strip_prefix("controlador.php?") else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::ProcedimentoTrabalhar)
        && consulta.unico("id_procedimento") == Ok(id)
}

/// `Ok(None)`: o NUP não está nesta página. `Ok(Some((id, href)))`: o id da
/// linha e o `href` com entidades decodificadas, ainda relativo. NUP em
/// duas linhas, ou dois links para ele na mesma linha, é recusa.
pub(super) fn link_do_processo_no_controle(
    html: &str,
    nup: &str,
) -> Result<Option<(String, String)>, ErroProcessoObter> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(ilegivel)?;
    let parser = dom.parser();
    let mut achado: Option<(String, String)> = None;
    let linhas = dom.query_selector("tr").into_iter().flatten();
    for linha in linhas {
        let Some(linha) = linha.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(id) = valor_do_atributo(linha, "id").map_err(ilegivel)? else {
            continue;
        };
        let Some(id) = id
            .strip_prefix('P')
            .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        else {
            continue;
        };
        let mut nesta_linha: Option<String> = None;
        for link in linha.query_selector(parser, "a").into_iter().flatten() {
            let Some(link) = link.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            if link.inner_text(parser) != nup {
                continue;
            }
            let Some(href) = valor_do_atributo(link, "href").map_err(ilegivel)? else {
                continue;
            };
            let href = decodificar_atributo(&href).ok_or_else(|| ilegivel(()))?;
            if !href.starts_with("controlador.php?acao=procedimento_trabalhar&") {
                continue;
            }
            if nesta_linha.is_some() {
                return Err(ErroProcessoObter::LinksDoProcessoRepetidosNoControle);
            }
            nesta_linha = Some(href);
        }
        let Some(href) = nesta_linha else {
            continue;
        };
        if !eh_link_do_processo(&href, id) {
            continue;
        }
        if achado.is_some() {
            return Err(ErroProcessoObter::ProcessoRepetidoNoControle);
        }
        achado = Some((id.to_owned(), href));
    }
    Ok(achado)
}
