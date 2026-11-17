//! RED/GREEN da unidade vazia em GO (Refs #57). As telas gravadas
//! `unidade-troca-e-volta-go#6` e `bloco-devolver-go#38` (unidade sem
//! processos) servem a visão R sem `<table>`: `div{T}AreaTabela`,
//! `div{T}AreaPaginacaoSuperior` e `div{T}AreaPaginacaoInferior` vazias,
//! `hdn{T}NroItens` 0 e `hdn{T}PaginaAtual` 0, sem mensagem de lista vazia.
//! Só esse conjunto comprova a lista vazia; tabela ausente sem ele continua
//! `tabela_ausente`.
use super::*;
use crate::pagina_do_controle::{ErroPaginaDoControle, planejar};
use crate::tela_html::HtmlServido;

/// Os campos da categoria como a tela de GO os publica (ordem e ids).
struct Vazia<'a> {
    area: &'a str,
    nro_itens: &'a str,
    pagina_atual: &'a str,
    paginacao: &'a str,
}

const GRAVADA: Vazia<'static> = Vazia {
    area: "\n\n",
    nro_itens: "<input type=\"hidden\" id=\"hdn{T}NroItens\" name=\"hdn{T}NroItens\" value=\"0\"/>",
    pagina_atual: "0",
    paginacao: "\n",
};

fn categoria(nome: &str, v: &Vazia<'_>) -> String {
    format!(
        "<div id=\"div{nome}\" class=\"ml-0 pl-0 d-none d-md-block col-12 col-md-6\">\
         <div id=\"div{nome}AreaPaginacaoSuperior\" class=\"infraAreaPaginacao\">{p}</div>\
         <div id=\"div{nome}AreaTabela\" class=\"infraAreaTabela\" >{area}</div>\
         {nro}\
         <input type=\"hidden\" id=\"hdn{nome}ItemId\" name=\"hdn{nome}ItemId\" value=\"\"/>\
         <div id=\"div{nome}AreaPaginacaoInferior\" class=\"infraAreaPaginacao\">{p}</div>\
         <input type=\"hidden\" id=\"hdn{nome}PaginaAtual\" name=\"hdn{nome}PaginaAtual\" value=\"{atual}\"/>\
         <input type=\"hidden\" id=\"hdn{nome}HashCriterios\" name=\"hdn{nome}HashCriterios\" value=\"3d0b8e60\"/>\
         </div>",
        p = v.paginacao,
        area = v.area,
        nro = v.nro_itens.replace("{T}", nome),
        atual = v.pagina_atual,
    )
}

fn tela(visao: &str, unidade: &str, recebidos: &Vazia<'_>, gerados: &Vazia<'_>) -> String {
    format!(
        "<a id=\"lnkInfraUnidade\" href=\"#\" title=\"GIRÊNCUO DI CANTOBULUDODI\">{unidade}</a>\
         <div id=\"divTabelaProcesso\">{}{}</div>\
         <input type=\"hidden\" id=\"hdnTipoVisualizacao\" name=\"hdnTipoVisualizacao\" value=\"{visao}\" />\
         <input type=\"hidden\" id=\"hdnExibirRecebidos\" name=\"hdnExibirRecebidos\" value=\"false\" />\
         <input type=\"hidden\" id=\"hdnIdMarcador18141\" name=\"hdnIdMarcador18141\" value=\"\" />",
        categoria("Recebidos", recebidos),
        categoria("Gerados", gerados),
    )
}

fn gravada() -> String {
    tela("R", "ABC/GECONT-18141", &GRAVADA, &GRAVADA)
}

