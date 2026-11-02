//! RED/GREEN de `listar-processos [--pagina N]`: sequência de requisições,
//! orçamento, categoria observada e restauração da tela encontrada.
use super::tests::{EnvioRoteirizado, TelaD, TelaR, cookie, instalacao, tela_atual};
use super::*;
use crate::caixa_detalhada::LeituraDetalhada;
use crate::dominio::Categoria;

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
        &arquivo,
        &mut envio,
    );
    assert!(envio.respostas.is_empty(), "respostas não consumidas");
    (resultado, envio.registro)
}

#[test]
fn sessao_em_r_troca_para_d_le_e_restaura_r() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (resultado, registro) = listar(r().html(), None, vec![(200, d.html()), (200, r().html())]);
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=R pagina=0"
        ]
    );
    let leitura = resultado.unwrap();
    assert_eq!(leitura.unidade, "TESTE");
    assert_eq!(leitura.total_declarado, 5);
    assert_eq!(leitura.faixa_observada, Some((1, 5)));
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
fn sessao_ja_em_d_nao_troca_nem_restaura() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (resultado, registro) = listar(d.html(), None, vec![]);
    assert_eq!(registro.len(), 0);
    let leitura = resultado.unwrap();
    assert!(leitura.processos.iter().all(|p| p.categoria.is_none()));
    assert_eq!(leitura.processos.len(), 5);
}

#[test]
fn pagina_n_a_partir_da_r_custa_tres_posts() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(
        r().html(),
        Some(2),
        vec![(200, d1.html()), (200, d2.html()), (200, r().html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=D pagina=1",
            "POST visao=R pagina=1"
        ]
    );
    let leitura = resultado.unwrap();
    assert_eq!(leitura.faixa_observada, Some((3, 4)));
    assert_eq!(leitura.processos[0].categoria, Some(Categoria::Gerados));
    assert_eq!(leitura.processos[1].categoria, None);
}

#[test]
fn pagina_ja_lembrada_pela_d_nao_pede_pagina() {
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) = listar(
        r().html(),
        Some(2),
        vec![(200, d2.html()), (200, r().html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=R pagina=1"
        ]
    );
    assert_eq!(resultado.unwrap().faixa_observada, Some((3, 4)));
}

#[test]
fn sessao_em_d_pagina_n_volta_a_pagina_exibida() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d3 = TelaD::nova(5, 2, 2, vec![10]);
    let (resultado, registro) =
        listar(d1.html(), Some(3), vec![(200, d3.html()), (200, d1.html())]);
    assert_eq!(registro, ["POST visao=D pagina=2", "POST visao=D pagina=0"]);
    assert_eq!(resultado.unwrap().faixa_observada, Some((5, 5)));
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
fn pagina_fora_do_intervalo_vista_so_na_d_restaura_a_r() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let (resultado, registro) = listar(
        r().html(),
        Some(4),
        vec![(200, d1.html()), (200, r().html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=R pagina=0"
        ]
    );
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: 4 }
    );
}

#[test]
fn cabecalho_desconhecido_na_d_falha_fechado_e_restaura_a_r() {
    let mut d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    d.cabecalho = d.cabecalho.replacen("Atribuição", "Situação", 1);
    let (resultado, registro) = listar(r().html(), None, vec![(200, d.html()), (200, r().html())]);
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=R pagina=0"
        ]
    );
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::Detalhada(
            crate::caixa_detalhada::ErroDetalhada::CabecalhoNaoReconhecido
        )
    );
}

#[test]
fn restauracao_recusada_pelo_servidor_e_erro_tipado_sem_lista() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let (resultado, registro) = listar(
        r().html(),
        None,
        vec![
            (200, d.html()),
            (
                500,
                "<html><div id='divInfraExcecao'>Erro de banco.</div></html>".to_owned(),
            ),
        ],
    );
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "POST visao=R pagina=0"
        ]
    );
    let erro = resultado.unwrap_err();
    assert_eq!(erro.subcodigo(), "visao_do_controle_nao_restaurada");
    let http = erro.status_http().unwrap();
    assert_eq!(http.status, 500);
    assert_eq!(http.mensagem_do_servidor.as_deref(), Some("Erro de banco."));
    let detalhes = erro.detalhes_json().unwrap();
    assert_eq!(detalhes["visao_deixada"], "D");
    assert_eq!(detalhes["causa"], "status_http_de_erro_do_servidor");
}

#[test]
fn restauracao_que_nao_devolve_a_r_como_estava_e_recusada() {
    let d = TelaD::nova(5, 0, 10, vec![1, 2, 3, 9, 10]);
    let mut outra_pagina = r();
    outra_pagina.pagina_g = "1";
    outra_pagina.gerados.1 = 2;
    let mut outro_criterio = r();
    outro_criterio.hash_r = "9999";
    for (volta, causa) in [
        (d.html(), "visao_do_controle_nao_trocada_apos_envio"),
        (
            outra_pagina.html(),
            "paginas_da_visao_resumida_alteradas_apos_restaurar",
        ),
        (
            outro_criterio.html(),
            "criterios_da_visao_resumida_alterados_apos_restaurar",
        ),
    ] {
        let (resultado, registro) = listar(r().html(), None, vec![(200, d.html()), (200, volta)]);
        assert_eq!(
            registro,
            [
                "POST visao=D recebidos=0 gerados=0",
                "POST visao=R pagina=0"
            ]
        );
        let erro = resultado.unwrap_err();
        assert_eq!(erro.subcodigo(), "visao_do_controle_nao_restaurada");
        assert_eq!(erro.detalhes_json().unwrap()["causa"], causa);
    }
}

#[test]
fn volta_a_pagina_exibida_da_d_que_falha_diz_onde_a_d_ficou() {
    let d1 = TelaD::nova(5, 0, 2, vec![1, 2]);
    let d2 = TelaD::nova(5, 1, 2, vec![3, 9]);
    let (resultado, registro) =
        listar(d1.html(), Some(2), vec![(200, d2.html()), (200, d2.html())]);
    assert_eq!(registro.len(), 2);
    let erro = resultado.unwrap_err();
    assert_eq!(erro.subcodigo(), "pagina_da_visao_detalhada_nao_restaurada");
    let detalhes = erro.detalhes_json().unwrap();
    assert_eq!(detalhes["pagina_deixada"], 2);
    assert_eq!(
        detalhes["causa"],
        "faixa_da_resposta_diferente_da_pagina_solicitada"
    );
}

#[test]
fn troca_sem_d_na_resposta_nao_tem_de_onde_restaurar() {
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
        &arquivo,
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
