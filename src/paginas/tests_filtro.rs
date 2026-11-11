//! RED/GREEN do replay do acervo (Refs #57). (1) Filtro de marcador
//! ativo: o POST de "Listar processos com o marcador" das sete instalações
//! (`marcador-listar-processos-*`, passo 2) serve a visão R com
//! `hdnIdMarcador{id da unidade}` preenchido e só as linhas do marcador;
//! as 715 telas GET do acervo trazem o campo vazio. (2) Legenda no
//! singular: as mesmas telas publicam `1 registro:` e
//! `Processos recebidos (1 registro):`, que a visão D já lê.
use super::*;

fn linha(id: u32) -> String {
    format!("<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>")
}

fn tabela(nome: &str, legenda: &str, ids: &[u32]) -> String {
    format!(
        "<table id='tblProcessos{nome}'><caption>{legenda}</caption>{}</table>",
        ids.iter().map(|id| linha(*id)).collect::<String>()
    )
}

fn pagina(marcador: &str, recebidos: &str, gerados: &str) -> String {
    format!(
        "<a id='lnkInfraUnidade'>TESTE</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <input id='hdnRecebidosHashCriterios' value='aaaa'>\
         <input id='hdnGeradosHashCriterios' value='bbbb'>\
         <input type='hidden' id='hdnIdComMarcador' value='' />\
         <input type='hidden' id='hdnIdMarcador110000001' name='hdnIdMarcador110000001' value='{marcador}' />\
         {recebidos}{gerados}"
    )
}

#[test]
fn filtro_de_marcador_ativo_e_recusado_antes_de_publicar() {
    let html = pagina(
        "3",
        &tabela("Recebidos", "6 registros:", &[1, 2, 3, 4, 5, 6]),
        &tabela("Gerados", "0 registros:", &[]),
    );
    let erro = juntar(&[html], None).err();
    assert_eq!(erro, Some(ErroCaixa::FiltroDeMarcadorAtivoNoControle));
}

#[test]
fn campo_do_marcador_vazio_nao_e_filtro() {
    let html = pagina(
        "",
        &tabela("Recebidos", "2 registros:", &[1, 2]),
        &tabela("Gerados", "0 registros:", &[]),
    );
    assert!(juntar(&[html], None).is_ok());
}

#[test]
fn legenda_no_singular_comprova_a_pagina_unica() {
    for legenda in ["1 registro:", "Processos recebidos (1 registro):"] {
        let html = pagina(
            "",
            &tabela("Recebidos", legenda, &[7]),
            &tabela("Gerados", "0 registros:", &[]),
        );
        let leitura = juntar(&[html], None).unwrap();
        assert_eq!(
            leitura.recebidos.faixa_observada(),
            Ok(FaixaObservada::Lidas { inicio: 1, fim: 1 }),
            "{legenda}"
        );
        assert!(
            conferir_faixa_publicavel(Categoria::Recebidos, &leitura.recebidos).is_ok(),
            "{legenda}"
        );
    }
}
