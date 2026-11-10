//! O link assinado do histórico na árvore, nas duas formas observadas
//! (03/11/2026, capturas privadas lidas só pela estrutura):
//! - `UnB` 4.0.9: o `onclick` de `div#divConsultarAndamento a` é
//!   `consultarAndamento();` e o `script` inline define, em duas linhas,
//!   `function consultarAndamento(){` e
//!   `parent.document.getElementById('ifrVisualizacao').src = '<URL>';`;
//! - GO 5.0.3: o `onclick` é `consultarAndamento('<URL>');` (a função não
//!   é definida na página).
//!
//! Só essas formas exatas; nada de regex nem de JavaScript executado.
use super::ErroLinkDoHistorico;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;

/// URL absoluta do histórico, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDoHistorico(String);

impl EnderecoDoHistorico {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDoHistorico {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDoHistorico(<redigido>)")
    }
}

const FUNCAO: &str = "function consultarAndamento(){";
const SRC: &str = "parent.document.getElementById('ifrVisualizacao').src = '";
const SEM_ARGUMENTO: &str = "consultarAndamento();";
const COM_ARGUMENTO: &str = "consultarAndamento('";

/// Lê o único link do histórico da árvore servida e o confere contra a
/// instalação e o id do processo aberto. `LinkDoHistoricoAusenteNaArvore`
/// é a árvore que não publica nenhuma das formas.
pub(crate) fn ler_link_do_historico(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDoHistorico, ErroLinkDoHistorico> {
    let mut achados = do_onclick(html)?;
    achados.extend(do_script(html)?);
    let url = match achados.as_slice() {
        [] => return Err(ErroLinkDoHistorico::LinkDoHistoricoAusenteNaArvore),
        [url] => url,
        _ => return Err(ErroLinkDoHistorico::LinksDoHistoricoRepetidosNaArvore),
    };
    conferir_endereco_relativo(instalacao, url, super::ACAO_DO_HISTORICO, id)
        .map(EnderecoDoHistorico)
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                ErroLinkDoHistorico::EnderecoDoHistoricoNaoReconhecido
            }
            EnderecoRecusado::DeOutroProcesso => {
                ErroLinkDoHistorico::EnderecoDoHistoricoDeOutroProcesso
            }
        })
}

/// Forma GO: URL no `onclick` de cada `a` dentro de `div#divConsultarAndamento`.
fn do_onclick(html: &str) -> Result<Vec<String>, ErroLinkDoHistorico> {
    let ilegivel = ErroLinkDoHistorico::HtmlDaArvoreNaoInterpretado;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ilegivel)?;
    let parser = dom.parser();
    let mut achados = Vec::new();
    for div in nos_com_id(&dom, "divConsultarAndamento").map_err(|_| ilegivel)? {
        let Some(div) = div.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        for a in div.query_selector(parser, "a").into_iter().flatten() {
            let Some(a) = a.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let bruto = valor_do_atributo(a, "onclick")
                .map_err(|_| ilegivel)?
                .ok_or(ErroLinkDoHistorico::LinkDoHistoricoForaDaFormaObservada)?;
            let onclick = decodificar_atributo(&bruto).ok_or(ilegivel)?;
            if onclick == SEM_ARGUMENTO {
                continue;
            }
            let url = onclick
                .strip_prefix(COM_ARGUMENTO)
                .and_then(|resto| resto.strip_suffix("');"))
                .ok_or(ErroLinkDoHistorico::LinkDoHistoricoForaDaFormaObservada)?;
            achados.push(url.to_owned());
        }
    }
    Ok(achados)
}

/// Forma `UnB`: a linha da função seguida da linha que atribui o `src`.
fn do_script(html: &str) -> Result<Vec<String>, ErroLinkDoHistorico> {
    let linhas: Vec<&str> = html.lines().map(str::trim).collect();
    let mut achados = Vec::new();
    for (i, linha) in linhas.iter().enumerate() {
        if !linha.contains("function consultarAndamento") {
            continue;
        }
        let url = (*linha == FUNCAO)
            .then(|| linhas.get(i + 1))
            .flatten()
            .and_then(|seguinte| seguinte.strip_prefix(SRC))
            .and_then(|resto| resto.strip_suffix("';"))
            .ok_or(ErroLinkDoHistorico::LinkDoHistoricoForaDaFormaObservada)?;
        achados.push(url.to_owned());
    }
    Ok(achados)
}
