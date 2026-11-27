//! Hipótese sintética de projeção das páginas processuais armazenadas.
use crate::{Estado, escapar};
use iesei::documentos_simulados::ler_processo;
use iesei::instalacao::Instalacao;
use iesei::observacao::UnidadeExibida;
use iesei::pesquisa_processo::{Criterios, consultar_pagina};
use rusqlite::Connection;
use std::fmt::Write as _;

pub(super) fn pagina(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
    deslocamento: u64,
) -> Result<Option<String>, Estado> {
    let consultar = |inicio| {
        consultar_pagina(banco, instalacao, cenario, unidade, criterios, inicio)
            .map_err(|_| Estado::DadosInsuficientes)
    };
    let Some(pagina) = consultar(deslocamento)? else {
        return Ok(None);
    };
    let primeira = consultar(0)?.ok_or(Estado::DadosInsuficientes)?;
    let tamanho = primeira.fim;
    let fim_esperado = deslocamento.saturating_add(tamanho).min(primeira.total);
    if primeira.numero != 1
        || pagina.total != primeira.total
        || pagina.fim != fim_esperado
        || u64::from(pagina.numero - 1).checked_mul(tamanho) != Some(deslocamento)
    {
        return Err(Estado::DadosInsuficientes);
    }
    let mut linhas = String::new();
    for id in &pagina.ids {
        let processo = ler_processo(banco, instalacao, cenario, id)
            .map_err(|_| Estado::DadosInsuficientes)?
            .ok_or(Estado::DadosInsuficientes)?;
        let url = escapar(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
            crate::protocolo_sintetico::INFRA_HASH
        ));
        write!(
            linhas,
            "<tr class='pesquisaTituloRegistro'><td colspan='2' class='pesquisaTituloEsquerda'>\
             <a class='arvore' href='{url}'><img class='arvore' src='svg/arvore.svg' alt='Processo'/></a>\
             <a class='protocoloNormal' href='{url}'>{}</a></td>\
             <td class='pesquisaTituloDireita'></td></tr>",
            escapar(&processo.nup),
        )
        .map_err(|_| Estado::DadosInsuficientes)?;
    }
    super::paginas::renderizar_lista(
        pagina.numero,
        pagina.inicio,
        pagina.fim,
        pagina.total,
        tamanho,
        &linhas,
    )
    .map(Some)
}
