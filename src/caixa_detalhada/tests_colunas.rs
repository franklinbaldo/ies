//! RED/GREEN das colunas configuradas da tabela D. Cabeçalhos e células
//! sintéticos na estrutura de GO/PEN1 (5.0.x, `aria-label`, `data-label`)
//! e da `UnB` (4.0.9, tooltip, sem `data-label`).
use super::*;

pub(super) fn rotulo(texto: &str) -> String {
    format!(
        "<th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>{texto}</div>\
         <div class='infraDivSetaOrdenacao'><a><img title='Ordenar' class='infraImgOrdenacao'></a></div></div></th>"
    )
}

fn icone_aria(texto: &str) -> String {
    format!(
        "<th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>\
         <img aria-label='{texto}' onmouseover=\"return infraTooltipMostrar('{texto}');\" class='infraImg'></div></div></th>"
    )
}

fn icone_tooltip(texto: &str) -> String {
    format!(
        "<th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>\
         <img onmouseover=\"return infraTooltipMostrar('{texto}');\" src='svg/x.svg' class='infraImg' /></div></div></th>"
    )
}

/// Tabela D de uma linha (P7) com os cabeçalhos e células dados, depois de
/// seleção e ícones (sem rótulo) e antes da coluna final de ações.
pub(super) fn tabela(cabecalhos: &[String], celulas: &[&str]) -> String {
    let cabecalho: String = cabecalhos.concat();
    let linha: String = celulas.concat();
    format!(
        "<html><body><a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <table id='tblProcessosDetalhado' class='infraTable tabelaControle'>\
         <caption>Lista de Processos (1 registros):</caption>\
         <tr><th class='infraTh'><a id='lnkInfraCheck'><img title='Selecionar Tudo'></a></th>\
         <th class='infraTh'>&nbsp;</th>{}{cabecalho}<th class='infraTh'>&nbsp;</th></tr>\
         <tr id='P7'><td><input type='checkbox'></td><td data-label='&nbsp;'><a><img class='imagemStatus'></a></td>\
         <td data-label='Processo'><a class='processoVisualizado' \
         href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7'>000007<wbr>/2026</a></td>\
         {linha}<td data-label='&nbsp;'></td></tr></table></body></html>",
        rotulo("Processo")
    )
}

pub(super) fn ler(html: &str) -> Result<LeituraDetalhada, ErroDetalhada> {
    ler_paginas(&[html.to_owned()], Some("PPI"))
}

#[test]
fn colunas_de_go_saem_com_chaves_estaveis_e_texto_colapsado() {
    let html = tabela(
        &[
            rotulo("Atribuição"),
            rotulo("Tipo"),
            "<th class='infraTh'>Interessados</th>".to_owned(),
            icone_aria("Controle de Prazo"),
            rotulo("Última&nbsp;Mov."),
        ],
        &[
            "<td data-label='Atribuição'><a class='ancoraSigla'>pessoa7</a></td>",
            "<td data-label='Tipo'>\n  Tipo   do&nbsp;processo </td>",
            "<td data-label='Interessados'><div>\
             <div class='divItemCelula'><div class='divIconeItemCelula'><img></div><div class='divRotuloItemCelula'>Pessoa A</div></div>\
             <div class='divItemCelula'><div class='divIconeItemCelula'><img></div><div class='divRotuloItemCelula'> Pessoa  B </div></div>\
             </div></td>",
            "<td></td>",
            "<td data-label='Última Mov.'>3</td>",
        ],
    );
    let leitura = ler(&html).unwrap();
    let processo = &leitura.processos[0];
    assert_eq!(processo.numero, "000007/2026");
    let json = serde_json::to_value(processo).unwrap();
    assert!(json.get("categoria").is_none());
    let colunas = json["colunas"].as_object().unwrap();
    let chaves: Vec<_> = colunas.keys().map(String::as_str).collect();
    let mut esperadas = vec!["atribuicao", "tipo", "interessados", "ultima_movimentacao"];
    esperadas.sort_unstable();
    let mut obtidas = chaves.clone();
    obtidas.sort_unstable();
    assert_eq!(obtidas, esperadas);
    assert_eq!(colunas["atribuicao"], "pessoa7");
    assert_eq!(colunas["tipo"], "Tipo do processo");
    assert_eq!(
        colunas["interessados"],
        serde_json::json!(["Pessoa A", "Pessoa B"])
    );
    assert!(colunas.get("controle_de_prazo").is_none());
    assert_eq!(colunas["ultima_movimentacao"], "3");
    assert_eq!(
        leitura.colunas_ausentes(),
        [
            "anotacao",
            "especificacao",
            "prioridade",
            "observacao",
            "marcadores"
        ]
    );
}

