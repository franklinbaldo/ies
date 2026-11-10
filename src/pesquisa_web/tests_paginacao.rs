//! RED/GREEN da regra `navegar(inicio)`: página N, tamanho da barra e
//! intervalo, sem rede. HTML sintético com a estrutura observada.
use super::*;
use crate::dominio::IdProcedimento;
use crate::pesquisa_web::resposta::ler_resposta_html;
use crate::pesquisa_web::tests_fixtures::resultado;

fn lista(inicio: u64, fim: u64, total: &str) -> (RespostaDaPesquisa, String) {
    let html = resultado("00001", inicio, fim, total);
    (ler_resposta_html(&html).unwrap(), html)
}

fn planejar(
    html: &str,
    resposta: &RespostaDaPesquisa,
    n: u32,
) -> Result<Option<PedidoDePaginaDaPesquisa>, ErroPaginacaoDaPesquisa> {
    planejar_pagina(resposta, Some(html), n)
}

fn faixa(inicio: u64, fim: u64) -> FaixaDaPesquisa {
    FaixaDaPesquisa { inicio, fim }
}

#[test]
fn pagina_1_nao_pede_post_de_pagina() {
    let (resposta, html) = lista(1, 10, "1.234");
    assert_eq!(planejar(&html, &resposta, 1), Ok(None));
}

#[test]
fn pagina_2_usa_o_tamanho_da_barra() {
    let (resposta, html) = lista(1, 10, "1.234");
    assert_eq!(
        planejar(&html, &resposta, 2),
        Ok(Some(PedidoDePaginaDaPesquisa {
            numero: 2,
            inicio: 10,
            total: 1234,
            esperada: faixa(11, 20),
        }))
    );
}

#[test]
fn ultima_pagina_tem_faixa_parcial_e_a_seguinte_esta_fora() {
    let (resposta, html) = lista(1, 10, "1.234");
    assert_eq!(
        planejar(&html, &resposta, 124),
        Ok(Some(PedidoDePaginaDaPesquisa {
            numero: 124,
            inicio: 1230,
            total: 1234,
            esperada: faixa(1231, 1234),
        }))
    );
    assert_eq!(
        planejar(&html, &resposta, 125),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 125,
            paginas: 124
        })
    );
}

#[test]
fn lista_de_uma_pagina_so_tem_a_pagina_1() {
    let (resposta, html) = lista(1, 3, "3");
    assert_eq!(planejar(&html, &resposta, 1), Ok(None));
    assert_eq!(
        planejar(&html, &resposta, 2),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 2,
            paginas: 1
        })
    );
}

#[test]
fn sem_resultado_e_processo_unico_nao_tem_pagina_2() {
    assert_eq!(
        planejar_pagina(&RespostaDaPesquisa::SemResultado, None, 2),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 2,
            paginas: 0
        })
    );
    assert_eq!(
        planejar_pagina(&RespostaDaPesquisa::SemResultado, None, 1),
        Ok(None)
    );
    let unico = RespostaDaPesquisa::ProcessoUnico {
        id: IdProcedimento::novo("7").unwrap(),
    };
    assert_eq!(planejar_pagina(&unico, None, 1), Ok(None));
    assert_eq!(
        planejar_pagina(&unico, None, 2),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 2,
            paginas: 1
        })
    );
}

#[test]
fn sem_as_linhas_literais_de_navegar_a_regra_nao_e_observada() {
    let (resposta, html) = lista(1, 10, "1.234");
    for trecho in [
        "document.getElementById('hdnInicio').value = inicio;",
        "document.getElementById('frmPesquisaProtocolo').submit();",
    ] {
        let alterado = html.replace(trecho, "outra();");
        assert_eq!(
            planejar(&alterado, &resposta, 2),
            Err(ErroPaginacaoDaPesquisa::RegraDeNavegacaoDaPesquisaNaoObservada),
            "{trecho}"
        );
    }
}

#[test]
fn pagina_fora_do_intervalo_vem_antes_da_regra() {
    let (resposta, html) = lista(1, 10, "1.234");
    let sem_regra = html.replace("function navegar", "function outra");
    assert_eq!(
        planejar(&sem_regra, &resposta, 500),
        Err(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 500,
            paginas: 124
        })
    );
}

#[test]
fn links_de_navegar_fora_da_regra_sao_recusados() {
    let (resposta, html) = lista(1, 10, "1.234");
    for errado in [
        "navegar('15')",
        "navegar('1240')",
        "navegar('x')",
        "navegar(10)",
    ] {
        let alterado = html.replacen("navegar('10')", errado, 1);
        assert_eq!(
            planejar(&alterado, &resposta, 2),
            Err(ErroPaginacaoDaPesquisa::LinkDePaginacaoDaPesquisaForaDaRegra),
            "{errado}"
        );
    }
}

#[test]
fn varias_paginas_sem_links_de_navegar_e_paginador_nao_publicado() {
    let (resposta, html) = lista(1, 10, "1.234");
    let sem_links = html.replace("javascript:navegar(", "javascript:outra(");
    assert_eq!(
        planejar(&sem_links, &resposta, 2),
        Err(ErroPaginacaoDaPesquisa::PaginadorDaPesquisaNaoPublicado)
    );
}

fn pedido() -> PedidoDePaginaDaPesquisa {
    PedidoDePaginaDaPesquisa {
        numero: 2,
        inicio: 10,
        total: 1234,
        esperada: faixa(11, 20),
    }
}

#[test]
fn pagina_obtida_com_total_e_faixa_esperados_e_aceita() {
    let (resposta, _) = lista(11, 20, "1.234");
    let lida = conferir_pagina(&pedido(), resposta).unwrap();
    assert_eq!(lida.faixa, faixa(11, 20));
    assert_eq!(lida.processos.len(), 10);
}

#[test]
fn pagina_obtida_diferente_do_pedido_e_recusada() {
    let (outro_total, _) = lista(11, 20, "1.235");
    assert_eq!(
        conferir_pagina(&pedido(), outro_total),
        Err(ErroPaginacaoDaPesquisa::TotalDaPesquisaAlteradoAposPaginar)
    );
    let (outra_faixa, _) = lista(1, 10, "1.234");
    assert_eq!(
        conferir_pagina(&pedido(), outra_faixa),
        Err(ErroPaginacaoDaPesquisa::FaixaDaPesquisaDiferenteDaPaginaSolicitada)
    );
    assert_eq!(
        conferir_pagina(&pedido(), RespostaDaPesquisa::SemResultado),
        Err(ErroPaginacaoDaPesquisa::PaginaDaPesquisaSemResultadoAposPaginar)
    );
    let unico = RespostaDaPesquisa::ProcessoUnico {
        id: IdProcedimento::novo("7").unwrap(),
    };
    assert_eq!(
        conferir_pagina(&pedido(), unico),
        Err(ErroPaginacaoDaPesquisa::RedirecionamentoAoPaginarAPesquisa)
    );
}

#[test]
fn paginar_pesquisa_declara_submit_por_script_e_so_hdn_inicio() {
    assert_eq!(
        PAGINAR_PESQUISA_DE_PROTOCOLO.envio,
        EnvioDoFormulario::SubmitPorScript
    );
    assert_eq!(
        PAGINAR_PESQUISA_DE_PROTOCOLO.campos_ocultos_reescritos,
        ["hdnInicio"]
    );
    assert_eq!(
        PAGINAR_PESQUISA_DE_PROTOCOLO.efeito,
        EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado
        }
    );
}
