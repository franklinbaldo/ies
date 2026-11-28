//! Resolução de membros por autocomplete declarado na tela.
use super::Membro;
use crate::acao_do_sei::AcaoAjaxDoSei;
use crate::formulario_web::{ConsultaAjax, EfeitoAvaliado, TelaServida, derivar_consulta_ajax};
use crate::instalacao::Instalacao;
use crate::processo_enviar::{ler_resposta_da_consulta, termo_alargado};
use crate::visao_do_controle::EnvioDoControle;
use std::collections::HashSet;

const CONSULTA: ConsultaAjax = ConsultaAjax {
    acao_ajax: AcaoAjaxDoSei::UnidadeAutoCompletarEnvioProcesso,
    campos: &["palavras_pesquisa"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos grupos_envio.py: _unidade_achada; replay original pendente",
    },
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
enum ErroMembro {
    AutocompleteNaoDeclarado,
    UnidadeNaoOferecida,
    UnidadeAmbigua,
}

fn url_declarada(html: &crate::tela_html::HtmlServido) -> Result<String, ErroMembro> {
    let dom = crate::tela_html::arvore(html);
    let mut urls = Vec::new();
    const PREFIXO: &str = "new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','";
    for script in dom.elementos("script") {
        let texto = script.texto();
        for resto in texto.split(PREFIXO).skip(1) {
            let (url, _) = resto
                .split_once("');")
                .ok_or(ErroMembro::AutocompleteNaoDeclarado)?;
            urls.push(url.to_owned());
        }
    }
    match urls.as_slice() {
        [url] => Ok(url.clone()),
        _ => Err(ErroMembro::AutocompleteNaoDeclarado),
    }
}

fn chave(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn escolher(termo: &str, itens: &[(String, String)], alargado: bool) -> Result<Membro, ErroMembro> {
    let termo = chave(termo);
    let exatos: Vec<_> = itens
        .iter()
        .filter(|(_, nome)| {
            chave(nome) == termo
                || nome.split_once(" - ").is_some_and(|(sigla, descricao)| {
                    chave(sigla) == termo || chave(descricao) == termo
                })
        })
        .collect();
    let (id, nome) = match exatos.as_slice() {
        [item] => *item,
        [] => match itens {
            [item] if !alargado => item,
            [] => return Err(ErroMembro::UnidadeNaoOferecida),
            _ if alargado => return Err(ErroMembro::UnidadeNaoOferecida),
            _ => return Err(ErroMembro::UnidadeAmbigua),
        },
        _ => return Err(ErroMembro::UnidadeAmbigua),
    };
    Ok(Membro {
        id: id.clone(),
        nome: nome.clone(),
    })
}

/// Consultas AJAX de leitura não são o POST mutador. Resolver todos antes
/// de preparar a escrita; deduplicar pela identidade lexical, não pelo nome.
pub(super) fn resolver<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    cookie: &str,
    pedidos: &[String],
    envio: &mut V,
) -> anyhow::Result<Vec<Membro>> {
    if pedidos.is_empty() {
        return Ok(Vec::new());
    }
    let url = url_declarada(tela.html)?;
    let mut membros = Vec::new();
    let mut ids = HashSet::new();
    for pedido in pedidos {
        let termo = pedido.trim();
        let mut consultar = |termo: &str| -> anyhow::Result<Vec<(String, String)>> {
            let post = derivar_consulta_ajax(
                &CONSULTA,
                instalacao,
                tela,
                &url,
                &[("palavras_pesquisa", termo)],
            )?;
            Ok(ler_resposta_da_consulta(envio.post(post, cookie)?)?)
        };
        let itens = consultar(termo)?;
        let membro = if itens.is_empty() {
            if let Some(alargado) = termo_alargado(termo) {
                escolher(termo, &consultar(&alargado)?, true)?
            } else {
                escolher(termo, &itens, false)?
            }
        } else {
            escolher(termo, &itens, false)?
        };
        if ids.insert(membro.id.clone()) {
            membros.push(membro);
        }
    }
    Ok(membros)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: &str, nome: &str) -> (String, String) {
        (id.to_owned(), nome.to_owned())
    }
    #[test]
    fn sigla_nome_e_rotulo_conservam_o_id() {
        let itens = [item("001", "X - Unidade"), item("1", "Y - Outra")];
        for termo in ["X", "Unidade", "X - Unidade"] {
            assert_eq!(escolher(termo, &itens, false).unwrap().id, "001");
        }
    }
    #[test]
    fn homonimos_nao_decidem() {
        let itens = [item("001", "X - Unidade"), item("1", "Y - Unidade")];
        assert!(matches!(
            escolher("Unidade", &itens, false),
            Err(ErroMembro::UnidadeAmbigua)
        ));
    }
    #[test]
    fn alargamento_nao_aceita_o_unico_item_incorreto() {
        let itens = [item("1", "DEF/GECONT - Outra")];
        assert!(matches!(
            escolher("ABC/GECONT-18141", &itens, true),
            Err(ErroMembro::UnidadeNaoOferecida)
        ));
        assert_eq!(escolher("GECONT", &itens, false).unwrap().id, "1");
    }
}
