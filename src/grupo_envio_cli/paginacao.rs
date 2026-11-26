//! Paginação Infra: formulário observado, sem executar JavaScript.
//! Hipótese para grupos: o mesmo protocolo visto em blocos GO e no Todos.
//! A lista é completa ou falha; não há restauração da página nesta fatia.
use super::{ErroGrupo, Linha, ler_pagina};
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
use std::collections::HashSet;

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

fn acumular(
    linhas: &mut Vec<Linha>,
    ids: &mut HashSet<String>,
    total: usize,
    legenda: Legenda,
    pagina: Vec<Linha>,
) -> Result<(), ErroGrupo> {
    if legenda.total != total {
        return Err(ErroGrupo::ListaIncompleta);
    }
    if let Some(faixa) = legenda.faixa {
        if faixa.inicio != linhas.len() + 1
            || faixa.fim != linhas.len() + pagina.len()
            || faixa.fim > total
        {
            return Err(ErroGrupo::ListaIncompleta);
        }
    } else if !linhas.is_empty() || pagina.len() != total {
        return Err(ErroGrupo::ListaIncompleta);
    }
    for linha in pagina {
        if !ids.insert(linha.item.id.clone()) {
            return Err(ErroGrupo::IdentidadeAmbigua);
        }
        linhas.push(linha);
    }
    Ok(())
}

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

pub(super) fn ler_todas<V: EnvioDoControle>(
    mut html: HtmlServido,
    mut url: String,
    codificacao: Option<CodificacaoDaTela>,
    sessao: &ReferenciasDaSessao,
    instalacao: &Instalacao,
    envio: &mut V,
) -> anyhow::Result<Vec<Linha>> {
    let (legenda, pagina) = ler_pagina(&html)?;
    let total = legenda.total;
    let mut linhas = Vec::new();
    let mut ids = HashSet::new();
    acumular(&mut linhas, &mut ids, total, legenda, pagina)?;
    for indice in 1..200 {
        if linhas.len() == total {
            return Ok(linhas);
        }
        let indice = indice.to_string();
        let campos = campos_da_pagina(&html, &indice)?;
        let tela = TelaServida {
            url: &url,
            html: &html,
            codificacao: codificacao.ok_or(ErroGrupo::ListaIncompleta)?,
        };
        let post = derivar_submissao(&PAGINAR, instalacao, &tela, None, &campos)?;
        post.url().clone_into(&mut url);
        let RespostaDoEnvio::Html(proxima) =
            enviar_submissao_via(post, sessao.cookie(), |post, cookie| envio.post(post, cookie))?
        else {
            return Err(ErroGrupo::ListaIncompleta.into());
        };
        html = proxima;
        let (legenda, pagina) = ler_pagina(&html)?;
        acumular(&mut linhas, &mut ids, total, legenda, pagina)?;
    }
    if linhas.len() == total {
        Ok(linhas)
    } else {
        Err(ErroGrupo::ListaIncompleta.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pagina(total: usize, inicio: usize, fim: usize, id: &str) -> (Legenda, Vec<Linha>) {
        let html = format!(
            "<form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio ({total} registros - {inicio} a {fim}):</caption><tr><th></th><th>Nome</th></tr><tr><td><input type='checkbox' value='{id}'></td><td>Mesmo</td></tr></table></form>"
        );
        ler_pagina(&HtmlServido::de_teste(&html)).unwrap()
    }

    #[test]
    fn paginas_contiguas_preservam_homonimos() {
        let mut linhas = Vec::new();
        let mut ids = HashSet::new();
        for (indice, id) in [(1, "004"), (2, "6")] {
            let (legenda, pagina) = pagina(2, indice, indice, id);
            acumular(&mut linhas, &mut ids, 2, legenda, pagina).unwrap();
        }
        let ids: Vec<_> = linhas.iter().map(|l| l.item.id.as_str()).collect();
        assert_eq!(ids, ["004", "6"]);
    }

    #[test]
    fn segunda_pagina_nao_pode_repetir_id_faixa_ou_mudar_total() {
        for (total, indice, id) in [(2, 2, "4"), (2, 1, "6"), (3, 2, "6")] {
            let mut linhas = Vec::new();
            let mut ids = HashSet::new();
            let (legenda, primeira) = pagina(2, 1, 1, "4");
            acumular(&mut linhas, &mut ids, 2, legenda, primeira).unwrap();
            let (legenda, segunda) = pagina(total, indice, indice, id);
            assert!(acumular(&mut linhas, &mut ids, 2, legenda, segunda).is_err());
        }
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
