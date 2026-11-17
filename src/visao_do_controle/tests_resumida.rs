//! RED/GREEN de `recebidos|gerados` com a sessão na D e de `unidade colunas`.
use super::tests::{EnvioRoteirizado, TelaD, TelaR, cookie, instalacao, tela_atual, tela_nivel};
use super::*;
use crate::dominio::Categoria;
use crate::tela_html::HtmlServido;
use crate::transporte::sessao::CookieDaSessao;

fn r() -> TelaR {
    TelaR::nova((3, vec![1, 2]), (1, vec![3]))
}

fn d() -> TelaD {
    TelaD::nova(4, 0, 10, vec![1, 2, 3, 9])
}

fn resumida(
    tela: String,
    categoria: Categoria,
    pagina: Option<usize>,
    respostas: Vec<(i32, String)>,
) -> (Result<HtmlServido, ErroVisaoDoControle>, Vec<String>) {
    let (_pasta, arquivo) = cookie();
    let mut envio = EnvioRoteirizado::novo(respostas);
    let resultado = obter_na_visao_resumida(
        &instalacao(),
        &tela_atual(tela),
        Some("TESTE"),
        categoria,
        pagina,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(envio.respostas.is_empty(), "respostas não consumidas");
    (resultado, envio.registro)
}

#[test]
fn sessao_em_d_troca_para_r_le_a_lista_e_fica_na_r() {
    let (resultado, registro) = resumida(
        d().html(),
        Categoria::Gerados,
        None,
        vec![(200, r().html())],
    );
    assert_eq!(registro, ["POST visao=R pagina=0"]);
    let html = resultado.unwrap();
    assert!(html.contains("tblProcessosGerados"));
    assert!(html.contains("N-3"));
}

fn segunda() -> TelaR {
    let mut segunda = r();
    segunda.pagina_r = "1";
    segunda.recebidos = (3, 3, vec![4]);
    segunda
}

/// Decisão do proprietário (02/11/2026): a R fica lembrando a página N.
#[test]
fn sessao_em_d_com_pagina_n_fica_na_pagina_n_da_r() {
    let (resultado, registro) = resumida(
        d().html(),
        Categoria::Recebidos,
        Some(2),
        vec![(200, r().html()), (200, segunda().html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=R pagina=0",
            "POST visao=R recebidos=1 gerados=0"
        ]
    );
    let html = resultado.unwrap();
    assert!(html.contains("N-4"));
    assert!(!html.contains("N-1"));
}

#[test]
fn pagina_encontrada_da_r_e_a_da_resposta_da_troca() {
    let (resultado, registro) = resumida(
        d().html(),
        Categoria::Recebidos,
        Some(1),
        vec![(200, segunda().html()), (200, r().html())],
    );
    assert_eq!(
        registro,
        [
            "POST visao=R pagina=0",
            "POST visao=R recebidos=0 gerados=0"
        ]
    );
    assert!(resultado.unwrap().contains("N-1"));
}

#[test]
fn pagina_da_r_que_nao_confere_falha_sem_post_de_volta() {
    let mut outro_criterio = segunda();
    outro_criterio.hash_r = "9999";
    for resposta in [r().html(), outro_criterio.html()] {
        let (_resultado, registro) = resumida(
            d().html(),
            Categoria::Recebidos,
            Some(2),
            vec![(200, r().html()), (200, resposta)],
        );
        assert_eq!(
            registro,
            [
                "POST visao=R pagina=0",
                "POST visao=R recebidos=1 gerados=0"
            ]
        );
    }
}

#[test]
fn troca_para_r_recusada_pelo_servidor_sobe_o_status() {
    let (resultado, registro) = resumida(
        d().html(),
        Categoria::Recebidos,
        None,
        vec![(503, "<html><title>Indisponível</title></html>".to_owned())],
    );
    assert_eq!(registro, ["POST visao=R pagina=0"]);
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        ErroVisaoDoControle::Submissao(crate::formulario_web::ErroSubmissaoFormulario::Transporte(
            crate::transporte::FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                status: 503,
                ..
            })
        ))
    ));
}

#[test]
fn pagina_fora_do_intervalo_vista_na_r_fica_na_r() {
    let (resultado, registro) = resumida(
        d().html(),
        Categoria::Gerados,
        Some(2),
        vec![(200, r().html())],
    );
    assert_eq!(registro, ["POST visao=R pagina=0"]);
    let erro = resultado.unwrap_err();
    // Ao vivo (PEN1, 04/11/2026) a recusa saía como "visão não obtida",
    // embora a troca para R tivesse passado: a recusa é a da página da R.
    assert!(matches!(
        erro,
        ErroVisaoDoControle::Pagina(
            crate::pagina_do_controle::ErroPaginaDoControle::PaginaForaDoIntervalo {
                solicitada: 2,
                paginas: 1,
                ..
            }
        )
    ));
}

