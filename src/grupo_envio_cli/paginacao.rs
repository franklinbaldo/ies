//! Paginação Infra: formulário observado, sem executar JavaScript.
//! Hipótese para grupos: o mesmo protocolo visto em blocos GO e no Todos.
//! Sem pedido explícito, não há POST; não há varredura nem restauração.
use super::{ErroGrupo, Item, Linha, ler_pagina};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::legenda::Legenda;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioDoControle;

const PAGINAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::GrupoUnidadeListar,
    formulario: "frmGrupoUnidadeLista",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &["selInfraPaginacaoSuperior", "selInfraPaginacaoInferior"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

fn campos_da_pagina<'a>(
    html: &HtmlServido,
    indice: &'a str,
) -> anyhow::Result<Vec<(&'static str, &'a str)>> {
    let dom = arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, PAGINAR.formulario)?;
    let mut campos = vec![("hdnInfraPaginaAtual", indice)];
    for nome in PAGINAR.campos_usuario {
        let seletores: Vec<_> = form
            .elementos("select")
            .filter(|n| n.atributo("name") == Some(*nome))
            .collect();
        match seletores.as_slice() {
            [] => {}
            [select]
                if select
                    .elementos("option")
                    .filter(|n| n.atributo("value") == Some(indice))
                    .count()
                    == 1 =>
            {
                campos.push((*nome, indice));
            }
            _ => return Err(ErroGrupo::ListaIncompleta.into()),
        }
    }
    if campos.len() == 1 {
        return Err(ErroGrupo::ListaIncompleta.into());
    }
    Ok(campos)
}

// Índice publicado é zero-based. Na última página, a quantidade de
// linhas pode ser menor que o tamanho nominal.
fn geometria(html: &HtmlServido, legenda: &Legenda) -> anyhow::Result<(usize, usize)> {
    let Some(faixa) = &legenda.faixa else {
        return Ok((0, legenda.total));
    };
    let dom = arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, PAGINAR.formulario)?;
    let indices: Vec<_> = form
        .elementos("input")
        .filter(|n| n.atributo("name") == Some("hdnInfraPaginaAtual"))
        .collect();
    let [campo] = indices.as_slice() else {
        return Err(ErroGrupo::ListaIncompleta.into());
    };
    let indice: usize = campo
        .atributo("value")
        .ok_or(ErroGrupo::ListaIncompleta)?
        .parse()?;
    let tamanho = if indice == 0 {
        faixa.fim
    } else {
        (faixa.inicio - 1) / indice
    };
    if tamanho == 0
        || indice.checked_mul(tamanho) != Some(faixa.inicio - 1)
        || faixa.fim != faixa.inicio.saturating_add(tamanho - 1).min(legenda.total)
    {
        return Err(ErroGrupo::ListaIncompleta.into());
    }
    Ok((indice, tamanho))
}

pub(super) fn ler_observada<V: EnvioDoControle>(
    html: HtmlServido,
    url: String,
    codificacao: Option<CodificacaoDaTela>,
    sessao: &ReferenciasDaSessao,
    instalacao: &Instalacao,
    pagina: Option<u32>,
    envio: &mut V,
) -> anyhow::Result<(String, Legenda, Vec<Linha>)> {
    let (legenda, linhas) = ler_pagina(&html)?;
    let Some(pagina) = pagina else {
        return Ok((url, legenda, linhas));
    };
    let destino = usize::try_from(pagina.checked_sub(1).ok_or(ErroGrupo::PaginaNaoOferecida)?)?;
    let (atual, tamanho) = geometria(&html, &legenda)?;
    if destino == atual {
        return Ok((url, legenda, linhas));
    }
    let inicio = destino
        .checked_mul(tamanho)
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n <= legenda.total && tamanho > 0)
        .ok_or(ErroGrupo::PaginaNaoOferecida)?;
    let indice = destino.to_string();
    let campos = campos_da_pagina(&html, &indice)?;
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao: codificacao.ok_or(ErroGrupo::ListaIncompleta)?,
    };
    let post = derivar_submissao(&PAGINAR, instalacao, &tela, None, &campos)?;
    let url = post.url().to_owned();
    let RespostaDoEnvio::Html(proxima) =
        enviar_submissao_via(post, sessao.cookie(), |post, cookie| envio.post(post, cookie))?
    else {
        return Err(ErroGrupo::ListaIncompleta.into());
    };
    let (nova, linhas) = ler_pagina(&proxima)?;
    let fim = inicio.saturating_add(tamanho - 1).min(legenda.total);
    if nova.total != legenda.total
        || nova.faixa.as_ref().map(|f| (f.inicio, f.fim)) != Some((inicio, fim))
        || geometria(&proxima, &nova)? != (destino, tamanho)
    {
        return Err(ErroGrupo::ListaIncompleta.into());
    }
    Ok((url, nova, linhas))
}

