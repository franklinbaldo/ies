//! Leitura da tela `procedimento_configurar_detalhe` (GET, somente leitura).
//! Nada é salvo: `sbmSalvar` não é enviado por este módulo.
use super::{ColunaDoDetalhe, NivelDeDetalhe, OpcaoDoDetalhe};
use crate::erro_html::ErroHtml;
use crate::formulario_web::{ErroSubmissaoFormulario, decodificar_atributo, localizar_formulario};
use crate::html_atributos::{ErroAtributoHtml, atributo_presente, valor_do_atributo};
use std::collections::HashMap;

/// Formulário observado (o `id` é o publicado pelo SEI, apesar do nome).
pub(crate) const FORMULARIO_DO_NIVEL_DE_DETALHE: &str = "frmAcessoExternoCancelar";
pub(crate) const ACAO_DO_NIVEL_DE_DETALHE: &str = "procedimento_configurar_detalhe";

/// um erro por fato verificável da tela de configuração.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroNivelDeDetalhe {
    AcaoDoFormularioDoNivelDeDetalheDiferente,
    /// O formulário não publica nenhuma das 12 caixas do catálogo.
    /// Uma caixa do catálogo ausente não é erro: sai com `disponivel: false`.
    NenhumaOpcaoDoDetalhePublicada,
    OpcaoDoDetalheRepetida {
        coluna: ColunaDoDetalhe,
    },
    RotuloDaOpcaoDoDetalheDiferente {
        coluna: ColunaDoDetalhe,
    },
    AtributoHtml(#[from] ErroAtributoHtml),
    Html(#[from] ErroHtml),
}

type E = ErroNivelDeDetalhe;

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome).map_err(E::AtributoHtml)
}

/// Texto visível com espaços (inclusive `&nbsp;`) colapsados.
pub(crate) fn texto_colapsado(texto: &str) -> String {
    texto
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `action` com exatamente um `acao=procedimento_configurar_detalhe`.
fn conferir_acao(form: &tl::HTMLTag<'_>) -> Result<(), E> {
    let diferente = E::AcaoDoFormularioDoNivelDeDetalheDiferente;
    let bruto = atributo(form, "action")?.ok_or(diferente)?;
    let acao = decodificar_atributo(&bruto).ok_or(diferente)?;
    let (_, consulta) = acao.split_once('?').ok_or(diferente)?;
    let acoes: Vec<_> = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("acao="))
        .collect();
    if acoes == [ACAO_DO_NIVEL_DE_DETALHE] {
        Ok(())
    } else {
        Err(diferente)
    }
}

/// As 12 opções do catálogo: as que a tela publica (`disponivel`) e as
/// marcadas. A tela pode publicar um subconjunto (`UnB` 4.0.9: 11, sem
/// Prioridade); caixa `chkSin*` fora do catálogo é recusa.
pub(crate) fn ler_nivel_de_detalhe(html: &str) -> Result<NivelDeDetalhe, ErroNivelDeDetalhe> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_do_nivel_de_detalhe_nao_interpretado"))?;
    let parser = dom.parser();
    let form =
        localizar_formulario(&dom, FORMULARIO_DO_NIVEL_DE_DETALHE).map_err(|erro| match erro {
            ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela => E::Html(
                ErroHtml::repetido("formularios_do_nivel_de_detalhe_repetidos"),
            ),
            ErroSubmissaoFormulario::AtributoHtml(falha) => E::AtributoHtml(falha),
            _ => E::Html(ErroHtml::ausente("formulario_do_nivel_de_detalhe_ausente")),
        })?;
    conferir_acao(form)?;
    let mut marcadas: HashMap<ColunaDoDetalhe, bool> = HashMap::new();
    let mut rotulos: HashMap<String, String> = HashMap::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if elemento == "label" {
            if let Some(alvo) = atributo(tag, "for")? {
                rotulos.insert(alvo, texto_colapsado(&tag.inner_text(parser)));
            }
            continue;
        }
        let caixa = elemento == "input"
            && atributo(tag, "type")?.is_some_and(|t| t.eq_ignore_ascii_case("checkbox"));
        let Some(nome) = atributo(tag, "name")?.filter(|n| caixa && n.starts_with("chkSin")) else {
            continue;
        };
        let coluna = ColunaDoDetalhe::pelo_campo(&nome)
            .ok_or(ErroHtml::valor("opcao_do_nivel_de_detalhe_nao_prevista"))?;
        let marcada = atributo_presente(tag, "checked").map_err(E::AtributoHtml)?;
        if marcadas.insert(coluna, marcada).is_some() {
            return Err(E::OpcaoDoDetalheRepetida { coluna });
        }
    }
    if marcadas.is_empty() {
        return Err(E::NenhumaOpcaoDoDetalhePublicada);
    }
    let mut opcoes = Vec::with_capacity(ColunaDoDetalhe::TODAS.len());
    for coluna in ColunaDoDetalhe::TODAS {
        let Some(&marcada) = marcadas.get(&coluna) else {
            opcoes.push(OpcaoDoDetalhe {
                coluna,
                disponivel: false,
                marcada: false,
            });
            continue;
        };
        if rotulos.get(coluna.campo()).map(String::as_str) != Some(coluna.rotulo()) {
            return Err(E::RotuloDaOpcaoDoDetalheDiferente { coluna });
        }
        opcoes.push(OpcaoDoDetalhe {
            coluna,
            disponivel: true,
            marcada,
        });
    }
    Ok(NivelDeDetalhe { opcoes })
}