#[test]
fn icone_da_unb_e_rotulado_pelo_tooltip_e_celulas_sem_data_label() {
    let html = tabela(
        &[
            rotulo("Anotação"),
            icone_tooltip("Para Devolver"),
            icone_tooltip("Aguardando Retorno"),
        ],
        &["<td><span>nota</span></td>", "<td>-9</td>", "<td></td>"],
    );
    let leitura = ler(&html).unwrap();
    assert_eq!(
        leitura.colunas,
        [
            ColunaDoDetalhe::Anotacao,
            ColunaDoDetalhe::RetornoDevolver,
            ColunaDoDetalhe::RetornoAguardando
        ]
    );
    let json = serde_json::to_value(&leitura.processos[0]).unwrap();
    assert_eq!(json["colunas"]["anotacao"], "nota");
    assert_eq!(json["colunas"]["para_devolver"], "-9");
    assert!(json["colunas"].get("aguardando_retorno").is_none());
}

#[test]
fn as_nove_colunas_de_texto_presentes_nao_deixam_ausentes() {
    let rotulos = [
        "Atribuição",
        "Anotação",
        "Tipo",
        "Especificação",
        "Prioridade",
        "Interessados",
        "Observação",
        "Última Mov.",
        "Marcadores",
    ];
    let cabecalhos: Vec<_> = rotulos.iter().map(|r| rotulo(r)).collect();
    let celulas = vec!["<td></td>"; rotulos.len()];
    let leitura = ler(&tabela(&cabecalhos, &celulas)).unwrap();
    assert_eq!(leitura.colunas_ausentes().len(), 0);
    let json = serde_json::to_value(&leitura.processos[0]).unwrap();
    assert_eq!(json["colunas"]["interessados"], serde_json::json!([]));
    // Marcadores é lista (um processo pode ter vários): célula vazia é `[]`.
    assert_eq!(json["colunas"]["marcadores"], serde_json::json!([]));
}

#[test]
fn cabecalho_desconhecido_ou_repetido_falha_fechado() {
    assert_eq!(
        ler(&tabela(&[rotulo("Situação")], &["<td>x</td>"])),
        Err(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido)
    );
    assert_eq!(
        ler(&tabela(&[icone_aria("Ícone novo")], &["<td></td>"])),
        Err(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido)
    );
    assert_eq!(
        ler(&tabela(
            &["<th><div class='infraDivRotuloOrdenacao'></div></th>".to_owned()],
            &["<td></td>"]
        )),
        Err(ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido)
    );
    assert_eq!(
        ler(&tabela(
            &[rotulo("Tipo"), rotulo("Tipo")],
            &["<td>a</td>", "<td>b</td>"]
        )),
        Err(ErroDetalhada::ColunaDaVisaoDetalhadaRepetida)
    );
}

#[test]
fn celula_fora_do_lugar_ou_com_itens_inesperados_e_recusada() {
    assert_eq!(
        ler(&tabela(
            &[rotulo("Tipo")],
            &["<td data-label='Especificação'>x</td>"]
        )),
        Err(ErroDetalhada::RotuloDaCelulaDiferenteDoCabecalho)
    );
    assert_eq!(
        ler(&tabela(
            &["<th class='infraTh'>Interessados</th>".to_owned()],
            &["<td data-label='Interessados'>Pessoa A, Pessoa B</td>"]
        )),
        Err(ErroDetalhada::ItensDaCelulaNaoSeparados)
    );
    assert_eq!(
        ler(&tabela(
            &[rotulo("Marcadores")],
            &["<td><div class='divItemCelula'><div class='divRotuloItemCelula'>M</div></div></td>"]
        )),
        Err(ErroDetalhada::TextoForaDosMarcadoresNaCelula)
    );
    assert_eq!(
        ler(&tabela(
            &[rotulo("Observação")],
            &["<td><div class='divItemCelula'><div class='divRotuloItemCelula'>M</div></div></td>"]
        )),
        Err(ErroDetalhada::ItensNaoPrevistosNaCelulaDetalhada)
    );
}
