//! RED/GREEN de `listar-processos [--pagina N]`: sequência de requisições,
//! orçamento e categoria observada; a sessão fica na D, sem restaurar.
use super::tests::{EnvioRoteirizado, TelaD, TelaR, cookie, instalacao, tela_atual};
use super::*;
use crate::caixa_detalhada::LeituraDetalhada;
use crate::dominio::Categoria;
use crate::transporte::sessao::CookieDaSessao;

/// R da `UnB`/PEN1 em miniatura: Recebidos P1, P2 (completos); Gerados com
/// 3 registros, página 1 mostra P3.
fn r() -> TelaR {
    TelaR::nova((2, vec![1, 2]), (3, vec![3]))
}

fn listar(
    tela: String,
    pagina: Option<usize>,
    respostas: Vec<(i32, String)>,
) -> (Result<LeituraDetalhada, ErroVisaoDoControle>, Vec<String>) {
    let (_pasta, arquivo) = cookie();
    let mut envio = EnvioRoteirizado::novo(respostas);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(tela),
        Some("TESTE"),
        pagina,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(envio.respostas.is_empty(), "respostas não consumidas");
    (resultado, envio.registro)
}

#[test]
fn sessao_em_r_troca_para_d_le_e_fica_na_d() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (resultado, registro) = listar(r().html(), None, vec![(200, d.html())]);
    assert_eq!(registro, ["POST visao=D recebidos=0 gerados=0"]);
    let leitura = resultado.unwrap();
    assert_eq!(leitura.unidade, "TESTE");
    assert_eq!(leitura.total_declarado, 5);
    assert_eq!(
        leitura.faixa_observada,
        crate::paginas::FaixaObservada::Lidas { inicio: 1, fim: 5 }
    );
    let categorias: Vec<_> = leitura.processos.iter().map(|p| p.categoria).collect();
    assert_eq!(
        categorias,
        [
            Some(Categoria::Recebidos),
            Some(Categoria::Recebidos),
            Some(Categoria::Gerados),
            None,
            None
        ]
    );
    let json = serde_json::to_value(&leitura.processos[2]).unwrap();
    assert_eq!(json["categoria"], "gerados");
    assert_eq!(json["colunas"]["atribuicao"], "pessoa3");
    assert_eq!(json["colunas"]["tipo"], "Tipo sintético");
    assert_eq!(
        json["colunas"]["interessados"],
        serde_json::json!(["Interessado A", "Interessado B"])
    );
    assert!(serde_json::to_value(&leitura.processos[3]).unwrap()["categoria"].is_null());
}

#[test]
fn sessao_ja_em_d_le_sem_post() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (resultado, registro) = listar(d.html(), None, vec![]);
    assert_eq!(registro.len(), 0);
    let leitura = resultado.unwrap();
    assert!(leitura.processos.iter().all(|p| p.categoria.is_none()));
    assert_eq!(leitura.processos.len(), 5);
}

/// Decisão do proprietário (02/11/2026): a D fica lembrando a página N.
#[test]
fn pagina_n_a_partir_da_r_fica_na_pagina_n_da_d() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(
        r().html(),
        Some(2),
        vec![(200, d1.html()), (200, d2.html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=D pagina=1"
        ]
    );
    let leitura = resultado.unwrap();
    assert_eq!(
        leitura.faixa_observada,
        crate::paginas::FaixaObservada::Lidas { inicio: 3, fim: 4 }
    );
    assert_eq!(leitura.processos[0].categoria, Some(Categoria::Gerados));
    assert_eq!(leitura.processos[1].categoria, None);
}

#[test]
fn pagina_encontrada_da_d_e_a_da_resposta_da_troca() {
    // A D lembrava a página 2; `--pagina 1` pede a página 1 e fica nela.
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(
        r().html(),
        Some(1),
        vec![(200, d2.html()), (200, d1.html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=D pagina=0"
        ]
    );
    assert_eq!(
        resultado.unwrap().faixa_observada,
        crate::paginas::FaixaObservada::Lidas { inicio: 1, fim: 2 }
    );
}

#[test]
fn falha_do_post_de_pagina_vindo_da_r_e_publicada_sem_post_de_volta() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d3 = TelaD::nova(5, 2, 2, vec![10]);
    let erro_500 = "<html><div id='divInfraExcecao'>Erro.</div></html>".to_owned();
    for resposta in [(500, erro_500), (200, d3.html())] {
        let (_resultado, registro) = listar(r().html(), Some(2), vec![(200, d1.html()), resposta]);
        assert_eq!(
            registro,
            [
                "POST visao=D recebidos=0 gerados=0",
                "POST visao=D pagina=1"
            ]
        );
    }
}