fn colunas(
    tela: String,
    respostas: Vec<(i32, String)>,
) -> (
    Result<(String, crate::nivel_de_detalhe::NivelDeDetalhe), ErroVisaoDoControle>,
    Vec<String>,
) {
    let (_pasta, arquivo) = cookie();
    let mut envio = EnvioRoteirizado::novo(respostas);
    let resultado = ler_colunas(
        &instalacao(),
        &tela_atual(tela),
        Some("TESTE"),
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(envio.respostas.is_empty(), "respostas não consumidas");
    (resultado, envio.registro)
}

#[test]
fn colunas_com_sessao_em_d_custam_um_get_a_mais() {
    let nivel = tela_nivel(&[
        "chkSinAtribuicao",
        "chkSinTipoProcesso",
        "chkSinInteressados",
    ]);
    let (resultado, registro) = colunas(d().html(), vec![(200, nivel)]);
    assert_eq!(registro, ["GET procedimento_configurar_detalhe"]);
    let (unidade, nivel) = resultado.unwrap();
    assert_eq!(unidade, "TESTE");
    assert_eq!(nivel.opcoes.iter().filter(|o| o.marcada).count(), 3);
}

#[test]
fn colunas_com_sessao_em_r_trocam_leem_e_ficam_na_d() {
    let r = TelaR::nova((2, vec![1, 2]), (2, vec![3, 9]));
    let (resultado, registro) = colunas(r.html(), vec![(200, d().html()), (200, tela_nivel(&[]))]);
    assert_eq!(
        registro,
        [
            "POST visao=D recebidos=0 gerados=0",
            "GET procedimento_configurar_detalhe"
        ]
    );
    assert!(resultado.unwrap().1.opcoes.iter().all(|o| !o.marcada));
}

/// Ensaio live `UnB` 4.0.9 (02/11/2026): 11 opções, sem Prioridade.
#[test]
fn colunas_da_unb_com_onze_opcoes_publicam_a_prioridade_indisponivel() {
    let publicadas: Vec<_> = crate::nivel_de_detalhe::ColunaDoDetalhe::TODAS
        .into_iter()
        .filter(|c| *c != crate::nivel_de_detalhe::ColunaDoDetalhe::Prioridade)
        .collect();
    let nivel = crate::nivel_de_detalhe::tests::tela_com(&publicadas, &["chkSinAtribuicao"]);
    let (resultado, registro) = colunas(d().html(), vec![(200, nivel)]);
    assert_eq!(registro, ["GET procedimento_configurar_detalhe"]);
    let (_, nivel) = resultado.unwrap();
    assert_eq!(nivel.opcoes.len(), 12);
    let indisponiveis: Vec<_> = nivel
        .opcoes
        .iter()
        .filter(|o| !o.disponivel)
        .map(|o| o.coluna)
        .collect();
    assert_eq!(
        indisponiveis,
        [crate::nivel_de_detalhe::ColunaDoDetalhe::Prioridade]
    );
}

#[test]
fn d_sem_link_do_nivel_de_detalhe_e_recusada_sem_get() {
    let mut sem = d();
    sem.link_nivel = false;
    let (resultado, registro) = colunas(sem.html(), vec![]);
    assert_eq!(registro.len(), 0);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::LinkDoNivelDeDetalheAusente
    );
}

#[test]
fn link_do_nivel_com_causa_de_consulta_ou_chave_repetida() {
    let repetida = d().html().replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=procedimento_configurar_detalhe&amp;acao=procedimento_configurar_detalhe&amp;",
        1,
    );
    let (resultado, _) = colunas(repetida, vec![]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::AcaoDoNivelIlegivel(
            crate::consulta_do_controlador::ErroAcaoDaUrl::AcaoRepetida
        )
    );
    let outra_acao = d().html().replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=procedimento_excluir&amp;",
        1,
    );
    let (resultado, _) = colunas(outra_acao, vec![]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::LinkDoNivelDeDetalheNaoReconhecido
    );
    let par_sem_igual = d().html().replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "lixo&amp;acao=procedimento_configurar_detalhe&amp;",
        1,
    );
    let (resultado, _) = colunas(par_sem_igual, vec![]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroVisaoDoControle::ConsultaDoNivelIlegivel(
            crate::consulta_do_controlador::ErroConsultaDoControlador::ParSemIgual
        )
    );
}

#[test]
fn link_do_nivel_de_detalhe_fora_da_forma_observada_e_recusado() {
    for troca in [
        (
            "acaoConfigurarDetalhe('controlador.php?",
            "acaoConfigurarDetalhe('https://outro.gov.br/controlador.php?",
        ),
        ("infra_hash=eeee", "infra_hash=zzzz"),
    ] {
        let html = d().html().replacen(troca.0, troca.1, 1);
        let (resultado, registro) = colunas(html, vec![]);
        assert_eq!(registro.len(), 0, "{troca:?}");
        assert_eq!(
            resultado.unwrap_err(),
            ErroVisaoDoControle::LinkDoNivelDeDetalheNaoReconhecido,
            "{troca:?}"
        );
    }
}