#[derive(serde::Serialize)]
struct Listagem {
    unidade: String,
    origem: &'static str,
    total_declarado: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    faixa_observada: Option<crate::legenda::Faixa>,
    itens: Vec<Item>,
}

pub(super) fn saida(unidade: String, legenda: Legenda, itens: Vec<Item>) -> serde_json::Value {
    let faixa = legenda.faixa.or_else(|| {
        (legenda.total > 0).then_some(crate::legenda::Faixa {
            inicio: 1,
            fim: legenda.total,
        })
    });
    serde_json::json!(Listagem {
        unidade,
        origem: "web",
        total_declarado: legenda.total,
        faixa_observada: faixa,
        itens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometria_aceita_pagina_lembrada_inclusive_ultima_curta() {
        for (indice, inicio, fim) in [(0, 1, 50), (1, 51, 100), (2, 101, 103)] {
            let html = HtmlServido::de_teste(&format!(
                "<form id='frmGrupoUnidadeLista'><input name='hdnInfraPaginaAtual' value='{indice}'></form>"
            ));
            let legenda = Legenda {
                total: 103,
                faixa: Some(crate::legenda::Faixa { inicio, fim }),
            };
            assert_eq!(geometria(&html, &legenda).unwrap(), (indice, 50));
        }
    }

    #[test]
    fn geometria_recusa_indice_incoerente() {
        let html = HtmlServido::de_teste(
            "<form id='frmGrupoUnidadeLista'><input name='hdnInfraPaginaAtual' value='1'></form>",
        );
        let legenda = Legenda {
            total: 103,
            faixa: Some(crate::legenda::Faixa { inicio: 1, fim: 50 }),
        };
        assert!(geometria(&html, &legenda).is_err());
    }

    #[test]
    fn saida_distingue_faixa_observada_de_total() {
        let valor = saida("PPI".to_owned(), Legenda {
            total: 103,
            faixa: Some(crate::legenda::Faixa { inicio: 101, fim: 103 }),
        }, Vec::new());
        assert_eq!(valor.get("total_declarado").unwrap(), 103);
        assert_eq!(valor.get("faixa_observada").unwrap(), &serde_json::json!({"inicio":101,"fim":103}));
        assert!(valor.get("total").is_none());
        let vazio = saida("PPI".to_owned(), Legenda { total: 0, faixa: None }, Vec::new());
        assert!(vazio.get("faixa_observada").is_none());
    }

    #[test]
    fn cli_pagina_explicita_deve_ser_positiva() {
        use clap::Parser;
        use crate::Cli;
        assert!(Cli::try_parse_from(["ies", "grupo-envio", "listar"]).is_ok());
        assert!(Cli::try_parse_from(["ies", "grupo-envio", "listar", "--pagina", "2"]).is_ok());
        assert!(Cli::try_parse_from(["ies", "grupo-envio", "listar", "--pagina", "0"]).is_err());
    }

    #[test]
    fn destino_de_paginacao_deve_ser_oferecido() {
        let html = HtmlServido::de_teste(
            "<form id='frmGrupoUnidadeLista'><select name='selInfraPaginacaoSuperior'><option value='0'>1</option><option value='1'>2</option></select></form>",
        );
        assert_eq!(
            campos_da_pagina(&html, "1").unwrap(),
            [
                ("hdnInfraPaginaAtual", "1"),
                ("selInfraPaginacaoSuperior", "1")
            ]
        );
        assert!(campos_da_pagina(&html, "2").is_err());
    }
}