#[test]
fn pagina_ja_lembrada_pela_d_nao_pede_pagina() {
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(r().html(), Some(2), vec![(200, d2.html())]);
    assert_eq!(registro, ["POST visao=D recebidos=0 gerados=0"]);
    assert_eq!(
        resultado.unwrap().faixa_observada,
        crate::paginas::FaixaObservada::Lidas { inicio: 3, fim: 4 }
    );
}

#[test]
fn sessao_em_d_pagina_n_fica_na_pagina_n() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d3 = TelaD::nova(5, 2, 2, vec![10]);
    let (resultado, registro) = listar(d1.html(), Some(3), vec![(200, d3.html())]);
    assert_eq!(registro, ["POST visao=D pagina=2"]);
    assert_eq!(
        resultado.unwrap().faixa_observada,
        crate::paginas::FaixaObservada::Lidas { inicio: 5, fim: 5 }
    );
}

#[test]
fn sessao_em_d_pagina_n_com_faixa_errada_falha_sem_post_de_volta() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(d1.html(), Some(3), vec![(200, d2.html())]);
    assert_eq!(registro, ["POST visao=D pagina=2"]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::FaixaDaRespostaDiferenteDaPaginaSolicitada
    );
}

#[test]
fn pagina_fora_do_intervalo_antes_de_qualquer_post_quando_calculavel() {
    // R: 2 + 3 = 5 registros, no máximo 5 páginas.
    let (resultado, registro) = listar(r().html(), Some(6), vec![]);
    assert_eq!(registro.len(), 0);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: 6 }
    );
    // D já exibida: 5 registros em páginas de 2.
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let (resultado, registro) = listar(d1.html(), Some(4), vec![]);
    assert_eq!(registro.len(), 0);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: 4 }
    );
}

#[test]
fn pagina_fora_do_intervalo_vista_so_na_d_fica_na_d() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let (resultado, registro) = listar(r().html(), Some(4), vec![(200, d1.html())]);
    assert_eq!(registro, ["POST visao=D recebidos=0 gerados=0"]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: 4 }
    );
}

#[test]
fn cabecalho_desconhecido_na_d_falha_fechado_sem_post_de_volta() {
    let mut d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    d.cabecalho = d.cabecalho.replacen("Atribuição", "Situação", 1);
    let (resultado, registro) = listar(r().html(), None, vec![(200, d.html())]);
    assert_eq!(registro, ["POST visao=D recebidos=0 gerados=0"]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::Detalhada(
            crate::caixa_detalhada::ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido
        )
    );
}

#[test]
fn troca_sem_d_na_resposta_e_recusada() {
    let (resultado, registro) = listar(r().html(), None, vec![(200, r().html())]);
    assert_eq!(registro, ["POST visao=D recebidos=0 gerados=0"]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::VisaoNaoTrocadaAposEnvio {
            esperada: Visao::Detalhada
        }
    );
    let mut outra = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    outra.unidade = "OUTRA";
    let (resultado, _) = listar(r().html(), None, vec![(200, outra.html())]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::UnidadeAlteradaAoTrocarVisao
    );
}

#[test]
fn unidade_solicitada_diferente_falha_antes_de_qualquer_post() {
    let (_pasta, arquivo) = cookie();
    let mut envio = EnvioRoteirizado::novo(vec![]);
    let resultado = listar_na_visao_detalhada(
        &instalacao(),
        &tela_atual(r().html()),
        Some("OUTRA"),
        None,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert_eq!(envio.registro.len(), 0);
    assert!(resultado.is_err());
}

#[test]
fn d_sem_campo_de_pagina_como_a_unb_so_tem_a_pagina_um() {
    let mut d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    d.indice = None;
    let (resultado, registro) = listar(d.html(), Some(1), vec![]);
    assert_eq!(registro.len(), 0);
    assert_eq!(resultado.unwrap().processos.len(), 5);
    let (resultado, registro) = listar(d.html(), Some(2), vec![]);
    assert_eq!(registro.len(), 0);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: 2 }
    );
}

/// Golden da saída de `listar-processos` (contrato da visão detalhada).
#[test]
fn saida_da_lista_detalhada_segue_o_golden() {
    let d = TelaD::nova(5, 0, 2, vec![1, 9]);
    let (resultado, _) = listar(r().html(), None, vec![(200, d.html())]);
    let valor = crate::unidade_lista_cli::json_da_detalhada(&resultado.unwrap(), "web");
    let texto = format!("{}\n", serde_json::to_string_pretty(&valor).unwrap());
    assert_eq!(
        texto,
        include_str!("../../tests/fixtures/caixa-detalhada-v2.json")
    );
}