#[test]
fn tela_gravada_da_unidade_vazia_publica_total_zero_sem_faixa() {
    let leitura = juntar(
        &[HtmlServido::de_teste(&gravada())],
        Some("ABC/GECONT-18141"),
    )
    .unwrap();
    for (cat, tabela, cobertura) in [
        (
            Categoria::Recebidos,
            &leitura.caixa.recebidos,
            &leitura.recebidos,
        ),
        (Categoria::Gerados, &leitura.caixa.gerados, &leitura.gerados),
    ] {
        assert_eq!(tabela.total_declarado, 0, "{cat:?}");
        assert!(tabela.processos.is_empty(), "{cat:?}");
        assert_eq!(
            cobertura.faixa_observada(),
            Ok(FaixaObservada::SemLinhas),
            "{cat:?}"
        );
        assert!(cobertura.completa(tabela), "{cat:?}");
        assert!(conferir_faixa_publicavel(cat, cobertura).is_ok());
        assert_eq!(
            faixa_na_pagina(&HtmlServido::de_teste(&gravada()), cat),
            Ok(None)
        );
    }
    assert_eq!(leitura.unidade.as_deref(), Some("ABC/GECONT-18141"));
}

#[test]
fn pagina_2_da_unidade_vazia_fica_fora_do_intervalo() {
    assert_eq!(
        planejar(&HtmlServido::de_teste(&gravada()), Categoria::Gerados, 2),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo {
            categoria: Categoria::Gerados,
            solicitada: 2,
            paginas: 1,
        })
    );
}

fn ausente(html: &str, categoria: Categoria) {
    assert_eq!(
        juntar(&[HtmlServido::de_teste(html)], None).err(),
        Some(ErroCaixa::TabelaAusente { categoria }),
        "{html}"
    );
}

#[test]
fn tabela_ausente_sem_a_prova_estrutural_continua_recusada() {
    let sem_nro = Vazia {
        nro_itens: "",
        ..GRAVADA
    };
    let nro_nao_zero = Vazia {
        nro_itens: "<input type=\"hidden\" id=\"hdn{T}NroItens\" value=\"10\"/>",
        ..GRAVADA
    };
    let nro_vazio = Vazia {
        nro_itens: "<input type=\"hidden\" id=\"hdn{T}NroItens\" value=\"\"/>",
        ..GRAVADA
    };
    let nro_repetido = Vazia {
        nro_itens: "<input id=\"hdn{T}NroItens\" value=\"0\"/><input id=\"hdn{T}NroItens\" value=\"0\"/>",
        ..GRAVADA
    };
    let area_com_conteudo = Vazia {
        area: "<div>Carregando</div>",
        ..GRAVADA
    };
    let area_com_texto = Vazia {
        area: "Aguarde",
        ..GRAVADA
    };
    let outra_pagina = Vazia {
        pagina_atual: "3",
        ..GRAVADA
    };
    let com_paginador = Vazia {
        paginacao: "<a id=\"lnkGeradosProximaPaginaSuperior\" href=\"#\">&gt;</a>",
        ..GRAVADA
    };
    for gerados in [
        &sem_nro,
        &nro_nao_zero,
        &nro_vazio,
        &nro_repetido,
        &area_com_conteudo,
        &area_com_texto,
        &outra_pagina,
        &com_paginador,
    ] {
        // Recebidos com tabela: a recusa observada é só a de Gerados.
        let html = tela("R", "ABC/GECONT-18141", &GRAVADA, gerados).replacen(
            "<div id=\"divRecebidosAreaTabela\" class=\"infraAreaTabela\" >\n\n",
            "<div id=\"divRecebidosAreaTabela\" class=\"infraAreaTabela\" >\
             <table id=\"tblProcessosRecebidos\"><caption>0 registros:</caption></table>",
            1,
        );
        assert!(html.contains("tblProcessosRecebidos"));
        ausente(&html, Categoria::Gerados);
    }
}

#[test]
fn sem_visao_r_ou_sem_unidade_nao_ha_prova_de_lista_vazia() {
    ausente(
        &tela("", "ABC/GECONT-18141", &GRAVADA, &GRAVADA),
        Categoria::Recebidos,
    );
    ausente(&tela("R", "", &GRAVADA, &GRAVADA), Categoria::Recebidos);
    let sem_area = gravada().replace("divRecebidosAreaTabela", "divOutraArea");
    ausente(&sem_area, Categoria::Recebidos);
}
